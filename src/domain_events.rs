//! Domain events enum for location domain

use crate::events::{
    LocationArchived, LocationDefined, LocationMetadataAdded, LocationUpdated,
    ParentLocationRemoved, ParentLocationSet,
};
use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};

/// Enum wrapper for location domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationDomainEvent {
    /// A location was defined
    LocationDefined(LocationDefined),
    /// A location was updated
    LocationUpdated(LocationUpdated),
    /// A parent location was set
    ParentLocationSet(ParentLocationSet),
    /// A parent location was removed
    ParentLocationRemoved(ParentLocationRemoved),
    /// Metadata was added to a location
    LocationMetadataAdded(LocationMetadataAdded),
    /// A location was archived
    LocationArchived(LocationArchived),
}

impl DomainEvent for LocationDomainEvent {
    fn aggregate_id(&self) -> uuid::Uuid {
        match self {
            Self::LocationDefined(e) => e.aggregate_id(),
            Self::LocationUpdated(e) => e.aggregate_id(),
            Self::ParentLocationSet(e) => e.aggregate_id(),
            Self::ParentLocationRemoved(e) => e.aggregate_id(),
            Self::LocationMetadataAdded(e) => e.aggregate_id(),
            Self::LocationArchived(e) => e.aggregate_id(),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            Self::LocationDefined(e) => e.event_type(),
            Self::LocationUpdated(e) => e.event_type(),
            Self::ParentLocationSet(e) => e.event_type(),
            Self::ParentLocationRemoved(e) => e.event_type(),
            Self::LocationMetadataAdded(e) => e.event_type(),
            Self::LocationArchived(e) => e.event_type(),
        }
    }
}
