//! Location domain events

use crate::aggregate::{LocationType, Address, GeoCoordinates, VirtualLocation};
use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Location defined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationDefined {
    /// The unique identifier of the location
    pub location_id: Uuid,
    /// The name of the location
    pub name: String,
    /// The type of location (physical, virtual, etc.)
    pub location_type: LocationType,
    /// The physical address (if applicable)
    pub address: Option<Address>,
    /// The geographic coordinates (if applicable)
    pub coordinates: Option<GeoCoordinates>,
    /// Virtual location details (if applicable)
    pub virtual_location: Option<VirtualLocation>,
    /// The parent location ID (for hierarchical locations)
    pub parent_id: Option<Uuid>,
}

impl DomainEvent for LocationDefined {
    fn aggregate_id(&self) -> Uuid {
        self.location_id
    }

    fn event_type(&self) -> &'static str {
        "LocationDefined"
    }

    fn subject(&self) -> String {
        format!("location.{}.defined", self.location_id)
    }
}
