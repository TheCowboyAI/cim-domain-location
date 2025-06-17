//! Comprehensive tests for the Location domain

use cim_domain_location::{
    Location, Address, GeoCoordinates, VirtualLocation,
    aggregate::{LocationType, LocationMarker},
    DefineLocation, UpdateLocation, SetParentLocation, RemoveParentLocation,
    AddLocationMetadata, ArchiveLocation,
    LocationDefined, LocationUpdated, ParentLocationSet, ParentLocationRemoved,
    LocationMetadataAdded, LocationArchived,
    LocationQueryHandler, FindLocationsQuery, GetLocationHierarchyQuery,
    FindLocationsInBoundsQuery, LocationReadModel,
};
use cim_domain::{EntityId, CommandHandler, CommandEnvelope};
use std::collections::HashMap;
use uuid::Uuid;

/// Test L1: Define physical location with address
///
/// ```mermaid
/// graph TD
///     A[Start Test] --> B[Create DefineLocation Command]
///     B --> C[Create Physical Address]
///     C --> D[Execute Location Creation]
///     D --> E[Verify Location Properties]
///     E --> F[Test Complete]
/// ```
#[test]
fn test_l1_define_physical_location() {
    let location_id = Uuid::new_v4();

    let address = Address::new(
        "1600 Pennsylvania Avenue NW".to_string(),
        "Washington".to_string(),
        "DC".to_string(),
        "USA".to_string(),
        "20500".to_string(),
    );

    let location = Location::new_physical(
        EntityId::from_uuid(location_id),
        "White House".to_string(),
        address.clone(),
    ).unwrap();

    assert_eq!(location.name, "White House");
    assert_eq!(location.location_type, LocationType::Physical);
    assert_eq!(location.address, Some(address));
    assert!(!location.is_archived());
}

/// Test L2: Define virtual location
///
/// ```mermaid
/// graph TD
///     A[Start Test] --> B[Create VirtualLocation]
///     B --> C[Define Virtual Location Command]
///     C --> D[Execute Location Creation]
///     D --> E[Verify Virtual Properties]
///     E --> F[Test Complete]
/// ```
#[test]
fn test_l2_define_virtual_location() {
    let location_id = Uuid::new_v4();

    let virtual_location = VirtualLocation {
        platform: "Zoom".to_string(),
        platform_id: "123-456-789".to_string(),
        url: Some("https://zoom.us/j/123456789".to_string()),
        platform_data: HashMap::new(),
    };

    let location = Location::new_virtual(
        EntityId::from_uuid(location_id),
        "Daily Standup".to_string(),
        virtual_location.clone(),
    ).unwrap();

    assert_eq!(location.name, "Daily Standup");
    assert_eq!(location.location_type, LocationType::Virtual);
    assert_eq!(location.virtual_location, Some(virtual_location));
}

/// Test L3: Geographic coordinate validation and distance calculation
///
/// ```mermaid
/// graph TD
///     A[Start Test] --> B[Create NYC Coordinates]
///     B --> C[Create LA Coordinates]
///     C --> D[Calculate Distance]
///     D --> E[Verify Distance Accuracy]
///     E --> F[Test Coordinate Validation]
///     F --> G[Test Complete]
/// ```
#[test]
fn test_l3_geographic_calculations() {
    // New York City coordinates
    let nyc = GeoCoordinates::new(40.7128, -74.0060);
    assert!(nyc.validate().is_ok());

    // Los Angeles coordinates
    let la = GeoCoordinates::new(34.0522, -118.2437);
    assert!(la.validate().is_ok());

    // Calculate distance (should be ~3,944 km)
    let distance = nyc.distance_to(&la);
    assert!((distance - 3_944_000.0).abs() < 10_000.0);

    // Test invalid coordinates
    let invalid_lat = GeoCoordinates::new(91.0, -74.0060);
    assert!(invalid_lat.validate().is_err());

    let invalid_lon = GeoCoordinates::new(40.7128, -181.0);
    assert!(invalid_lon.validate().is_err());
}

/// Test L4: Location hierarchy management
///
/// ```mermaid
/// graph TD
///     A[Create Parent Location] --> B[Create Child Location]
///     B --> C[Set Parent Relationship]
///     C --> D[Verify Hierarchy]
///     D --> E[Remove Parent]
///     E --> F[Verify Top-Level]
///     F --> G[Test Complete]
/// ```
#[test]
fn test_l4_location_hierarchy() {
    let parent_id = EntityId::<LocationMarker>::new();
    let child_id = EntityId::<LocationMarker>::new();

    // Create parent location
    let parent_address = Address::new(
        "1 Main St".to_string(),
        "Springfield".to_string(),
        "IL".to_string(),
        "USA".to_string(),
        "62701".to_string(),
    );

    let parent = Location::new_physical(
        parent_id,
        "Main Building".to_string(),
        parent_address,
    ).unwrap();

    // Create child location
    let child_address = Address::new(
        "1 Main St, Suite 100".to_string(),
        "Springfield".to_string(),
        "IL".to_string(),
        "USA".to_string(),
        "62701".to_string(),
    );

    let mut child = Location::new_physical(
        child_id,
        "Suite 100".to_string(),
        child_address,
    ).unwrap();

    // Set parent relationship
    child.set_parent(parent_id).unwrap();
    assert_eq!(child.parent_id, Some(parent_id));

    // Remove parent relationship
    child.remove_parent().unwrap();
    assert_eq!(child.parent_id, None);
}

/// Test L5: Location metadata management
///
/// ```mermaid
/// graph TD
///     A[Create Location] --> B[Add Single Metadata]
///     B --> C[Verify Metadata]
///     C --> D[Add Bulk Metadata]
///     D --> E[Verify All Metadata]
///     E --> F[Test Complete]
/// ```
#[test]
fn test_l5_metadata_management() {
    let location_id = EntityId::<LocationMarker>::new();
    let coordinates = GeoCoordinates::new(37.7749, -122.4194);

    let mut location = Location::new_from_coordinates(
        location_id,
        "Test Location".to_string(),
        coordinates,
    ).unwrap();

    // Add single metadata
    location.add_metadata("building_code".to_string(), "B-123".to_string());
    assert_eq!(location.get_metadata().get("building_code"), Some(&"B-123".to_string()));

    // Add bulk metadata
    let mut bulk_metadata = HashMap::new();
    bulk_metadata.insert("floor".to_string(), "3".to_string());
    bulk_metadata.insert("capacity".to_string(), "50".to_string());

    location.add_metadata_bulk(bulk_metadata);

    assert_eq!(location.get_metadata().get("floor"), Some(&"3".to_string()));
    assert_eq!(location.get_metadata().get("capacity"), Some(&"50".to_string()));
    assert_eq!(location.get_metadata().len(), 3);
}

/// Test L6: Location archiving functionality
///
/// ```mermaid
/// graph TD
///     A[Create Active Location] --> B[Verify Active State]
///     B --> C[Archive Location]
///     C --> D[Verify Archived State]
///     D --> E[Test Update Prevention]
///     E --> F[Test Complete]
/// ```
#[test]
fn test_l6_location_archiving() {
    let location_id = EntityId::<LocationMarker>::new();
    let coordinates = GeoCoordinates::new(40.7128, -74.0060);

    let mut location = Location::new_from_coordinates(
        location_id,
        "Test Location".to_string(),
        coordinates,
    ).unwrap();

    // Initially not archived
    assert!(!location.is_archived());

    // Archive the location
    location.archive().unwrap();
    assert!(location.is_archived());

    // Cannot archive again
    assert!(location.archive().is_err());

    // Cannot update archived location
    let new_coords = GeoCoordinates::new(34.0522, -118.2437);
    assert!(location.update_details(
        Some("New Name".to_string()),
        None,
        Some(new_coords),
        None,
    ).is_err());
}

/// Test L7: Query handler functionality
///
/// ```mermaid
/// graph TD
///     A[Create Query Handler] --> B[Add Test Locations]
///     B --> C[Test Find by Name]
///     C --> D[Test Find by Type]
///     D --> E[Test Geographic Queries]
///     E --> F[Test Hierarchy Queries]
///     F --> G[Test Complete]
/// ```
#[tokio::test]
async fn test_l7_query_handler() {
    let mut query_handler = LocationQueryHandler::new();

    // Create test locations
    let office_id = Uuid::new_v4();
    let office_coords = GeoCoordinates::new(37.7749, -122.4194);
    let office = Location::new_from_coordinates(
        EntityId::from_uuid(office_id),
        "San Francisco Office".to_string(),
        office_coords,
    ).unwrap();

    let warehouse_id = Uuid::new_v4();
    let warehouse_coords = GeoCoordinates::new(37.8044, -122.2712);
    let warehouse = Location::new_from_coordinates(
        EntityId::from_uuid(warehouse_id),
        "Oakland Warehouse".to_string(),
        warehouse_coords,
    ).unwrap();

    // Add to query handler
    query_handler.upsert_location(&office);
    query_handler.upsert_location(&warehouse);

    // Test find by name pattern
    let query = FindLocationsQuery {
        name_pattern: Some("office".to_string()),
        location_type: None,
        within_distance_of: None,
        parent_id: None,
        metadata_filters: HashMap::new(),
        include_archived: false,
        limit: None,
        offset: None,
    };

    let results = query_handler.find_locations(query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "San Francisco Office");

    // Test find by type
    let query = FindLocationsQuery {
        name_pattern: None,
        location_type: Some(LocationType::Physical),
        within_distance_of: None,
        parent_id: None,
        metadata_filters: HashMap::new(),
        include_archived: false,
        limit: None,
        offset: None,
    };

    let results = query_handler.find_locations(query).unwrap();
    assert_eq!(results.len(), 2);

    // Test geographic query (within 10km of SF office)
    let sf_center = GeoCoordinates::new(37.7749, -122.4194);
    let nearby = query_handler.find_nearby(sf_center, 10_000.0).unwrap();
    assert_eq!(nearby.len(), 1); // Only SF office should be within 10km
}

/// Test L8: Location bounds queries
///
/// ```mermaid
/// graph TD
///     A[Create Locations in Different Areas] --> B[Define Geographic Bounds]
///     B --> C[Execute Bounds Query]
///     C --> D[Verify Results]
///     D --> E[Test Edge Cases]
///     E --> F[Test Complete]
/// ```
#[tokio::test]
async fn test_l8_location_bounds_queries() {
    let mut query_handler = LocationQueryHandler::new();

    // San Francisco location
    let sf_id = Uuid::new_v4();
    let sf_coords = GeoCoordinates::new(37.7749, -122.4194);
    let sf_location = Location::new_from_coordinates(
        EntityId::from_uuid(sf_id),
        "SF Location".to_string(),
        sf_coords,
    ).unwrap();

    // New York location
    let ny_id = Uuid::new_v4();
    let ny_coords = GeoCoordinates::new(40.7128, -74.0060);
    let ny_location = Location::new_from_coordinates(
        EntityId::from_uuid(ny_id),
        "NY Location".to_string(),
        ny_coords,
    ).unwrap();

    query_handler.upsert_location(&sf_location);
    query_handler.upsert_location(&ny_location);

    // Query for California bounds (should include SF, not NY)
    let query = FindLocationsInBoundsQuery {
        southwest: GeoCoordinates::new(32.0, -125.0), // Southern California
        northeast: GeoCoordinates::new(42.0, -114.0), // Northern California/Nevada
        location_types: None,
        include_archived: false,
    };

    let results = query_handler.find_in_bounds(query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "SF Location");
}

/// Test L9: Location statistics
///
/// ```mermaid
/// graph TD
///     A[Create Mixed Locations] --> B[Archive Some Locations]
///     B --> C[Generate Statistics]
///     C --> D[Verify Counts]
///     D --> E[Verify Type Distribution]
///     E --> F[Test Complete]
/// ```
#[tokio::test]
async fn test_l9_location_statistics() {
    let mut query_handler = LocationQueryHandler::new();

    // Create various types of locations
    let physical_id = Uuid::new_v4();
    let physical_coords = GeoCoordinates::new(37.7749, -122.4194);
    let physical_location = Location::new_from_coordinates(
        EntityId::from_uuid(physical_id),
        "Physical Location".to_string(),
        physical_coords,
    ).unwrap();

    let virtual_id = Uuid::new_v4();
    let virtual_location = Location::new_virtual(
        EntityId::from_uuid(virtual_id),
        "Virtual Meeting Room".to_string(),
        VirtualLocation {
            platform: "Zoom".to_string(),
            platform_id: "123".to_string(),
            url: None,
            platform_data: HashMap::new(),
        },
    ).unwrap();

    // Create archived location
    let archived_id = Uuid::new_v4();
    let archived_coords = GeoCoordinates::new(40.7128, -74.0060);
    let mut archived_location = Location::new_from_coordinates(
        EntityId::from_uuid(archived_id),
        "Archived Location".to_string(),
        archived_coords,
    ).unwrap();
    archived_location.archive().unwrap();

    query_handler.upsert_location(&physical_location);
    query_handler.upsert_location(&virtual_location);
    query_handler.upsert_location(&archived_location);

    // Get statistics
    let stats = query_handler.get_statistics();

    assert_eq!(stats.total, 3);
    assert_eq!(stats.active, 2);
    assert_eq!(stats.archived, 1);
    assert_eq!(stats.with_coordinates, 2); // Physical and archived have coordinates
    assert_eq!(*stats.by_type.get(&LocationType::Physical).unwrap_or(&0), 1);
    assert_eq!(*stats.by_type.get(&LocationType::Virtual).unwrap_or(&0), 1);
}

/// Test L10: Complex location hierarchy
///
/// ```mermaid
/// graph TD
///     A[Create Root Location] --> B[Create Building Locations]
///     B --> C[Create Floor Locations]
///     C --> D[Create Room Locations]
///     D --> E[Query Full Hierarchy]
///     E --> F[Verify Tree Structure]
///     F --> G[Test Complete]
/// ```
#[tokio::test]
async fn test_l10_complex_hierarchy() {
    let mut query_handler = LocationQueryHandler::new();

    // Create campus (root)
    let campus_id = Uuid::new_v4();
    let campus_coords = GeoCoordinates::new(37.7749, -122.4194);
    let campus = Location::new_from_coordinates(
        EntityId::from_uuid(campus_id),
        "Tech Campus".to_string(),
        campus_coords,
    ).unwrap();

    // Create building (child of campus)
    let building_id = Uuid::new_v4();
    let building_coords = GeoCoordinates::new(37.7750, -122.4195);
    let mut building = Location::new_from_coordinates(
        EntityId::from_uuid(building_id),
        "Building A".to_string(),
        building_coords,
    ).unwrap();
    building.set_parent(EntityId::from_uuid(campus_id)).unwrap();

    // Create floor (child of building)
    let floor_id = Uuid::new_v4();
    let floor_coords = GeoCoordinates::new(37.7750, -122.4195);
    let mut floor = Location::new_from_coordinates(
        EntityId::from_uuid(floor_id),
        "Floor 3".to_string(),
        floor_coords,
    ).unwrap();
    floor.set_parent(EntityId::from_uuid(building_id)).unwrap();

    query_handler.upsert_location(&campus);
    query_handler.upsert_location(&building);
    query_handler.upsert_location(&floor);

    // Query hierarchy starting from campus
    let query = GetLocationHierarchyQuery {
        root_location_id: Some(campus_id),
        max_depth: Some(3),
        include_archived: false,
    };

    let hierarchy = query_handler.get_hierarchy(query).unwrap();

    assert_eq!(hierarchy.len(), 1); // One root
    assert_eq!(hierarchy[0].location.name, "Tech Campus");
    assert_eq!(hierarchy[0].children.len(), 1); // One building
    assert_eq!(hierarchy[0].children[0].location.name, "Building A");
    assert_eq!(hierarchy[0].children[0].children.len(), 1); // One floor
    assert_eq!(hierarchy[0].children[0].children[0].location.name, "Floor 3");
}
