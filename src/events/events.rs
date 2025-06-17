//! Location domain events

use crate::aggregate::{LocationType, Address, GeoCoordinates, VirtualLocation};
use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Location details updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationUpdated {
    /// The unique identifier of the location
    pub location_id: Uuid,
    /// Previous name
    pub previous_name: Option<String>,
    /// New name  
    pub name: Option<String>,
    /// Previous address
    pub previous_address: Option<Address>,
    /// New address
    pub address: Option<Address>,
    /// Previous coordinates
    pub previous_coordinates: Option<GeoCoordinates>,
    /// New coordinates
    pub coordinates: Option<GeoCoordinates>,
    /// Previous virtual location
    pub previous_virtual_location: Option<VirtualLocation>,
    /// New virtual location
    pub virtual_location: Option<VirtualLocation>,
    /// Reason for update
    pub reason: String,
}

/// Parent location set for hierarchical structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentLocationSet {
    /// Child location ID
    pub location_id: Uuid,
    /// Parent location ID
    pub parent_id: Uuid,
    /// Previous parent (if any)
    pub previous_parent_id: Option<Uuid>,
    /// Reason for setting parent
    pub reason: String,
}

/// Parent location removed (made top-level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentLocationRemoved {
    /// Location ID that was made top-level
    pub location_id: Uuid,
    /// Previous parent location ID
    pub previous_parent_id: Uuid,
    /// Reason for removing parent
    pub reason: String,
}

/// Metadata added to location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationMetadataAdded {
    /// Location ID
    pub location_id: Uuid,
    /// Metadata that was added
    pub added_metadata: HashMap<String, String>,
    /// All metadata after addition
    pub current_metadata: HashMap<String, String>,
    /// Reason for adding metadata
    pub reason: String,
}

/// Location archived (soft deleted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationArchived {
    /// Location ID that was archived
    pub location_id: Uuid,
    /// Name of the archived location
    pub name: String,
    /// Type of the archived location
    pub location_type: LocationType,
    /// Reason for archiving
    pub reason: String,
}

/// Base trait for location events
pub trait LocationEvent: DomainEvent {
    fn location_id(&self) -> Uuid;
}

// DomainEvent implementations
impl DomainEvent for LocationDefined {
    fn aggregate_id(&self) -> Uuid { self.location_id }
    fn event_type(&self) -> &'static str { "LocationDefined" }
    fn subject(&self) -> String {
        format!("location.{}.defined", self.location_id)
    }
}

impl LocationEvent for LocationDefined {
    fn location_id(&self) -> Uuid { self.location_id }
}

impl DomainEvent for LocationUpdated {
    fn aggregate_id(&self) -> Uuid { self.location_id }
    fn event_type(&self) -> &'static str { "LocationUpdated" }
    fn subject(&self) -> String {
        format!("location.{}.updated", self.location_id)
    }
}

impl LocationEvent for LocationUpdated {
    fn location_id(&self) -> Uuid { self.location_id }
}

impl DomainEvent for ParentLocationSet {
    fn aggregate_id(&self) -> Uuid { self.location_id }
    fn event_type(&self) -> &'static str { "ParentLocationSet" }
    fn subject(&self) -> String {
        format!("location.{}.parent_set", self.location_id)
    }
}

impl LocationEvent for ParentLocationSet {
    fn location_id(&self) -> Uuid { self.location_id }
}

impl DomainEvent for ParentLocationRemoved {
    fn aggregate_id(&self) -> Uuid { self.location_id }
    fn event_type(&self) -> &'static str { "ParentLocationRemoved" }
    fn subject(&self) -> String {
        format!("location.{}.parent_removed", self.location_id)
    }
}

impl LocationEvent for ParentLocationRemoved {
    fn location_id(&self) -> Uuid { self.location_id }
}

impl DomainEvent for LocationMetadataAdded {
    fn aggregate_id(&self) -> Uuid { self.location_id }
    fn event_type(&self) -> &'static str { "LocationMetadataAdded" }
    fn subject(&self) -> String {
        format!("location.{}.metadata_added", self.location_id)
    }
}

impl LocationEvent for LocationMetadataAdded {
    fn location_id(&self) -> Uuid { self.location_id }
}

impl DomainEvent for LocationArchived {
    fn aggregate_id(&self) -> Uuid { self.location_id }
    fn event_type(&self) -> &'static str { "LocationArchived" }
    fn subject(&self) -> String {
        format!("location.{}.archived", self.location_id)
    }
}

impl LocationEvent for LocationArchived {
    fn location_id(&self) -> Uuid { self.location_id }
}
