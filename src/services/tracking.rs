//! Location tracking services

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::value_objects::Coordinates;

#[async_trait]
pub trait LocationTrackingService: Send + Sync {
    async fn start_tracking(&self, user_id: &Uuid, location_id: &Uuid) -> Result<TrackingSession, TrackingError>;
    async fn record_visit(&self, user_id: &Uuid, location_id: &Uuid, coordinates: &Coordinates) -> Result<VisitRecord, TrackingError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub location_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitRecord {
    pub visit_id: Uuid,
    pub user_id: Uuid,
    pub location_id: Uuid,
    pub coordinates: Coordinates,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum TrackingError {
    #[error("Tracking service unavailable")]
    ServiceUnavailable,
}

pub struct MockLocationTrackingService;

impl Default for MockLocationTrackingService {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl LocationTrackingService for MockLocationTrackingService {
    async fn start_tracking(&self, user_id: &Uuid, location_id: &Uuid) -> Result<TrackingSession, TrackingError> {
        Ok(TrackingSession {
            session_id: Uuid::new_v4(),
            user_id: *user_id,
            location_id: *location_id,
            started_at: chrono::Utc::now(),
        })
    }
    
    async fn record_visit(&self, user_id: &Uuid, location_id: &Uuid, coordinates: &Coordinates) -> Result<VisitRecord, TrackingError> {
        Ok(VisitRecord {
            visit_id: Uuid::new_v4(),
            user_id: *user_id,
            location_id: *location_id,
            coordinates: coordinates.clone(),
            timestamp: chrono::Utc::now(),
        })
    }
}