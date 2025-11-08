//! Location repository with event sourcing
//!
//! This repository reconstructs Location aggregates from their event history
//! stored in NATS JetStream.

use crate::aggregate::{Location, LocationMarker};
use crate::infrastructure::{NatsError, NatsEventStore};
use crate::LocationDomainEvent;
use cim_domain::{DomainResult, EntityId};
use std::sync::Arc;
use uuid::Uuid;

/// Repository for Location aggregates using event sourcing
pub struct LocationRepository {
    event_store: Arc<NatsEventStore>,
    snapshot_frequency: u64,
}

impl LocationRepository {
    /// Create a new location repository
    ///
    /// # Arguments
    /// * `event_store` - The NATS event store for persistence
    /// * `snapshot_frequency` - How often to create snapshots (0 = never)
    pub fn new(event_store: Arc<NatsEventStore>) -> Self {
        Self {
            event_store,
            snapshot_frequency: 100, // Default: snapshot every 100 events
        }
    }

    /// Set the snapshot frequency
    pub fn with_snapshot_frequency(mut self, frequency: u64) -> Self {
        self.snapshot_frequency = frequency;
        self
    }

    /// Load a location aggregate by ID
    ///
    /// This reconstructs the aggregate from its event history.
    pub async fn load(&self, location_id: EntityId<LocationMarker>) -> Result<Option<Location>, RepositoryError> {
        let uuid_id: Uuid = location_id.into();

        // Load all events for this aggregate
        let events = self
            .event_store
            .load_events(uuid_id)
            .await
            .map_err(|e| RepositoryError::EventStoreFailed(e.to_string()))?;

        if events.is_empty() {
            return Ok(None);
        }

        // Reconstruct aggregate from events
        let mut location = None;

        for event in events {
            match &location {
                None => {
                    // First event must be LocationDefined
                    if let LocationDomainEvent::LocationDefined(e) = &event {
                        // Create initial aggregate from LocationDefined event
                        location = Some(self.create_from_defined_event(e)?);
                    } else {
                        return Err(RepositoryError::InvalidEventSequence(
                            "First event must be LocationDefined".to_string(),
                        ));
                    }
                }
                Some(loc) => {
                    // Apply subsequent events
                    let new_loc = loc
                        .apply_event_pure(&event)
                        .map_err(|e| RepositoryError::EventApplicationFailed(e.to_string()))?;
                    location = Some(new_loc);
                }
            }
        }

        Ok(location)
    }

    /// Save events for a location aggregate
    ///
    /// This appends new events to the event store.
    pub async fn save(&self, events: Vec<LocationDomainEvent>) -> Result<(), RepositoryError> {
        self.event_store
            .append_events(events)
            .await
            .map_err(|e| RepositoryError::EventStoreFailed(e.to_string()))?;

        Ok(())
    }

    /// Helper to create initial aggregate from LocationDefined event
    fn create_from_defined_event(
        &self,
        event: &crate::events::LocationDefined,
    ) -> Result<Location, RepositoryError> {
        use crate::value_objects::LocationType;

        let location_id = EntityId::from_uuid(event.location_id);

        // Create location based on type
        let location = match &event.location_type {
            LocationType::Physical => {
                if let Some(address) = &event.address {
                    Location::new_physical(location_id, event.name.clone(), address.clone())
                } else if let Some(coords) = &event.coordinates {
                    Location::new_from_coordinates(
                        location_id,
                        event.name.clone(),
                        coords.clone(),
                    )
                } else {
                    return Err(RepositoryError::InvalidEvent(
                        "Physical location requires address or coordinates".to_string(),
                    ));
                }
            }
            LocationType::Virtual => {
                if let Some(virtual_loc) = &event.virtual_location {
                    Location::new_virtual(location_id, event.name.clone(), virtual_loc.clone())
                } else {
                    return Err(RepositoryError::InvalidEvent(
                        "Virtual location requires virtual location details".to_string(),
                    ));
                }
            }
            // For Logical and Hybrid, we'll use coordinates if available, otherwise address
            _ => {
                if let Some(coords) = &event.coordinates {
                    Location::new_from_coordinates(
                        location_id,
                        event.name.clone(),
                        coords.clone(),
                    )
                } else if let Some(address) = &event.address {
                    Location::new_physical(location_id, event.name.clone(), address.clone())
                } else {
                    return Err(RepositoryError::InvalidEvent(
                        "Location requires either address or coordinates".to_string(),
                    ));
                }
            }
        }
        .map_err(|e| RepositoryError::AggregateCreationFailed(e.to_string()))?;

        Ok(location)
    }
}

/// Errors that can occur during repository operations
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Event store operation failed: {0}")]
    EventStoreFailed(String),

    #[error("Invalid event sequence: {0}")]
    InvalidEventSequence(String),

    #[error("Event application failed: {0}")]
    EventApplicationFailed(String),

    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    #[error("Aggregate creation failed: {0}")]
    AggregateCreationFailed(String),

    #[error("Aggregate not found")]
    AggregateNotFound,
}
