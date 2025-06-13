//! Domain events enum for location domain

use crate::events::LocationDefined;
use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};

/// Enum wrapper for location domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationDomainEvent {
    /// A location was defined
    LocationDefined(LocationDefined),
}

impl DomainEvent for LocationDomainEvent {
    fn subject(&self) -> String {
        match self {
            Self::LocationDefined(e) => e.subject(),
        }
    }

    fn aggregate_id(&self) -> uuid::Uuid {
        match self {
            Self::LocationDefined(e) => e.aggregate_id(),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            Self::LocationDefined(e) => e.event_type(),
        }
    }
}
