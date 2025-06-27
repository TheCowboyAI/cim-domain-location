//! Location Domain Queries

use crate::value_objects::{Coordinates, LocationId, LocationType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod find_nearby;
pub mod get_hierarchy;
pub mod get_location;

pub use find_nearby::*;
pub use get_hierarchy::*;
pub use get_location::*;

/// Base trait for location queries
pub trait LocationQuery: Send + Sync {
    type Result;

    fn query_type(&self) -> &'static str;
}

/// Query to get a specific location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLocation {
    pub location_id: LocationId,
    pub include_children: bool,
    pub include_ancestors: bool,
}

/// Query to find locations within a radius
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindNearbyLocations {
    pub center: Coordinates,
    pub radius_km: f64,
    pub location_types: Option<Vec<LocationType>>,
}

/// Query to get location hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLocationHierarchy {
    pub root_location_id: LocationId,
    pub max_depth: Option<u32>,
}

/// Query handler for location queries
pub struct LocationQueryHandler {
    // Read model would be injected here
}

impl LocationQueryHandler {
    pub fn new() -> Self {
        Self {}
    }

    // Query handling methods would be implemented here
}
