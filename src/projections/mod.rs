//! Location Domain Projections

use crate::events::*;
use crate::value_objects::{GeoCoordinates, LocationType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Base trait for location projections
pub trait LocationProjection: Send + Sync {
    fn handle_location_defined(&mut self, event: &LocationDefined);
    fn handle_location_updated(&mut self, event: &LocationUpdated);
    fn handle_parent_location_set(&mut self, event: &ParentLocationSet);
    fn handle_parent_location_removed(&mut self, event: &ParentLocationRemoved);
    fn handle_location_metadata_added(&mut self, event: &LocationMetadataAdded);
    fn handle_location_archived(&mut self, event: &LocationArchived);
    fn projection_name(&self) -> &'static str;
}

/// Read model for location queries
#[derive(Debug, Clone, Default)]
pub struct LocationReadModel {
    pub locations: HashMap<Uuid, LocationView>,
    pub hierarchy: LocationHierarchy,
    pub spatial_index: SpatialIndex,
}

/// View of a single location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationView {
    pub id: Uuid,
    pub name: String,
    pub location_type: LocationType,
    pub coordinates: Option<GeoCoordinates>,
    pub parent_id: Option<Uuid>,
    pub children_ids: Vec<Uuid>,
    pub attributes: HashMap<String, String>,
}

/// Hierarchical view of locations
#[derive(Debug, Clone, Default)]
pub struct LocationHierarchy {
    pub roots: Vec<Uuid>,
    pub parent_child_map: HashMap<Uuid, Vec<Uuid>>,
    pub child_parent_map: HashMap<Uuid, Uuid>,
}

/// Spatial index for proximity queries
#[derive(Debug, Clone, Default)]
pub struct SpatialIndex {
    // In a real implementation, this would use an R-tree or similar
    pub locations_by_coordinates: Vec<(Uuid, GeoCoordinates)>,
}

impl LocationProjection for LocationReadModel {
    fn handle_location_defined(&mut self, event: &LocationDefined) {
        let view = LocationView {
            id: event.location_id,
            name: event.name.clone(),
            location_type: event.location_type.clone(),
            coordinates: event.coordinates.clone(),
            parent_id: event.parent_id,
            children_ids: Vec::new(),
            attributes: HashMap::new(),
        };

        self.locations.insert(event.location_id, view);

        if let Some(coords) = &event.coordinates {
            self.spatial_index
                .locations_by_coordinates
                .push((event.location_id, coords.clone()));
        }
    }

    fn handle_location_updated(&mut self, event: &LocationUpdated) {
        if let Some(location) = self.locations.get_mut(&event.location_id) {
            if let Some(name) = &event.name {
                location.name = name.clone();
            }
            if event.coordinates.is_some() {
                location.coordinates = event.coordinates.clone();
            }
        }
    }

    fn handle_parent_location_set(&mut self, event: &ParentLocationSet) {
        // Update hierarchy
        if let Some(location) = self.locations.get_mut(&event.location_id) {
            location.parent_id = Some(event.parent_id);
        }

        self.hierarchy
            .child_parent_map
            .insert(event.location_id, event.parent_id);
        self.hierarchy
            .parent_child_map
            .entry(event.parent_id)
            .or_insert_with(Vec::new)
            .push(event.location_id);
    }

    fn handle_parent_location_removed(&mut self, event: &ParentLocationRemoved) {
        if let Some(location) = self.locations.get_mut(&event.location_id) {
            location.parent_id = None;
        }

        self.hierarchy.child_parent_map.remove(&event.location_id);
        if let Some(children) = self
            .hierarchy
            .parent_child_map
            .get_mut(&event.previous_parent_id)
        {
            children.retain(|id| *id != event.location_id);
        }
    }

    fn handle_location_metadata_added(&mut self, event: &LocationMetadataAdded) {
        if let Some(location) = self.locations.get_mut(&event.location_id) {
            location.attributes = event.current_metadata.clone();
        }
    }

    fn handle_location_archived(&mut self, _event: &LocationArchived) {
        // Could mark as archived in the view or remove from active locations
    }

    fn projection_name(&self) -> &'static str {
        "LocationReadModel"
    }
}
