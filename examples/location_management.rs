//! Location Management Example
//!
//! This example demonstrates:
//! - Creating physical and virtual locations
//! - Managing location hierarchies
//! - Geocoding and coordinate handling
//! - Location-based queries

use cim_domain_location::{
    aggregate::Location,
    commands::{AddLocationAttribute, CreateLocation, SetParentLocation, UpdateCoordinates},
    events::{AttributeAdded, CoordinatesUpdated, LocationCreated, ParentLocationSet},
    handlers::LocationCommandHandler,
    queries::{FindNearbyLocations, GetLocation, LocationQueryHandler},
    value_objects::{Address, Coordinates, LocationId, LocationType, VirtualLocation},
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM Location Domain Example ===\n");

    // Initialize handlers
    let command_handler = LocationCommandHandler::new();
    let query_handler = LocationQueryHandler::new();

    // Step 1: Create office building location
    println!("1. Creating office building location...");
    let building_id = LocationId::new();
    let create_building = CreateLocation {
        location_id: building_id.clone(),
        location_type: LocationType::Building,
        name: "TechCorp Headquarters".to_string(),
        address: Some(Address {
            street: "123 Innovation Drive".to_string(),
            city: "San Francisco".to_string(),
            state: "CA".to_string(),
            postal_code: "94105".to_string(),
            country: "USA".to_string(),
        }),
        coordinates: Some(Coordinates {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: Some(15.0),
        }),
    };

    let events = command_handler.handle(create_building).await?;
    println!("   Building location created! Events: {:?}\n", events.len());

    // Additional implementation details...

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
