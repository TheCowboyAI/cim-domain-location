//! NATS event publisher adapter
//!
//! This adapter implements the EventPublisher port using NATS JetStream.

use crate::ports::{EventPublisher, PublishError, QueryError, event_to_subject};
use crate::LocationDomainEvent;
use async_nats::jetstream;
use async_trait::async_trait;
use cim_domain::DomainEvent;
use futures::StreamExt;
use serde_json;
use uuid::Uuid;

/// NATS-based event publisher
pub struct NatsEventPublisher {
    jetstream: jetstream::Context,
    stream_name: String,
}

impl NatsEventPublisher {
    /// Create a new NATS event publisher
    pub fn new(jetstream: jetstream::Context, stream_name: String) -> Self {
        Self {
            jetstream,
            stream_name,
        }
    }

    /// Get correlation ID from event
    fn get_correlation_id(event: &LocationDomainEvent) -> Option<Uuid> {
        // Events don't currently have correlation IDs
        // This would need to be added to event structs
        None
    }

    /// Get timestamp from event
    fn get_timestamp(event: &LocationDomainEvent) -> Option<chrono::DateTime<chrono::Utc>> {
        // Events don't currently have timestamps
        // This would need to be added to event structs
        None
    }
}

#[async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn publish(&self, event: &LocationDomainEvent) -> Result<(), PublishError> {
        let subject = event_to_subject(event);
        let payload = serde_json::to_vec(event)
            .map_err(|e| PublishError::SerializationError(e.to_string()))?;

        // Add event metadata as headers
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("event-type", event.event_type());
        headers.insert("aggregate-id", event.aggregate_id().to_string().as_str());

        if let Some(correlation_id) = Self::get_correlation_id(event) {
            headers.insert("correlation-id", correlation_id.to_string().as_str());
        }

        if let Some(timestamp) = Self::get_timestamp(event) {
            headers.insert("timestamp", timestamp.to_rfc3339().as_str());
        }

        self.jetstream
            .publish_with_headers(subject, headers, payload.into())
            .await
            .map_err(|e| PublishError::PublishFailed(e.to_string()))?
            .await
            .map_err(|e| PublishError::PublishFailed(e.to_string()))?;

        Ok(())
    }

    async fn publish_batch(&self, events: &[LocationDomainEvent]) -> Result<(), PublishError> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }

    async fn query_by_correlation(&self, correlation_id: Uuid) -> Result<Vec<LocationDomainEvent>, QueryError> {
        // This would require scanning all events and filtering by correlation_id header
        // For now, return empty vector
        // TODO: Implement proper correlation ID indexing
        Ok(Vec::new())
    }

    async fn query_by_aggregate(&self, aggregate_id: Uuid) -> Result<Vec<LocationDomainEvent>, QueryError> {
        let subject = format!("events.location.{}.>", aggregate_id);

        let stream = self
            .jetstream
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| QueryError::QueryFailed(e.to_string()))?;

        let consumer_name = format!("query-{}", Uuid::new_v4());

        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                filter_subject: subject,
                ..Default::default()
            })
            .await
            .map_err(|e| QueryError::ConsumerError(e.to_string()))?;

        let mut messages = consumer
            .messages()
            .await
            .map_err(|e| QueryError::QueryFailed(e.to_string()))?;

        let mut events = Vec::new();

        while let Some(Ok(msg)) = messages.next().await {
            let event: LocationDomainEvent = serde_json::from_slice(&msg.payload)
                .map_err(|e| QueryError::DeserializationError(e.to_string()))?;

            events.push(event);

            msg.ack()
                .await
                .map_err(|e| QueryError::QueryFailed(e.to_string()))?;
        }

        Ok(events)
    }

    async fn query_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<LocationDomainEvent>, QueryError> {
        // This would require filtering by timestamp header
        // For now, return empty vector
        // TODO: Implement proper timestamp-based querying
        Ok(Vec::new())
    }
}
