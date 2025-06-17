//! Location query handlers and projections for CQRS read side

use crate::aggregate::{Location, LocationType, Address, GeoCoordinates, VirtualLocation};
use cim_domain::{DomainError, DomainResult, AggregateRoot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Location read model for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReadModel {
    pub id: Uuid,
    pub name: String,
    pub location_type: LocationType,
    pub address: Option<Address>,
    pub coordinates: Option<GeoCoordinates>,
    pub virtual_location: Option<VirtualLocation>,
    pub parent_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
    pub archived: bool,
    pub version: u64,
}

/// Location summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSummary {
    pub id: Uuid,
    pub name: String,
    pub location_type: LocationType,
    pub formatted_address: Option<String>,
    pub parent_name: Option<String>,
    pub archived: bool,
}

/// Hierarchical location structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationHierarchy {
    pub location: LocationSummary,
    pub children: Vec<LocationHierarchy>,
    pub depth: u32,
}

/// Geographical query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationWithDistance {
    pub location: LocationReadModel,
    pub distance_meters: Option<f64>,
}

/// Query for finding locations by various criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindLocationsQuery {
    pub name_pattern: Option<String>,
    pub location_type: Option<LocationType>,
    pub within_distance_of: Option<(GeoCoordinates, f64)>, // coordinates and radius in meters
    pub parent_id: Option<Uuid>,
    pub metadata_filters: HashMap<String, String>,
    pub include_archived: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Query for location hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLocationHierarchyQuery {
    pub root_location_id: Option<Uuid>, // None for full hierarchy
    pub max_depth: Option<u32>,
    pub include_archived: bool,
}

/// Query for locations within geographic bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindLocationsInBoundsQuery {
    pub southwest: GeoCoordinates,
    pub northeast: GeoCoordinates,
    pub location_types: Option<Vec<LocationType>>,
    pub include_archived: bool,
}

/// Location query handler
pub struct LocationQueryHandler {
    /// In production, this would be a read-optimized store
    locations: HashMap<Uuid, LocationReadModel>,
}

impl LocationQueryHandler {
    /// Create new query handler
    pub fn new() -> Self {
        Self {
            locations: HashMap::new(),
        }
    }

    /// Add or update location in read model
    pub fn upsert_location(&mut self, location: &Location) {
        let read_model = LocationReadModel {
            id: *location.id().as_uuid(),
            name: location.name.clone(),
            location_type: location.location_type.clone(),
            address: location.address.clone(),
            coordinates: location.coordinates.clone(),
            virtual_location: location.virtual_location.clone(),
            parent_id: location.parent_id.map(|id| *id.as_uuid()),
            metadata: location.metadata.clone(),
            archived: location.archived,
            version: location.version(),
        };

        self.locations.insert(read_model.id, read_model);
    }

    /// Get location by ID
    pub fn get_location(&self, id: Uuid) -> Option<&LocationReadModel> {
        self.locations.get(&id)
    }

    /// Find locations by query criteria
    pub fn find_locations(&self, query: FindLocationsQuery) -> DomainResult<Vec<LocationReadModel>> {
        let mut results: Vec<_> = self.locations.values()
            .filter(|location| {
                // Filter by archived status
                if !query.include_archived && location.archived {
                    return false;
                }

                // Filter by name pattern
                if let Some(ref pattern) = query.name_pattern {
                    if !location.name.to_lowercase().contains(&pattern.to_lowercase()) {
                        return false;
                    }
                }

                // Filter by location type
                if let Some(ref location_type) = query.location_type {
                    if &location.location_type != location_type {
                        return false;
                    }
                }

                // Filter by parent
                if let Some(parent_id) = query.parent_id {
                    if location.parent_id != Some(parent_id) {
                        return false;
                    }
                }

                // Filter by metadata
                for (key, value) in &query.metadata_filters {
                    if location.metadata.get(key) != Some(value) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Filter by geographic distance
        if let Some((center_coords, radius)) = query.within_distance_of {
            results.retain(|location| {
                if let Some(ref coords) = location.coordinates {
                    coords.distance_to(&center_coords) <= radius
                } else {
                    false
                }
            });
        }

        // Apply pagination
        if let Some(offset) = query.offset {
            if offset < results.len() {
                results = results.into_iter().skip(offset).collect();
            } else {
                results.clear();
            }
        }

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Get location hierarchy
    pub fn get_hierarchy(&self, query: GetLocationHierarchyQuery) -> DomainResult<Vec<LocationHierarchy>> {
        let root_locations = if let Some(root_id) = query.root_location_id {
            vec![self.locations.get(&root_id)
                .ok_or_else(|| DomainError::generic(format!("Location {} not found", root_id)))?
                .clone()]
        } else {
            // Find all top-level locations (no parent)
            self.locations.values()
                .filter(|loc| loc.parent_id.is_none())
                .filter(|loc| query.include_archived || !loc.archived)
                .cloned()
                .collect()
        };

        let mut hierarchies = Vec::new();
        for root_location in root_locations {
            let hierarchy = self.build_hierarchy_recursive(
                &root_location,
                0,
                query.max_depth.unwrap_or(10),
                query.include_archived,
            );
            hierarchies.push(hierarchy);
        }

        Ok(hierarchies)
    }

    /// Find locations within geographic bounds
    pub fn find_in_bounds(&self, query: FindLocationsInBoundsQuery) -> DomainResult<Vec<LocationReadModel>> {
        let results: Vec<_> = self.locations.values()
            .filter(|location| {
                // Filter by archived status
                if !query.include_archived && location.archived {
                    return false;
                }

                // Filter by location type
                if let Some(ref types) = query.location_types {
                    if !types.contains(&location.location_type) {
                        return false;
                    }
                }

                // Filter by geographic bounds
                if let Some(ref coords) = location.coordinates {
                    coords.latitude >= query.southwest.latitude &&
                    coords.latitude <= query.northeast.latitude &&
                    coords.longitude >= query.southwest.longitude &&
                    coords.longitude <= query.northeast.longitude
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(results)
    }

    /// Find nearby locations
    pub fn find_nearby(&self, center: GeoCoordinates, radius_meters: f64) -> DomainResult<Vec<LocationWithDistance>> {
        let mut results: Vec<_> = self.locations.values()
            .filter(|location| !location.archived)
            .filter_map(|location| {
                if let Some(ref coords) = location.coordinates {
                    let distance = coords.distance_to(&center);
                    if distance <= radius_meters {
                        Some(LocationWithDistance {
                            location: location.clone(),
                            distance_meters: Some(distance),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Sort by distance
        results.sort_by(|a, b| {
            a.distance_meters.partial_cmp(&b.distance_meters).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Get location statistics
    pub fn get_statistics(&self) -> LocationStatistics {
        let total = self.locations.len();
        let archived = self.locations.values().filter(|loc| loc.archived).count();
        let active = total - archived;

        let by_type = self.locations.values()
            .filter(|loc| !loc.archived)
            .fold(HashMap::new(), |mut acc, loc| {
                *acc.entry(loc.location_type.clone()).or_insert(0) += 1;
                acc
            });

        let with_coordinates = self.locations.values()
            .filter(|loc| loc.coordinates.is_some())
            .count();

        LocationStatistics {
            total,
            active,
            archived,
            by_type,
            with_coordinates,
        }
    }

    // Helper method to build hierarchy recursively
    fn build_hierarchy_recursive(
        &self,
        location: &LocationReadModel,
        depth: u32,
        max_depth: u32,
        include_archived: bool,
    ) -> LocationHierarchy {
        let summary = LocationSummary {
            id: location.id,
            name: location.name.clone(),
            location_type: location.location_type.clone(),
            formatted_address: location.address.as_ref().map(|a| a.format_single_line()),
            parent_name: None, // Could be populated if needed
            archived: location.archived,
        };

        let children = if depth < max_depth {
            self.locations.values()
                .filter(|child| child.parent_id == Some(location.id))
                .filter(|child| include_archived || !child.archived)
                .map(|child| self.build_hierarchy_recursive(child, depth + 1, max_depth, include_archived))
                .collect()
        } else {
            Vec::new()
        };

        LocationHierarchy {
            location: summary,
            children,
            depth,
        }
    }
}

/// Location statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationStatistics {
    pub total: usize,
    pub active: usize,
    pub archived: usize,
    pub by_type: HashMap<LocationType, usize>,
    pub with_coordinates: usize,
}

impl Default for LocationQueryHandler {
    fn default() -> Self {
        Self::new()
    }
} 