//! Location Domain Projections

use crate::events::*;
use crate::value_objects::{Coordinates, LocationId, LocationType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod hierarchy_view;
pub mod location_view;
pub mod spatial_index;

pub use hierarchy_view::*;
pub use location_view::*;
pub use spatial_index::*;

/// Base trait for location projections
pub trait LocationProjection: Send + Sync {
    fn handle_event(&mut self, event: &LocationEvent);
    fn projection_name(&self) -> &'static str;
}

/// Read model for location queries
#[derive(Debug, Clone, Default)]
pub struct LocationReadModel {
    pub locations: HashMap<LocationId, LocationView>,
    pub hierarchy: LocationHierarchy,
    pub spatial_index: SpatialIndex,
}

/// View of a single location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationView {
    pub id: LocationId,
    pub name: String,
    pub location_type: LocationType,
    pub coordinates: Option<Coordinates>,
    pub parent_id: Option<LocationId>,
    pub children_ids: Vec<LocationId>,
    pub attributes: HashMap<String, String>,
}

/// Hierarchical view of locations
#[derive(Debug, Clone, Default)]
pub struct LocationHierarchy {
    pub roots: Vec<LocationId>,
    pub parent_child_map: HashMap<LocationId, Vec<LocationId>>,
    pub child_parent_map: HashMap<LocationId, LocationId>,
}

/// Spatial index for proximity queries
#[derive(Debug, Clone, Default)]
pub struct SpatialIndex {
    // In a real implementation, this would use an R-tree or similar
    pub locations_by_coordinates: Vec<(LocationId, Coordinates)>,
}

impl LocationProjection for LocationReadModel {
    fn handle_event(&mut self, event: &LocationEvent) {
        match event {
            LocationEvent::LocationCreated {
                location_id,
                name,
                location_type,
                coordinates,
                ..
            } => {
                let view = LocationView {
                    id: location_id.clone(),
                    name: name.clone(),
                    location_type: location_type.clone(),
                    coordinates: coordinates.clone(),
                    parent_id: None,
                    children_ids: Vec::new(),
                    attributes: HashMap::new(),
                };

                self.locations.insert(location_id.clone(), view);

                if let Some(coords) = coordinates {
                    self.spatial_index
                        .locations_by_coordinates
                        .push((location_id.clone(), coords.clone()));
                }
            }
            LocationEvent::ParentLocationSet {
                location_id,
                parent_id,
                ..
            } => {
                // Update hierarchy
                if let Some(location) = self.locations.get_mut(location_id) {
                    location.parent_id = Some(parent_id.clone());
                }

                self.hierarchy
                    .child_parent_map
                    .insert(location_id.clone(), parent_id.clone());
                self.hierarchy
                    .parent_child_map
                    .entry(parent_id.clone())
                    .or_insert_with(Vec::new)
                    .push(location_id.clone());
            }
            // Handle other events...
            _ => {}
        }
    }

    fn projection_name(&self) -> &'static str {
        "LocationReadModel"
    }
}
