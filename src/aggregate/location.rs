//! Location aggregate and related value objects
//!
//! Location is an aggregate that can represent any identifiable place through
//! various means: addresses, geo-coordinates, virtual locations, etc.

use cim_domain::{AggregateRoot, Entity, EntityId, DomainError, DomainResult};
use std::collections::HashMap;
use crate::value_objects::{
    LocationType, Address, GeoCoordinates, 
    VirtualLocation as EnhancedVirtualLocation
};

/// Location aggregate - represents any identifiable place
#[derive(Debug, Clone)]
pub struct Location {
    /// Core entity data
    entity: Entity<LocationMarker>,

    /// Version for optimistic concurrency control
    version: u64,

    /// Human-readable name for the location
    pub name: String,

    /// Type of location (physical, virtual, logical, etc.)
    pub location_type: LocationType,

    /// Physical address if applicable
    pub address: Option<Address>,

    /// Geographic coordinates if applicable
    pub coordinates: Option<GeoCoordinates>,

    /// Virtual location details if applicable
    pub virtual_location: Option<EnhancedVirtualLocation>,

    /// Parent location for hierarchical structures
    pub parent_id: Option<EntityId<LocationMarker>>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Whether this location is archived (soft deleted)
    pub archived: bool,
}

/// Marker type for Location entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocationMarker;

impl Location {
    /// Create a new physical location with an address
    pub fn new_physical(
        id: EntityId<LocationMarker>,
        name: String,
        address: Address,
    ) -> DomainResult<Self> {
        address.validate()?;

        Ok(Self {
            entity: Entity::with_id(id),
            version: 0,
            name,
            location_type: LocationType::Physical,
            address: Some(address),
            coordinates: None,
            virtual_location: None,
            parent_id: None,
            metadata: HashMap::new(),
            archived: false,
        })
    }

    /// Create a new virtual location
    pub fn new_virtual(
        id: EntityId<LocationMarker>,
        name: String,
        virtual_location: EnhancedVirtualLocation,
    ) -> DomainResult<Self> {
        Ok(Self {
            entity: Entity::with_id(id),
            version: 0,
            name,
            location_type: LocationType::Virtual,
            address: None,
            coordinates: None,
            virtual_location: Some(virtual_location),
            parent_id: None,
            metadata: HashMap::new(),
            archived: false,
        })
    }

    /// Create a new location with just coordinates
    pub fn new_from_coordinates(
        id: EntityId<LocationMarker>,
        name: String,
        coordinates: GeoCoordinates,
    ) -> DomainResult<Self> {
        coordinates.validate()?;

        Ok(Self {
            entity: Entity::with_id(id),
            version: 0,
            name,
            location_type: LocationType::Physical,
            address: None,
            coordinates: Some(coordinates),
            virtual_location: None,
            parent_id: None,
            metadata: HashMap::new(),
            archived: false,
        })
    }

    /// Set the address for this location
    pub fn set_address(&mut self, address: Address) -> DomainResult<()> {
        address.validate()?;

        if self.location_type == LocationType::Virtual {
            return Err(DomainError::ValidationError(
                "Cannot set physical address on virtual location".to_string()
            ));
        }

        self.address = Some(address);
        self.entity.touch();
        Ok(())
    }

    /// Set geographic coordinates
    pub fn set_coordinates(&mut self, coordinates: GeoCoordinates) -> DomainResult<()> {
        coordinates.validate()?;

        if self.location_type == LocationType::Virtual {
            return Err(DomainError::ValidationError(
                "Cannot set coordinates on virtual location".to_string()
            ));
        }

        self.coordinates = Some(coordinates);
        self.entity.touch();
        Ok(())
    }

    /// Set parent location for hierarchical structures
    pub fn set_parent(&mut self, parent_id: EntityId<LocationMarker>) -> DomainResult<()> {
        // Prevent self-reference
        if parent_id == self.entity.id {
            return Err(DomainError::ValidationError(
                "Location cannot be its own parent".to_string()
            ));
        }

        self.parent_id = Some(parent_id);
        self.entity.touch();
        Ok(())
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.entity.touch();
    }

    /// Update location details
    pub fn update_details(
        &mut self,
        name: Option<String>,
        address: Option<Address>,
        coordinates: Option<GeoCoordinates>,
        virtual_location: Option<EnhancedVirtualLocation>,
    ) -> DomainResult<()> {
        if self.archived {
            return Err(DomainError::ValidationError(
                "Cannot update archived location".to_string()
            ));
        }

        // Validate new address if provided
        if let Some(ref addr) = address {
            addr.validate()?;
        }

        // Validate new coordinates if provided
        if let Some(ref coords) = coordinates {
            coords.validate()?;
        }

        // Apply updates
        if let Some(new_name) = name {
            self.name = new_name;
        }

        if let Some(new_address) = address {
            self.address = Some(new_address);
        }

        if let Some(new_coordinates) = coordinates {
            self.coordinates = Some(new_coordinates);
        }

        if let Some(new_virtual_location) = virtual_location {
            self.virtual_location = Some(new_virtual_location);
        }

        self.entity.touch();
        Ok(())
    }

    /// Add multiple metadata entries
    pub fn add_metadata_bulk(&mut self, metadata: HashMap<String, String>) {
        for (key, value) in metadata {
            self.metadata.insert(key, value);
        }
        self.entity.touch();
    }

    /// Remove parent (make top-level)
    pub fn remove_parent(&mut self) -> DomainResult<()> {
        if self.archived {
            return Err(DomainError::ValidationError(
                "Cannot modify archived location".to_string()
            ));
        }

        self.parent_id = None;
        self.entity.touch();
        Ok(())
    }

    /// Archive this location (soft delete)
    pub fn archive(&mut self) -> DomainResult<()> {
        if self.archived {
            return Err(DomainError::ValidationError(
                "Location is already archived".to_string()
            ));
        }

        self.archived = true;
        self.entity.touch();
        Ok(())
    }

    /// Check if location is archived
    pub fn is_archived(&self) -> bool {
        self.archived
    }

    /// Get current metadata snapshot
    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

impl AggregateRoot for Location {
    type Id = EntityId<LocationMarker>;

    fn id(&self) -> Self::Id {
        self.entity.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
        self.entity.touch();
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{VirtualLocation as EnhancedVirtualLocation, VirtualLocationType, VirtualUrl, UrlType};

    /// Test address validation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Address] --> B{Validate}
    ///     B -->|Valid| C[Success]
    ///     B -->|Empty Fields| D[Error]
    /// ```
    #[test]
    fn test_address_validation() {
        // Valid address
        let valid_address = Address::new(
            "123 Main St".to_string(),
            "Springfield".to_string(),
            "IL".to_string(),
            "USA".to_string(),
            "62701".to_string(),
        );
        assert!(valid_address.validate().is_ok());

        // Invalid address - empty street
        let invalid_address = Address::new(
            "".to_string(),
            "Springfield".to_string(),
            "IL".to_string(),
            "USA".to_string(),
            "62701".to_string(),
        );
        assert!(invalid_address.validate().is_err());

        // Invalid address - empty locality
        let invalid_locality = Address::new(
            "123 Main St".to_string(),
            "".to_string(),
            "IL".to_string(),
            "USA".to_string(),
            "62701".to_string(),
        );
        assert!(invalid_locality.validate().is_err());

        // Invalid address - empty region
        let invalid_region = Address::new(
            "123 Main St".to_string(),
            "Springfield".to_string(),
            "".to_string(),
            "USA".to_string(),
            "62701".to_string(),
        );
        assert!(invalid_region.validate().is_err());

        // Invalid address - empty country
        let invalid_country = Address::new(
            "123 Main St".to_string(),
            "Springfield".to_string(),
            "IL".to_string(),
            "".to_string(),
            "62701".to_string(),
        );
        assert!(invalid_country.validate().is_err());

        // Invalid address - empty postal code
        let invalid_postal = Address::new(
            "123 Main St".to_string(),
            "Springfield".to_string(),
            "IL".to_string(),
            "USA".to_string(),
            "".to_string(),
        );
        assert!(invalid_postal.validate().is_err());
    }

    /// Test address with street2
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Base Address] --> B[Add Street2]
    ///     B --> C[Format Output]
    ///     C --> D[Verify Format]
    /// ```
    #[test]
    fn test_address_with_street2() {
        let address = Address::new(
            "123 Main St".to_string(),
            "Springfield".to_string(),
            "IL".to_string(),
            "USA".to_string(),
            "62701".to_string(),
        ).with_street2("Apt 4B".to_string());

        assert_eq!(address.street2, Some("Apt 4B".to_string()));

        let formatted = address.format_single_line();
        assert!(formatted.contains("123 Main St"));
        assert!(formatted.contains("Apt 4B"));
        assert!(formatted.contains("Springfield, IL 62701"));
        assert!(formatted.contains("USA"));
    }

    /// Test geo coordinates validation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Coords] --> B{Validate Range}
    ///     B -->|Valid| C[Success]
    ///     B -->|Out of Range| D[Error]
    /// ```
    #[test]
    fn test_geo_coordinates_validation() {
        // Valid coordinates
        let valid_coords = GeoCoordinates::new(40.7128, -74.0060);
        assert!(valid_coords.validate().is_ok());

        // Edge cases - maximum valid values
        let max_lat = GeoCoordinates::new(90.0, 0.0);
        assert!(max_lat.validate().is_ok());

        let min_lat = GeoCoordinates::new(-90.0, 0.0);
        assert!(min_lat.validate().is_ok());

        let max_lon = GeoCoordinates::new(0.0, 180.0);
        assert!(max_lon.validate().is_ok());

        let min_lon = GeoCoordinates::new(0.0, -180.0);
        assert!(min_lon.validate().is_ok());

        // Invalid latitude
        let invalid_lat = GeoCoordinates::new(91.0, -74.0060);
        assert!(invalid_lat.validate().is_err());

        let invalid_lat_neg = GeoCoordinates::new(-91.0, -74.0060);
        assert!(invalid_lat_neg.validate().is_err());

        // Invalid longitude
        let invalid_lon = GeoCoordinates::new(40.7128, -181.0);
        assert!(invalid_lon.validate().is_err());

        let invalid_lon_pos = GeoCoordinates::new(40.7128, 181.0);
        assert!(invalid_lon_pos.validate().is_err());
    }

    /// Test coordinates with altitude
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Base Coords] --> B[Add Altitude]
    ///     B --> C[Add Coord System]
    ///     C --> D[Verify Fields]
    /// ```
    #[test]
    fn test_coordinates_with_altitude() {
        let coords = GeoCoordinates::new(40.7128, -74.0060)
            .with_altitude(100.5)
            .with_coordinate_system("NAD83".to_string());

        assert_eq!(coords.altitude, Some(100.5));
        assert_eq!(coords.coordinate_system, "NAD83");
    }

    /// Test physical location creation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Physical] --> B[Validate Address]
    ///     B --> C[Set Fields]
    ///     C --> D[Verify State]
    /// ```
    #[test]
    fn test_location_creation() {
        let location_id = EntityId::<LocationMarker>::new();

        let address = Address::new(
            "1 Infinite Loop".to_string(),
            "Cupertino".to_string(),
            "CA".to_string(),
            "USA".to_string(),
            "95014".to_string(),
        );

        let location = Location::new_physical(
            location_id,
            "Apple Park".to_string(),
            address.clone(),
        ).unwrap();

        assert_eq!(location.name, "Apple Park");
        assert_eq!(location.location_type, LocationType::Physical);
        assert_eq!(location.address, Some(address));
        assert!(location.coordinates.is_none());
        assert!(location.virtual_location.is_none());
        assert!(!location.archived);
        assert_eq!(location.version, 0);
    }

    /// Test virtual location creation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Virtual] --> B[Set Platform]
    ///     B --> C[Add URL]
    ///     C --> D[Verify Type]
    /// ```
    #[test]
    fn test_virtual_location_creation() {
        let location_id = EntityId::<LocationMarker>::new();

        let virtual_loc = EnhancedVirtualLocation {
            location_type: VirtualLocationType::MeetingRoom { platform: "Zoom".to_string() },
            primary_identifier: "meeting-123".to_string(),
            urls: vec![VirtualUrl::new("https://zoom.us/j/123".to_string(), UrlType::Primary).unwrap()],
            ip_addresses: Vec::new(),
            network_info: None,
            metadata: HashMap::from([
                ("passcode".to_string(), "abc123".to_string()),
            ]),
        };

        let location = Location::new_virtual(
            location_id,
            "Team Standup Room".to_string(),
            virtual_loc.clone(),
        ).unwrap();

        assert_eq!(location.name, "Team Standup Room");
        assert_eq!(location.location_type, LocationType::Virtual);
        assert!(location.address.is_none());
        assert!(location.coordinates.is_none());
        assert_eq!(location.virtual_location, Some(virtual_loc));
    }

    /// Test location from coordinates
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create from Coords] --> B[Validate Coords]
    ///     B --> C[Set Type Physical]
    ///     C --> D[No Address]
    /// ```
    #[test]
    fn test_location_from_coordinates() {
        let location_id = EntityId::<LocationMarker>::new();
        let coords = GeoCoordinates::new(37.7749, -122.4194);

        let location = Location::new_from_coordinates(
            location_id,
            "Golden Gate Bridge".to_string(),
            coords.clone(),
        ).unwrap();

        assert_eq!(location.name, "Golden Gate Bridge");
        assert_eq!(location.location_type, LocationType::Physical);
        assert!(location.address.is_none());
        assert_eq!(location.coordinates, Some(coords));
    }

    /// Test location updates
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Location] --> B[Update Details]
    ///     B --> C[Verify Changes]
    ///     C --> D[Check Version]
    /// ```
    #[test]
    fn test_location_updates() {
        let location_id = EntityId::<LocationMarker>::new();
        let mut location = Location::new_from_coordinates(
            location_id,
            "Test Location".to_string(),
            GeoCoordinates::new(0.0, 0.0),
        ).unwrap();

        // Update name
        location.update_details(
            Some("Updated Location".to_string()),
            None,
            None,
            None,
        ).unwrap();

        assert_eq!(location.name, "Updated Location");

        // Add address
        let address = Address::new(
            "456 Oak Ave".to_string(),
            "Oakland".to_string(),
            "CA".to_string(),
            "USA".to_string(),
            "94612".to_string(),
        );

        location.update_details(
            None,
            Some(address.clone()),
            None,
            None,
        ).unwrap();

        assert_eq!(location.address, Some(address));
    }

    /// Test parent-child relationships
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Parent Location] --> B[Child Location]
    ///     B --> C[Set Parent]
    ///     C --> D{Self-Reference?}
    ///     D -->|No| E[Success]
    ///     D -->|Yes| F[Error]
    /// ```
    #[test]
    fn test_location_hierarchy() {
        let parent_id = EntityId::<LocationMarker>::new();
        let child_id = EntityId::<LocationMarker>::new();

        let mut child_location = Location::new_physical(
            child_id,
            "Conference Room A".to_string(),
            Address::new(
                "123 Main St".to_string(),
                "City".to_string(),
                "State".to_string(),
                "Country".to_string(),
                "12345".to_string(),
            ),
        ).unwrap();

        // Set parent
        child_location.set_parent(parent_id).unwrap();
        assert_eq!(child_location.parent_id, Some(parent_id));

        // Remove parent
        child_location.remove_parent().unwrap();
        assert_eq!(child_location.parent_id, None);

        // Try self-reference
        let result = child_location.set_parent(child_id);
        assert!(result.is_err());
    }

    /// Test metadata operations
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Location] --> B[Add Metadata]
    ///     B --> C[Add Bulk]
    ///     C --> D[Verify All]
    /// ```
    #[test]
    fn test_metadata_operations() {
        let location_id = EntityId::<LocationMarker>::new();
        let mut location = Location::new_physical(
            location_id,
            "Office".to_string(),
            Address::new(
                "789 Tech Blvd".to_string(),
                "Tech City".to_string(),
                "TC".to_string(),
                "Techland".to_string(),
                "00000".to_string(),
            ),
        ).unwrap();

        // Add single metadata
        location.add_metadata("capacity".to_string(), "50".to_string());
        assert_eq!(location.metadata.get("capacity"), Some(&"50".to_string()));

        // Add bulk metadata
        let bulk_metadata = HashMap::from([
            ("wifi".to_string(), "available".to_string()),
            ("parking".to_string(), "free".to_string()),
            ("accessibility".to_string(), "wheelchair".to_string()),
        ]);

        location.add_metadata_bulk(bulk_metadata);

        assert_eq!(location.metadata.len(), 4);
        assert_eq!(location.metadata.get("wifi"), Some(&"available".to_string()));
        assert_eq!(location.metadata.get("parking"), Some(&"free".to_string()));
        assert_eq!(location.metadata.get("accessibility"), Some(&"wheelchair".to_string()));
    }

    /// Test location archival
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Active Location] --> B[Archive]
    ///     B --> C{Already Archived?}
    ///     C -->|No| D[Success]
    ///     C -->|Yes| E[Error]
    ///     D --> F[Cannot Update]
    /// ```
    #[test]
    fn test_location_archival() {
        let location_id = EntityId::<LocationMarker>::new();
        let mut location = Location::new_physical(
            location_id,
            "Old Office".to_string(),
            Address::new(
                "999 Legacy Lane".to_string(),
                "History Town".to_string(),
                "HT".to_string(),
                "Pastland".to_string(),
                "99999".to_string(),
            ),
        ).unwrap();

        // Archive location
        assert!(!location.is_archived());
        location.archive().unwrap();
        assert!(location.is_archived());

        // Try to archive again
        let result = location.archive();
        assert!(result.is_err());

        // Try to update archived location
        let result = location.update_details(
            Some("New Name".to_string()),
            None,
            None,
            None,
        );
        assert!(result.is_err());

        // Try to remove parent on archived location
        let result = location.remove_parent();
        assert!(result.is_err());
    }

    /// Test distance calculation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[NYC Coords] --> B[LA Coords]
    ///     B --> C[Calculate Distance]
    ///     C --> D[Verify ~3944km]
    /// ```
    #[test]
    fn test_distance_calculation() {
        // New York City
        let nyc = GeoCoordinates::new(40.7128, -74.0060);

        // Los Angeles
        let la = GeoCoordinates::new(34.0522, -118.2437);

        let distance = nyc.distance_to(&la);

        // Should be approximately 3,944 km
        assert!((distance - 3_944_000.0).abs() < 10_000.0);

        // Test same location
        let same_distance = nyc.distance_to(&nyc);
        assert!(same_distance < 1.0); // Should be ~0
    }

    /// Test virtual location constraints
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Virtual Location] --> B{Set Address?}
    ///     B --> C[Error]
    ///     A --> D{Set Coords?}
    ///     D --> E[Error]
    /// ```
    #[test]
    fn test_virtual_location_constraints() {
        let location_id = EntityId::<LocationMarker>::new();
        let mut location = Location::new_virtual(
            location_id,
            "Virtual Meeting".to_string(),
            EnhancedVirtualLocation {
                location_type: VirtualLocationType::MeetingRoom { platform: "Teams".to_string() },
                primary_identifier: "meeting-456".to_string(),
                urls: Vec::new(),
                ip_addresses: Vec::new(),
                network_info: None,
                metadata: HashMap::new(),
            },
        ).unwrap();

        // Cannot set address on virtual location
        let address = Address::new(
            "123 Main St".to_string(),
            "City".to_string(),
            "State".to_string(),
            "Country".to_string(),
            "12345".to_string(),
        );
        let result = location.set_address(address);
        assert!(result.is_err());

        // Cannot set coordinates on virtual location
        let coords = GeoCoordinates::new(0.0, 0.0);
        let result = location.set_coordinates(coords);
        assert!(result.is_err());
    }

    /// Test aggregate root implementation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Location] --> B[Check ID]
    ///     B --> C[Check Version]
    ///     C --> D[Increment Version]
    ///     D --> E[Verify Change]
    /// ```
    #[test]
    fn test_aggregate_root_implementation() {
        let location_id = EntityId::<LocationMarker>::new();
        let mut location = Location::new_physical(
            location_id,
            "Test".to_string(),
            Address::new(
                "1 Test St".to_string(),
                "Test City".to_string(),
                "TS".to_string(),
                "Testland".to_string(),
                "00000".to_string(),
            ),
        ).unwrap();

        // Check ID
        assert_eq!(location.id(), location_id);

        // Check initial version
        assert_eq!(location.version(), 0);

        // Increment version
        location.increment_version();
        assert_eq!(location.version(), 1);

        // Another increment
        location.increment_version();
        assert_eq!(location.version(), 2);
    }
}
