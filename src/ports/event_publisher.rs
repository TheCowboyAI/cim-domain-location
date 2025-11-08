//! Event publisher port for NATS JetStream
//!
//! This port defines the interface for publishing events to NATS JetStream.
//! The actual implementation (adapter) would be injected at runtime.

use async_trait::async_trait;
use crate::LocationDomainEvent;
use cim_domain::DomainEvent;
use uuid::Uuid;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a location event to NATS JetStream
    async fn publish(&self, event: &LocationDomainEvent) -> Result<(), PublishError>;

    /// Publish multiple events as a batch
    async fn publish_batch(&self, events: &[LocationDomainEvent]) -> Result<(), PublishError>;

    /// Query events by correlation ID from JetStream
    async fn query_by_correlation(&self, correlation_id: Uuid) -> Result<Vec<LocationDomainEvent>, QueryError>;

    /// Query events by aggregate ID from JetStream
    async fn query_by_aggregate(&self, aggregate_id: Uuid) -> Result<Vec<LocationDomainEvent>, QueryError>;

    /// Query events within a time range
    async fn query_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<LocationDomainEvent>, QueryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Failed to connect to NATS: {0}")]
    ConnectionError(String),

    #[error("Failed to publish event: {0}")]
    PublishFailed(String),

    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Failed to query events: {0}")]
    QueryFailed(String),

    #[error("Consumer error: {0}")]
    ConsumerError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// Helper to determine the NATS subject for an event
pub fn event_to_subject(event: &LocationDomainEvent) -> String {
    let location_id = event.aggregate_id();

    match event {
        LocationDomainEvent::LocationDefined(_) => {
            format!("events.location.{}.defined", location_id)
        }
        LocationDomainEvent::LocationUpdated(_) => {
            format!("events.location.{}.updated", location_id)
        }
        LocationDomainEvent::ParentLocationSet(_) => {
            format!("events.location.{}.parent.set", location_id)
        }
        LocationDomainEvent::ParentLocationRemoved(_) => {
            format!("events.location.{}.parent.removed", location_id)
        }
        LocationDomainEvent::LocationMetadataAdded(_) => {
            format!("events.location.{}.metadata.added", location_id)
        }
        LocationDomainEvent::LocationArchived(_) => {
            format!("events.location.{}.archived", location_id)
        }
    }
}
