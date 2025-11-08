//! Spatial search services for location-based queries

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::value_objects::{Coordinates, LocationTypes};
use thiserror::Error;

/// Spatial search service trait for location-based queries
#[async_trait]
pub trait SpatialSearchService: Send + Sync {
    /// Find locations within a radius of a point
    async fn find_within_radius(
        &self,
        center: &Coordinates,
        radius_meters: f64,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialSearchResult, SpatialSearchError>;
    
    /// Find locations within a bounding box
    async fn find_within_bounds(
        &self,
        southwest: &Coordinates,
        northeast: &Coordinates,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialSearchResult, SpatialSearchError>;
    
    /// Find locations along a route/path
    async fn find_along_route(
        &self,
        route_points: &[Coordinates],
        corridor_width_meters: f64,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialSearchResult, SpatialSearchError>;
    
    /// Find nearest locations to a point
    async fn find_nearest(
        &self,
        point: &Coordinates,
        max_results: u32,
        max_distance_meters: Option<f64>,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialSearchResult, SpatialSearchError>;
    
    /// Get spatial statistics for a region
    async fn get_spatial_statistics(
        &self,
        region: &SpatialRegion,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialStatistics, SpatialSearchError>;
}

/// Spatial search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialSearchFilters {
    /// Filter by location types
    pub location_types: Option<Vec<LocationTypes>>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Filter by categories
    pub categories: Option<Vec<String>>,
    /// Filter by owner/user
    pub owner_id: Option<Uuid>,
    /// Filter by creation date range
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
    /// Filter by activity level
    pub min_activity_score: Option<f64>,
    /// Filter by verification status
    pub verified_only: Option<bool>,
    /// Custom metadata filters
    pub metadata_filters: Option<serde_json::Value>,
}

/// Spatial search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialSearchResult {
    pub request_id: Uuid,
    pub query: SpatialQuery,
    pub locations: Vec<SpatialLocationMatch>,
    pub total_count: u64,
    pub search_time_ms: u64,
    pub has_more_results: bool,
    pub next_page_token: Option<String>,
    pub search_metadata: SpatialSearchMetadata,
}

/// Spatial search query information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialQuery {
    pub query_type: SpatialQueryType,
    pub parameters: serde_json::Value,
    pub filters: Option<SpatialSearchFilters>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of spatial queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpatialQueryType {
    WithinRadius,
    WithinBounds,
    AlongRoute,
    Nearest,
    Statistics,
}

/// Location match from spatial search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialLocationMatch {
    pub location_id: Uuid,
    pub coordinates: Coordinates,
    pub distance_meters: Option<f64>,
    pub bearing_degrees: Option<f64>,
    pub location_type: LocationTypes,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub relevance_score: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub verification_status: VerificationStatus,
}

/// Location verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Disputed,
    Pending,
}

/// Spatial region definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpatialRegion {
    Circle {
        center: Coordinates,
        radius_meters: f64,
    },
    BoundingBox {
        southwest: Coordinates,
        northeast: Coordinates,
    },
    Polygon {
        vertices: Vec<Coordinates>,
    },
    RouteCorRidor {
        route_points: Vec<Coordinates>,
        corridor_width_meters: f64,
    },
}

/// Spatial statistics for a region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialStatistics {
    pub region: SpatialRegion,
    pub total_locations: u64,
    pub density_per_km2: f64,
    pub location_type_breakdown: std::collections::HashMap<LocationTypes, u64>,
    pub category_breakdown: std::collections::HashMap<String, u64>,
    pub average_distance_between_locations: f64,
    pub clustering_coefficient: f64,
    pub hotspots: Vec<SpatialHotspot>,
    pub coverage_percentage: f64,
}

/// Spatial hotspot identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialHotspot {
    pub center: Coordinates,
    pub radius_meters: f64,
    pub location_count: u64,
    pub density_score: f64,
    pub dominant_categories: Vec<String>,
}

/// Spatial search metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialSearchMetadata {
    pub index_version: String,
    pub search_algorithm: String,
    pub cache_hit: bool,
    pub spatial_resolution: f64,
    pub performance_metrics: SpatialPerformanceMetrics,
}

/// Performance metrics for spatial searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialPerformanceMetrics {
    pub index_lookup_time_ms: u64,
    pub filtering_time_ms: u64,
    pub sorting_time_ms: u64,
    pub total_time_ms: u64,
    pub locations_scanned: u64,
    pub cache_efficiency: f64,
}

/// Spatial search errors
#[derive(Debug, Error)]
pub enum SpatialSearchError {
    #[error("Invalid coordinates: {0}")]
    InvalidCoordinates(String),
    
    #[error("Invalid search radius: {0}")]
    InvalidRadius(f64),
    
    #[error("Invalid bounding box: {0}")]
    InvalidBounds(String),
    
    #[error("Search timeout after {0}ms")]
    SearchTimeout(u64),
    
    #[error("Too many results: {0} exceeds limit of {1}")]
    TooManyResults(u64, u64),
    
    #[error("Index unavailable: {0}")]
    IndexUnavailable(String),
    
    #[error("Invalid page token: {0}")]
    InvalidPageToken(String),
    
    #[error("Unsupported query type: {0:?}")]
    UnsupportedQuery(SpatialQueryType),
    
    #[error("Service error: {0}")]
    ServiceError(String),
}

/// Mock spatial search service for testing
pub struct MockSpatialSearchService {
    pub mock_locations: Vec<SpatialLocationMatch>,
    pub response_delay_ms: u64,
}

impl MockSpatialSearchService {
    pub fn new() -> Self {
        Self {
            mock_locations: Self::generate_mock_locations(),
            response_delay_ms: 50,
        }
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.response_delay_ms = delay_ms;
        self
    }
    
    fn generate_mock_locations() -> Vec<SpatialLocationMatch> {
        vec![
            SpatialLocationMatch {
                location_id: Uuid::new_v4(),
                coordinates: Coordinates::new(37.7749, -122.4194), // San Francisco
                distance_meters: Some(100.0),
                bearing_degrees: Some(45.0),
                location_type: LocationTypes::Physical,
                name: Some("Mock Location 1".to_string()),
                description: Some("A test location".to_string()),
                tags: vec!["test".to_string(), "mock".to_string()],
                categories: vec!["testing".to_string()],
                relevance_score: 0.95,
                last_updated: chrono::Utc::now(),
                verification_status: VerificationStatus::Verified,
            },
            SpatialLocationMatch {
                location_id: Uuid::new_v4(),
                coordinates: Coordinates::new(37.7849, -122.4094),
                distance_meters: Some(500.0),
                bearing_degrees: Some(90.0),
                location_type: LocationTypes::Physical,
                name: Some("Mock Location 2".to_string()),
                description: Some("Another test location".to_string()),
                tags: vec!["test".to_string()],
                categories: vec!["testing".to_string()],
                relevance_score: 0.85,
                last_updated: chrono::Utc::now(),
                verification_status: VerificationStatus::Unverified,
            },
        ]
    }
}

impl Default for MockSpatialSearchService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SpatialSearchService for MockSpatialSearchService {
    async fn find_within_radius(
        &self,
        center: &Coordinates,
        radius_meters: f64,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialSearchResult, SpatialSearchError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        
        if radius_meters <= 0.0 || radius_meters > 100000.0 {
            return Err(SpatialSearchError::InvalidRadius(radius_meters));
        }
        
        // Filter mock locations by distance (simplified)
        let filtered_locations: Vec<SpatialLocationMatch> = self.mock_locations
            .iter()
            .filter(|loc| {
                if let Some(distance) = loc.distance_meters {
                    distance <= radius_meters
                } else {
                    true // Include if distance not calculated
                }
            })
            .filter(|loc| {
                // Apply additional filters if provided
                if let Some(ref filters) = filters {
                    if let Some(ref types) = filters.location_types {
                        if !types.contains(&loc.location_type) {
                            return false;
                        }
                    }
                    if let Some(ref tags) = filters.tags {
                        if !tags.iter().any(|tag| loc.tags.contains(tag)) {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();
        
        Ok(SpatialSearchResult {
            request_id: Uuid::new_v4(),
            query: SpatialQuery {
                query_type: SpatialQueryType::WithinRadius,
                parameters: serde_json::json!({
                    "center": center,
                    "radius_meters": radius_meters
                }),
                filters: filters.clone(),
                timestamp: chrono::Utc::now(),
            },
            locations: filtered_locations.clone(),
            total_count: filtered_locations.len() as u64,
            search_time_ms: self.response_delay_ms,
            has_more_results: false,
            next_page_token: None,
            search_metadata: SpatialSearchMetadata {
                index_version: "1.0".to_string(),
                search_algorithm: "mock_spatial_index".to_string(),
                cache_hit: false,
                spatial_resolution: 1.0, // 1 meter resolution
                performance_metrics: SpatialPerformanceMetrics {
                    index_lookup_time_ms: 10,
                    filtering_time_ms: 5,
                    sorting_time_ms: 2,
                    total_time_ms: self.response_delay_ms,
                    locations_scanned: self.mock_locations.len() as u64,
                    cache_efficiency: 0.0,
                },
            },
        })
    }
    
    async fn find_within_bounds(
        &self,
        southwest: &Coordinates,
        northeast: &Coordinates,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialSearchResult, SpatialSearchError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        
        if southwest.latitude >= northeast.latitude || southwest.longitude >= northeast.longitude {
            return Err(SpatialSearchError::InvalidBounds(
                "Southwest corner must be southwest of northeast corner".to_string()
            ));
        }
        
        // Mock implementation - return all locations for simplicity
        Ok(SpatialSearchResult {
            request_id: Uuid::new_v4(),
            query: SpatialQuery {
                query_type: SpatialQueryType::WithinBounds,
                parameters: serde_json::json!({
                    "southwest": southwest,
                    "northeast": northeast
                }),
                filters: filters.clone(),
                timestamp: chrono::Utc::now(),
            },
            locations: self.mock_locations.clone(),
            total_count: self.mock_locations.len() as u64,
            search_time_ms: self.response_delay_ms,
            has_more_results: false,
            next_page_token: None,
            search_metadata: SpatialSearchMetadata {
                index_version: "1.0".to_string(),
                search_algorithm: "mock_spatial_index".to_string(),
                cache_hit: false,
                spatial_resolution: 1.0,
                performance_metrics: SpatialPerformanceMetrics {
                    index_lookup_time_ms: 10,
                    filtering_time_ms: 5,
                    sorting_time_ms: 2,
                    total_time_ms: self.response_delay_ms,
                    locations_scanned: self.mock_locations.len() as u64,
                    cache_efficiency: 0.0,
                },
            },
        })
    }
    
    async fn find_along_route(
        &self,
        _route_points: &[Coordinates],
        _corridor_width_meters: f64,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialSearchResult, SpatialSearchError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        
        // Mock implementation
        Ok(SpatialSearchResult {
            request_id: Uuid::new_v4(),
            query: SpatialQuery {
                query_type: SpatialQueryType::AlongRoute,
                parameters: serde_json::json!({}),
                filters: filters.clone(),
                timestamp: chrono::Utc::now(),
            },
            locations: self.mock_locations.clone(),
            total_count: self.mock_locations.len() as u64,
            search_time_ms: self.response_delay_ms,
            has_more_results: false,
            next_page_token: None,
            search_metadata: SpatialSearchMetadata {
                index_version: "1.0".to_string(),
                search_algorithm: "mock_spatial_index".to_string(),
                cache_hit: false,
                spatial_resolution: 1.0,
                performance_metrics: SpatialPerformanceMetrics {
                    index_lookup_time_ms: 10,
                    filtering_time_ms: 5,
                    sorting_time_ms: 2,
                    total_time_ms: self.response_delay_ms,
                    locations_scanned: self.mock_locations.len() as u64,
                    cache_efficiency: 0.0,
                },
            },
        })
    }
    
    async fn find_nearest(
        &self,
        _point: &Coordinates,
        max_results: u32,
        _max_distance_meters: Option<f64>,
        filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialSearchResult, SpatialSearchError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        
        let mut locations = self.mock_locations.clone();
        locations.truncate(max_results as usize);
        
        Ok(SpatialSearchResult {
            request_id: Uuid::new_v4(),
            query: SpatialQuery {
                query_type: SpatialQueryType::Nearest,
                parameters: serde_json::json!({"max_results": max_results}),
                filters: filters.clone(),
                timestamp: chrono::Utc::now(),
            },
            locations,
            total_count: max_results.min(self.mock_locations.len() as u32) as u64,
            search_time_ms: self.response_delay_ms,
            has_more_results: self.mock_locations.len() > max_results as usize,
            next_page_token: None,
            search_metadata: SpatialSearchMetadata {
                index_version: "1.0".to_string(),
                search_algorithm: "mock_spatial_index".to_string(),
                cache_hit: false,
                spatial_resolution: 1.0,
                performance_metrics: SpatialPerformanceMetrics {
                    index_lookup_time_ms: 10,
                    filtering_time_ms: 5,
                    sorting_time_ms: 2,
                    total_time_ms: self.response_delay_ms,
                    locations_scanned: self.mock_locations.len() as u64,
                    cache_efficiency: 0.0,
                },
            },
        })
    }
    
    async fn get_spatial_statistics(
        &self,
        region: &SpatialRegion,
        _filters: Option<SpatialSearchFilters>,
    ) -> Result<SpatialStatistics, SpatialSearchError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        
        let mut type_breakdown = std::collections::HashMap::new();
        let mut category_breakdown = std::collections::HashMap::new();
        
        for location in &self.mock_locations {
            *type_breakdown.entry(location.location_type.clone()).or_insert(0) += 1;
            for category in &location.categories {
                *category_breakdown.entry(category.clone()).or_insert(0) += 1;
            }
        }
        
        Ok(SpatialStatistics {
            region: region.clone(),
            total_locations: self.mock_locations.len() as u64,
            density_per_km2: 50.0, // Mock density
            location_type_breakdown: type_breakdown,
            category_breakdown,
            average_distance_between_locations: 250.0, // Mock average
            clustering_coefficient: 0.3, // Mock clustering
            hotspots: vec![
                SpatialHotspot {
                    center: Coordinates::new(37.7749, -122.4194),
                    radius_meters: 500.0,
                    location_count: 2,
                    density_score: 0.8,
                    dominant_categories: vec!["testing".to_string()],
                }
            ],
            coverage_percentage: 75.0, // Mock coverage
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_within_radius() {
        let service = MockSpatialSearchService::new();
        let center = Coordinates::new(37.7749, -122.4194);
        
        let result = service.find_within_radius(&center, 1000.0, None).await.unwrap();

        assert!(!result.locations.is_empty());
        assert_eq!(result.query.query_type, SpatialQueryType::WithinRadius);
    }
    
    #[tokio::test]
    async fn test_invalid_radius() {
        let service = MockSpatialSearchService::new();
        let center = Coordinates::new(37.7749, -122.4194);
        
        let result = service.find_within_radius(&center, -100.0, None).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SpatialSearchError::InvalidRadius(_) => (),
            _ => panic!("Expected InvalidRadius error"),
        }
    }
    
    #[tokio::test]
    async fn test_find_within_bounds() {
        let service = MockSpatialSearchService::new();
        let southwest = Coordinates::new(37.7000, -122.5000);
        let northeast = Coordinates::new(37.8000, -122.4000);
        
        let result = service.find_within_bounds(&southwest, &northeast, None).await.unwrap();

        assert!(!result.locations.is_empty());
        assert_eq!(result.query.query_type, SpatialQueryType::WithinBounds);
    }
    
    #[tokio::test]
    async fn test_invalid_bounds() {
        let service = MockSpatialSearchService::new();
        // Invalid bounds: southwest is northeast of northeast
        let southwest = Coordinates::new(37.8000, -122.4000);
        let northeast = Coordinates::new(37.7000, -122.5000);
        
        let result = service.find_within_bounds(&southwest, &northeast, None).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SpatialSearchError::InvalidBounds(_) => (),
            _ => panic!("Expected InvalidBounds error"),
        }
    }
    
    #[tokio::test]
    async fn test_find_nearest() {
        let service = MockSpatialSearchService::new();
        let point = Coordinates::new(37.7749, -122.4194);
        
        let result = service.find_nearest(&point, 1, None, None).await.unwrap();

        assert_eq!(result.locations.len(), 1);
        assert_eq!(result.query.query_type, SpatialQueryType::Nearest);
    }
    
    #[tokio::test]
    async fn test_spatial_statistics() {
        let service = MockSpatialSearchService::new();
        let region = SpatialRegion::Circle {
            center: Coordinates::new(37.7749, -122.4194),
            radius_meters: 1000.0,
        };
        
        let result = service.get_spatial_statistics(&region, None).await.unwrap();

        assert!(result.total_locations > 0);
        assert!(!result.location_type_breakdown.is_empty());
        assert!(!result.hotspots.is_empty());
    }
    
    #[tokio::test]
    async fn test_search_with_filters() {
        let service = MockSpatialSearchService::new();
        let center = Coordinates::new(37.7749, -122.4194);
        let filters = SpatialSearchFilters {
            location_types: Some(vec![LocationTypes::Physical]),
            tags: Some(vec!["test".to_string()]),
            categories: None,
            owner_id: None,
            created_after: None,
            created_before: None,
            min_activity_score: None,
            verified_only: None,
            metadata_filters: None,
        };
        
        let result = service.find_within_radius(&center, 1000.0, Some(filters)).await.unwrap();

        // All results should match the filter criteria
        for location in &result.locations {
            assert_eq!(location.location_type, LocationTypes::Physical);
            assert!(location.tags.contains(&"test".to_string()));
        }
    }
}