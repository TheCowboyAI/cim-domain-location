//! Integration tests for location domain

use cim_domain_location::{
    Location, LocationType, Address, GeoCoordinates,
    DefineLocation, LocationDefined,
};
use cim_core_domain::entity::EntityId;

#[test]
fn test_location_creation() {
    let location_id = EntityId::new();

    let address = Address::new(
        "123 Main St".to_string(),
        "Springfield".to_string(),
        "IL".to_string(),
        "USA".to_string(),
        "62701".to_string(),
    );

    let location = Location::new_physical(
        location_id,
        "Test Office".to_string(),
        address.clone(),
    ).unwrap();

    assert_eq!(location.name, "Test Office");
    assert_eq!(location.location_type, LocationType::Physical);
    assert_eq!(location.address, Some(address));
}

#[test]
fn test_coordinate_distance() {
    // New York City
    let nyc = GeoCoordinates::new(40.7128, -74.0060);

    // Los Angeles
    let la = GeoCoordinates::new(34.0522, -118.2437);

    let distance = nyc.distance_to(&la);

    // Should be approximately 3,944 km
    assert!((distance - 3_944_000.0).abs() < 10_000.0);
}
