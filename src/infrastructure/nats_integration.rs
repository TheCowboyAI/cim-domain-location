//! NATS JetStream integration for location domain events
//!
//! This module provides event store implementation using NATS JetStream
//! for durable, distributed event storage and replay.

use crate::LocationDomainEvent;
use async_nats::jetstream::{self, stream::Stream};
use cim_domain::DomainEvent;
use futures::StreamExt;
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

/// NATS-based event store using JetStream
pub struct NatsEventStore {
    jetstream: jetstream::Context,
    stream: Stream,
    stream_name: String,
}

impl NatsEventStore {
    /// Create a new NATS event store
    ///
    /// This will create or update the JetStream stream with the given name.
    /// The stream will be configured to store all events under "events.location.>"
    pub async fn new(
        jetstream: jetstream::Context,
        stream_name: String,
    ) -> Result<Self, NatsError> {
        // Create or get the stream
        let stream = jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: stream_name.clone(),
                subjects: vec!["events.location.>".to_string()],
                max_age: std::time::Duration::from_secs(365 * 24 * 60 * 60), // 1 year
                storage: jetstream::stream::StorageType::File,
                num_replicas: 1,
                ..Default::default()
            })
            .await
            .map_err(|e| NatsError::StreamCreationFailed(e.to_string()))?;

        Ok(Self {
            jetstream,
            stream,
            stream_name,
        })
    }

    /// Append events to the event store
    pub async fn append_events(
        &self,
        events: Vec<LocationDomainEvent>,
    ) -> Result<(), NatsError> {
        for event in events {
            self.append_event(event).await?;
        }
        Ok(())
    }

    /// Append a single event to the event store
    pub async fn append_event(&self, event: LocationDomainEvent) -> Result<(), NatsError> {
        let subject = self.event_subject(&event);
        let payload =
            serde_json::to_vec(&event).map_err(|e| NatsError::SerializationError(e.to_string()))?;

        // Add event metadata as headers
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("event-type", event.event_type());
        headers.insert("aggregate-id", event.aggregate_id().to_string().as_str());

        self.jetstream
            .publish_with_headers(subject, headers, payload.into())
            .await
            .map_err(|e| NatsError::PublishFailed(e.to_string()))?
            .await
            .map_err(|e| NatsError::PublishFailed(e.to_string()))?;

        Ok(())
    }

    /// Load all events for a given aggregate ID
    pub async fn load_events(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Vec<LocationDomainEvent>, NatsError> {
        let subject = format!("events.location.{}.>", aggregate_id);

        // Create a durable consumer for this aggregate
        let consumer_name = format!("location-{}", aggregate_id);

        let consumer = self
            .stream
            .get_or_create_consumer(
                &consumer_name,
                jetstream::consumer::pull::Config {
                    filter_subject: subject.clone(),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| NatsError::ConsumerCreationFailed(e.to_string()))?;

        let mut messages = consumer
            .messages()
            .await
            .map_err(|e| NatsError::FetchFailed(e.to_string()))?;

        let mut events = Vec::new();

        // Fetch all available messages
        while let Some(Ok(msg)) = messages.next().await {
            let event: LocationDomainEvent = serde_json::from_slice(&msg.payload)
                .map_err(|e| NatsError::DeserializationError(e.to_string()))?;

            events.push(event);

            msg.ack()
                .await
                .map_err(|e| NatsError::AckFailed(e.to_string()))?;
        }

        Ok(events)
    }

    /// Get the NATS subject for an event
    fn event_subject(&self, event: &LocationDomainEvent) -> String {
        let location_id = event.aggregate_id();

        let event_type = match event {
            LocationDomainEvent::LocationDefined(_) => "defined",
            LocationDomainEvent::LocationUpdated(_) => "updated",
            LocationDomainEvent::ParentLocationSet(_) => "parent_set",
            LocationDomainEvent::ParentLocationRemoved(_) => "parent_removed",
            LocationDomainEvent::LocationMetadataAdded(_) => "metadata_added",
            LocationDomainEvent::LocationArchived(_) => "archived",
        };

        format!("events.location.{}.{}", location_id, event_type)
    }
}

/// Errors that can occur during NATS operations
#[derive(Debug, thiserror::Error)]
pub enum NatsError {
    #[error("Failed to create stream: {0}")]
    StreamCreationFailed(String),

    #[error("Failed to publish event: {0}")]
    PublishFailed(String),

    #[error("Failed to create consumer: {0}")]
    ConsumerCreationFailed(String),

    #[error("Failed to fetch messages: {0}")]
    FetchFailed(String),

    #[error("Failed to acknowledge message: {0}")]
    AckFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}
