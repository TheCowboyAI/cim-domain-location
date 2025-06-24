//! Geographic coordinates value object

use cim_domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// Geographic coordinates value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoCoordinates {
    /// Latitude in decimal degrees (-90 to 90)
    pub latitude: f64,

    /// Longitude in decimal degrees (-180 to 180)
    pub longitude: f64,

    /// Altitude in meters (optional)
    pub altitude: Option<f64>,

    /// Coordinate system (default: WGS84)
    pub coordinate_system: String,
}

impl GeoCoordinates {
    /// Create new coordinates (defaults to WGS84)
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude: None,
            coordinate_system: "WGS84".to_string(),
        }
    }

    /// Add altitude
    pub fn with_altitude(mut self, altitude: f64) -> Self {
        self.altitude = Some(altitude);
        self
    }

    /// Use different coordinate system
    pub fn with_coordinate_system(mut self, system: String) -> Self {
        self.coordinate_system = system;
        self
    }

    /// Validate coordinate ranges
    pub fn validate(&self) -> DomainResult<()> {
        if self.latitude < -90.0 || self.latitude > 90.0 {
            return Err(DomainError::ValidationError(
                format!("Latitude {} is out of range [-90, 90]", self.latitude)
            ));
        }

        if self.longitude < -180.0 || self.longitude > 180.0 {
            return Err(DomainError::ValidationError(
                format!("Longitude {} is out of range [-180, 180]", self.longitude)
            ));
        }

        Ok(())
    }

    /// Calculate distance to another point (in meters, using Haversine formula)
    pub fn distance_to(&self, other: &GeoCoordinates) -> f64 {
        const EARTH_RADIUS_M: f64 = 6_371_000.0;

        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_M * c
    }
    
    /// Calculate bearing to another point (in degrees, 0-360)
    pub fn bearing_to(&self, other: &GeoCoordinates) -> f64 {
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();
        
        let x = delta_lon.sin() * lat2.cos();
        let y = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * delta_lon.cos();
        
        let bearing_rad = x.atan2(y);  // Fixed: x.atan2(y) instead of y.atan2(x)
        let bearing_deg = bearing_rad.to_degrees();
        
        // Normalize to 0-360 degrees
        (bearing_deg + 360.0) % 360.0
    }
    
    /// Get a bounding box around this point
    pub fn bounding_box(&self, radius_meters: f64) -> BoundingBox {
        const EARTH_RADIUS_M: f64 = 6_371_000.0;
        
        // Angular distance in radians
        let angular_distance = radius_meters / EARTH_RADIUS_M;
        
        // Calculate latitude bounds
        let min_lat = self.latitude - angular_distance.to_degrees();
        let max_lat = self.latitude + angular_distance.to_degrees();
        
        // Calculate longitude bounds (accounting for latitude)
        let lat_rad = self.latitude.to_radians();
        let delta_lon = (angular_distance / lat_rad.cos()).to_degrees();
        let min_lon = self.longitude - delta_lon;
        let max_lon = self.longitude + delta_lon;
        
        BoundingBox {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        }
    }
}

/// Geographic bounding box
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl BoundingBox {
    /// Check if a point is within this bounding box
    pub fn contains(&self, coords: &GeoCoordinates) -> bool {
        coords.latitude >= self.min_lat
            && coords.latitude <= self.max_lat
            && coords.longitude >= self.min_lon
            && coords.longitude <= self.max_lon
    }
    
    /// Calculate the center of the bounding box
    pub fn center(&self) -> GeoCoordinates {
        GeoCoordinates::new(
            (self.min_lat + self.max_lat) / 2.0,
            (self.min_lon + self.max_lon) / 2.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_distance_calculation() {
        // New York City
        let nyc = GeoCoordinates::new(40.7128, -74.0060);
        
        // Los Angeles
        let la = GeoCoordinates::new(34.0522, -118.2437);
        
        // Distance should be approximately 3935 km
        let distance = nyc.distance_to(&la);
        assert!((distance - 3_935_000.0).abs() < 10_000.0); // Within 10km accuracy
    }
    
    #[test]
    fn test_bearing_calculation() {
        let start = GeoCoordinates::new(0.0, 0.0);
        let north = GeoCoordinates::new(1.0, 0.0);
        let east = GeoCoordinates::new(0.0, 1.0);
        
        let bearing_north = start.bearing_to(&north);
        let bearing_east = start.bearing_to(&east);
        
        println!("Bearing to north: {}", bearing_north);
        println!("Bearing to east: {}", bearing_east);
        
        // North should be approximately 0 degrees
        assert!((bearing_north - 0.0).abs() < 1.0, "North bearing {} should be close to 0", bearing_north);
        // East should be approximately 90 degrees  
        assert!((bearing_east - 90.0).abs() < 1.0, "East bearing {} should be close to 90", bearing_east);
    }
} 