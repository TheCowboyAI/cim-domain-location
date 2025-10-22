//! Location domain events

use crate::value_objects::{Address, GeoCoordinates, LocationType, VirtualLocation};
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
    fn aggregate_id(&self) -> Uuid {
        self.location_id
    }
    fn event_type(&self) -> &'static str {
        "LocationDefined"
    }
}

impl LocationEvent for LocationDefined {
    fn location_id(&self) -> Uuid {
        self.location_id
    }
}

impl DomainEvent for LocationUpdated {
    fn aggregate_id(&self) -> Uuid {
        self.location_id
    }
    fn event_type(&self) -> &'static str {
        "LocationUpdated"
    }
}

impl LocationEvent for LocationUpdated {
    fn location_id(&self) -> Uuid {
        self.location_id
    }
}

impl DomainEvent for ParentLocationSet {
    fn aggregate_id(&self) -> Uuid {
        self.location_id
    }
    fn event_type(&self) -> &'static str {
        "ParentLocationSet"
    }
}

impl LocationEvent for ParentLocationSet {
    fn location_id(&self) -> Uuid {
        self.location_id
    }
}

impl DomainEvent for ParentLocationRemoved {
    fn aggregate_id(&self) -> Uuid {
        self.location_id
    }
    fn event_type(&self) -> &'static str {
        "ParentLocationRemoved"
    }
}

impl LocationEvent for ParentLocationRemoved {
    fn location_id(&self) -> Uuid {
        self.location_id
    }
}

impl DomainEvent for LocationMetadataAdded {
    fn aggregate_id(&self) -> Uuid {
        self.location_id
    }
    fn event_type(&self) -> &'static str {
        "LocationMetadataAdded"
    }
}

impl LocationEvent for LocationMetadataAdded {
    fn location_id(&self) -> Uuid {
        self.location_id
    }
}

impl DomainEvent for LocationArchived {
    fn aggregate_id(&self) -> Uuid {
        self.location_id
    }
    fn event_type(&self) -> &'static str {
        "LocationArchived"
    }
}

impl LocationEvent for LocationArchived {
    fn location_id(&self) -> Uuid {
        self.location_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test LocationDefined event
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Event] --> B[Verify Fields]
    ///     B --> C[Test Serialization]
    ///     C --> D[Test DomainEvent]
    /// ```
    #[test]
    fn test_location_defined_event() {
        let location_id = Uuid::new_v4();
        let address = Address::new(
            "123 Main St".to_string(),
            "City".to_string(),
            "State".to_string(),
            "Country".to_string(),
            "12345".to_string(),
        );

        let event = LocationDefined {
            location_id,
            name: "Test Location".to_string(),
            location_type: LocationType::Physical,
            address: Some(address.clone()),
            coordinates: None,
            virtual_location: None,
            parent_id: None,
        };

        // Test LocationEvent trait
        assert_eq!(event.location_id(), location_id);

        // Test DomainEvent trait
        assert_eq!(event.aggregate_id(), location_id);
        assert_eq!(event.event_type(), "LocationDefined");
        assert_eq!(event.subject(), format!("location.{location_id}.defined"));

        // Test serialization
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: LocationDefined = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.location_id, location_id);
        assert_eq!(deserialized.name, "Test Location");
        assert_eq!(deserialized.address, Some(address));
    }

    /// Test LocationUpdated event
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Update] --> B[Previous Values]
    ///     B --> C[New Values]
    ///     C --> D[Verify Changes]
    /// ```
    #[test]
    fn test_location_updated_event() {
        let location_id = Uuid::new_v4();
        let old_address = Address::new(
            "123 Main St".to_string(),
            "Old City".to_string(),
            "OS".to_string(),
            "Country".to_string(),
            "11111".to_string(),
        );
        let new_address = Address::new(
            "456 Oak Ave".to_string(),
            "New City".to_string(),
            "NS".to_string(),
            "Country".to_string(),
            "22222".to_string(),
        );

        let event = LocationUpdated {
            location_id,
            previous_name: Some("Old Name".to_string()),
            name: Some("New Name".to_string()),
            previous_address: Some(old_address),
            address: Some(new_address),
            previous_coordinates: None,
            coordinates: None,
            previous_virtual_location: None,
            virtual_location: None,
            reason: "Office relocation".to_string(),
        };

        assert_eq!(event.location_id(), location_id);
        assert_eq!(event.aggregate_id(), location_id);
        assert_eq!(event.event_type(), "LocationUpdated");
        assert_eq!(event.subject(), format!("location.{location_id}.updated"));
        assert_eq!(event.reason, "Office relocation");
    }

    /// Test ParentLocationSet event
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Child Location] --> B[Set Parent]
    ///     B --> C[Previous Parent]
    ///     C --> D[New Parent]
    /// ```
    #[test]
    fn test_parent_location_set_event() {
        let location_id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();
        let previous_parent_id = Uuid::new_v4();

        let event = ParentLocationSet {
            location_id,
            parent_id,
            previous_parent_id: Some(previous_parent_id),
            reason: "Organizational restructure".to_string(),
        };

        assert_eq!(event.location_id(), location_id);
        assert_eq!(event.aggregate_id(), location_id);
        assert_eq!(event.event_type(), "ParentLocationSet");
        assert_eq!(
            event.subject(),
            format!("location.{location_id}.parent_set")
        );
        assert_eq!(event.parent_id, parent_id);
        assert_eq!(event.previous_parent_id, Some(previous_parent_id));
    }

    /// Test ParentLocationRemoved event
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Child Location] --> B[Remove Parent]
    ///     B --> C[Previous Parent ID]
    ///     C --> D[Now Top-Level]
    /// ```
    #[test]
    fn test_parent_location_removed_event() {
        let location_id = Uuid::new_v4();
        let previous_parent_id = Uuid::new_v4();

        let event = ParentLocationRemoved {
            location_id,
            previous_parent_id,
            reason: "Made independent location".to_string(),
        };

        assert_eq!(event.location_id(), location_id);
        assert_eq!(event.aggregate_id(), location_id);
        assert_eq!(event.event_type(), "ParentLocationRemoved");
        assert_eq!(
            event.subject(),
            format!("location.{}.parent_removed", self.location_id)
        );
        assert_eq!(event.previous_parent_id, previous_parent_id);
    }

    /// Test LocationMetadataAdded event
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Location] --> B[Add Metadata]
    ///     B --> C[New Keys]
    ///     C --> D[Current State]
    /// ```
    #[test]
    fn test_location_metadata_added_event() {
        let location_id = Uuid::new_v4();
        let added_metadata = HashMap::from([
            ("capacity".to_string(), "100".to_string()),
            ("wifi".to_string(), "available".to_string()),
        ]);
        let current_metadata = HashMap::from([
            ("capacity".to_string(), "100".to_string()),
            ("wifi".to_string(), "available".to_string()),
            ("parking".to_string(), "free".to_string()),
        ]);

        let event = LocationMetadataAdded {
            location_id,
            added_metadata: added_metadata.clone(),
            current_metadata: current_metadata.clone(),
            reason: "Added facility information".to_string(),
        };

        assert_eq!(event.location_id(), location_id);
        assert_eq!(event.aggregate_id(), location_id);
        assert_eq!(event.event_type(), "LocationMetadataAdded");
        assert_eq!(
            event.subject(),
            format!("location.{location_id}.metadata_added")
        );
        assert_eq!(event.added_metadata.len(), 2);
        assert_eq!(event.current_metadata.len(), 3);
    }

    /// Test LocationArchived event
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Active Location] --> B[Archive]
    ///     B --> C[Capture Details]
    ///     C --> D[Archive Event]
    /// ```
    #[test]
    fn test_location_archived_event() {
        let location_id = Uuid::new_v4();

        let event = LocationArchived {
            location_id,
            name: "Old Office".to_string(),
            location_type: LocationType::Physical,
            reason: "Office closed permanently".to_string(),
        };

        assert_eq!(event.location_id(), location_id);
        assert_eq!(event.aggregate_id(), location_id);
        assert_eq!(event.event_type(), "LocationArchived");
        assert_eq!(event.subject(), format!("location.{location_id}.archived"));
        assert_eq!(event.name, "Old Office");
        assert_eq!(event.location_type, LocationType::Physical);
    }

    /// Test event serialization round-trip
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Original Event] --> B[Serialize]
    ///     B --> C[JSON]
    ///     C --> D[Deserialize]
    ///     D --> E[Compare]
    /// ```
    #[test]
    fn test_event_serialization_roundtrip() {
        let location_id = Uuid::new_v4();
        let coords = GeoCoordinates::new(40.7128, -74.0060).with_altitude(10.0);

        let event = LocationDefined {
            location_id,
            name: "Test Location".to_string(),
            location_type: LocationType::Physical,
            address: None,
            coordinates: Some(coords.clone()),
            virtual_location: None,
            parent_id: Some(Uuid::new_v4()),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&event).unwrap();

        // Deserialize back
        let deserialized: LocationDefined = serde_json::from_str(&json).unwrap();

        // Verify all fields match
        assert_eq!(deserialized.location_id, event.location_id);
        assert_eq!(deserialized.name, event.name);
        assert_eq!(deserialized.location_type, event.location_type);
        assert_eq!(deserialized.coordinates, Some(coords));
        assert_eq!(deserialized.parent_id, event.parent_id);
    }

    /// Test virtual location event
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Virtual Location] --> B[Platform Details]
    ///     B --> C[Create Event]
    ///     C --> D[Verify Virtual]
    /// ```
    #[test]
    fn test_virtual_location_event() {
        let location_id = Uuid::new_v4();
        let virtual_loc = VirtualLocation::website(
            "https://discord.gg/abc123",
            "Community Voice Channel".to_string(),
        )
        .unwrap();

        let event = LocationDefined {
            location_id,
            name: "Community Voice Channel".to_string(),
            location_type: LocationType::Virtual,
            address: None,
            coordinates: None,
            virtual_location: Some(virtual_loc.clone()),
            parent_id: None,
        };

        assert_eq!(event.location_type, LocationType::Virtual);
        assert!(event.address.is_none());
        assert!(event.coordinates.is_none());
        assert_eq!(event.virtual_location, Some(virtual_loc));
    }
}
