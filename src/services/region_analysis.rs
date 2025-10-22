//! Region analysis services

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::value_objects::Coordinates;

#[async_trait]
pub trait RegionAnalysisService: Send + Sync {
    async fn analyze_region(&self, region_id: &Uuid) -> Result<RegionAnalysis, RegionAnalysisError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAnalysis {
    pub region_id: Uuid,
    pub area_km2: f64,
    pub location_density: f64,
    pub center_point: Coordinates,
}

#[derive(Debug, thiserror::Error)]
pub enum RegionAnalysisError {
    #[error("Region not found")]
    NotFound,
}

pub struct MockRegionAnalysisService;

impl Default for MockRegionAnalysisService {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl RegionAnalysisService for MockRegionAnalysisService {
    async fn analyze_region(&self, region_id: &Uuid) -> Result<RegionAnalysis, RegionAnalysisError> {
        Ok(RegionAnalysis {
            region_id: *region_id,
            area_km2: 100.0,
            location_density: 25.0,
            center_point: Coordinates::new(37.7749, -122.4194),
        })
    }
}