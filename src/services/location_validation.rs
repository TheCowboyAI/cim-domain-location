//! Location validation services

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::value_objects::{Address, Coordinates, LocationTypes};
use thiserror::Error;

/// Location validation service trait
#[async_trait]
pub trait LocationValidationService: Send + Sync {
    /// Validate coordinates
    async fn validate_coordinates(&self, coordinates: &Coordinates) -> Result<ValidationResult, ValidationError>;
    
    /// Validate address
    async fn validate_address(&self, address: &Address) -> Result<ValidationResult, ValidationError>;
    
    /// Validate location type consistency
    async fn validate_location_type(&self, location_type: &LocationTypes, coordinates: Option<&Coordinates>) -> Result<ValidationResult, ValidationError>;
    
    /// Cross-validate address and coordinates
    async fn cross_validate(&self, address: &Address, coordinates: &Coordinates) -> Result<CrossValidationResult, ValidationError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence_score: f64,
    pub validation_issues: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossValidationResult {
    pub address_matches_coordinates: bool,
    pub distance_meters: f64,
    pub confidence_score: f64,
    pub discrepancies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub issue_type: ValidationIssueType,
    pub severity: ValidationSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    OutOfBounds,
    InvalidFormat,
    Inconsistent,
    Suspicious,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Validation service unavailable")]
    ServiceUnavailable,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Mock validation service
pub struct MockLocationValidationService;

impl Default for MockLocationValidationService {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl LocationValidationService for MockLocationValidationService {
    async fn validate_coordinates(&self, coordinates: &Coordinates) -> Result<ValidationResult, ValidationError> {
        let mut issues = Vec::new();
        
        // Basic coordinate validation
        if coordinates.latitude.abs() > 90.0 {
            issues.push(ValidationIssue {
                field: "latitude".to_string(),
                issue_type: ValidationIssueType::OutOfBounds,
                severity: ValidationSeverity::Error,
                message: "Latitude must be between -90 and 90 degrees".to_string(),
            });
        }
        
        if coordinates.longitude.abs() > 180.0 {
            issues.push(ValidationIssue {
                field: "longitude".to_string(),
                issue_type: ValidationIssueType::OutOfBounds,
                severity: ValidationSeverity::Error,
                message: "Longitude must be between -180 and 180 degrees".to_string(),
            });
        }
        
        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            confidence_score: if issues.is_empty() { 1.0 } else { 0.0 },
            validation_issues: issues,
            suggestions: vec![],
        })
    }
    
    async fn validate_address(&self, address: &Address) -> Result<ValidationResult, ValidationError> {
        let mut issues = Vec::new();
        
        if address.street1.is_empty() {
            issues.push(ValidationIssue {
                field: "street".to_string(),
                issue_type: ValidationIssueType::InvalidFormat,
                severity: ValidationSeverity::Warning,
                message: "Street address is recommended".to_string(),
            });
        }
        
        Ok(ValidationResult {
            is_valid: true, // Mock always validates
            confidence_score: if issues.is_empty() { 1.0 } else { 0.8 },
            validation_issues: issues,
            suggestions: vec![],
        })
    }
    
    async fn validate_location_type(&self, _location_type: &LocationTypes, _coordinates: Option<&Coordinates>) -> Result<ValidationResult, ValidationError> {
        Ok(ValidationResult {
            is_valid: true,
            confidence_score: 1.0,
            validation_issues: vec![],
            suggestions: vec![],
        })
    }
    
    async fn cross_validate(&self, _address: &Address, _coordinates: &Coordinates) -> Result<CrossValidationResult, ValidationError> {
        Ok(CrossValidationResult {
            address_matches_coordinates: true,
            distance_meters: 50.0, // Mock small discrepancy
            confidence_score: 0.9,
            discrepancies: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinate_validation() {
        let service = MockLocationValidationService;
        
        // Valid coordinates
        let valid_coords = Coordinates::new(37.7749, -122.4194).unwrap();
        let result = service.validate_coordinates(&valid_coords).await.unwrap();
        assert!(result.is_valid);
        
        // Invalid latitude
        let invalid_coords = Coordinates::new(91.0, -122.4194).unwrap();
        let result = service.validate_coordinates(&invalid_coords).await.unwrap();
        assert!(!result.is_valid);
        assert!(!result.validation_issues.is_empty());
    }
}