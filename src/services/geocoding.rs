//! Geocoding services for address-to-coordinate conversion

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::value_objects::{Address, Coordinates};
use thiserror::Error;

/// Geocoding service trait for converting addresses to coordinates
#[async_trait]
pub trait GeocodingService: Send + Sync {
    /// Convert an address to coordinates
    async fn geocode(&self, address: &Address) -> Result<GeocodeResult, GeocodingError>;
    
    /// Convert coordinates to an address (reverse geocoding)
    async fn reverse_geocode(&self, coordinates: &Coordinates) -> Result<ReverseGeocodeResult, GeocodingError>;
    
    /// Batch geocode multiple addresses
    async fn batch_geocode(&self, addresses: &[Address]) -> Result<Vec<GeocodeResult>, GeocodingError>;
    
    /// Validate an address without geocoding
    async fn validate_address(&self, address: &Address) -> Result<AddressValidationResult, GeocodingError>;
}

/// Result of geocoding operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeocodeResult {
    pub request_id: Uuid,
    pub input_address: Address,
    pub coordinates: Coordinates,
    pub confidence_score: f64,
    pub precision_level: PrecisionLevel,
    pub formatted_address: Address,
    pub additional_info: GeocodeInfo,
}

/// Result of reverse geocoding operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseGeocodeResult {
    pub request_id: Uuid,
    pub input_coordinates: Coordinates,
    pub address: Address,
    pub confidence_score: f64,
    pub precision_level: PrecisionLevel,
    pub additional_info: GeocodeInfo,
}

/// Address validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressValidationResult {
    pub request_id: Uuid,
    pub input_address: Address,
    pub is_valid: bool,
    pub validation_issues: Vec<ValidationIssue>,
    pub suggested_corrections: Vec<Address>,
    pub confidence_score: f64,
}

/// Precision level of geocoding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrecisionLevel {
    /// Exact address match
    Exact,
    /// Street-level precision
    Street,
    /// Neighborhood level
    Neighborhood,
    /// City level
    City,
    /// Region/state level
    Region,
    /// Country level
    Country,
    /// Approximate only
    Approximate,
}

/// Additional geocoding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeocodeInfo {
    pub provider: String,
    pub response_time_ms: u64,
    pub rate_limit_remaining: Option<u32>,
    pub geocoding_method: GeocodingMethod,
    pub data_sources: Vec<String>,
}

/// Method used for geocoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeocodingMethod {
    /// Real-time API call
    RealTime,
    /// Cached result
    Cached,
    /// Offline/local database
    Offline,
    /// Interpolated from nearby addresses
    Interpolated,
}

/// Address validation issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

/// Types of validation issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    /// Required field is missing
    MissingField,
    /// Field format is invalid
    InvalidFormat,
    /// Field value doesn't exist
    NonExistent,
    /// Field value is ambiguous
    Ambiguous,
    /// Field value is outdated
    Outdated,
    /// Spelling or typo detected
    Typo,
}

/// Severity of validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Critical issue preventing geocoding
    Critical,
    /// Warning that may affect accuracy
    Warning,
    /// Informational note
    Info,
}

/// Geocoding service errors
#[derive(Debug, Error)]
pub enum GeocodingError {
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("No results found for address")]
    NoResults,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("API key invalid or expired")]
    InvalidApiKey,
    
    #[error("Quota exceeded")]
    QuotaExceeded,
    
    #[error("Geocoding timeout")]
    Timeout,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid coordinates: {0}")]
    InvalidCoordinates(String),
    
    #[error("Provider error: {0}")]
    ProviderError(String),
}

/// Mock geocoding service for testing
pub struct MockGeocodingService {
    pub fail_rate: f64,
    pub response_delay_ms: u64,
}

impl MockGeocodingService {
    pub fn new() -> Self {
        Self {
            fail_rate: 0.0,
            response_delay_ms: 100,
        }
    }
    
    pub fn with_fail_rate(mut self, fail_rate: f64) -> Self {
        self.fail_rate = fail_rate;
        self
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.response_delay_ms = delay_ms;
        self
    }
}

impl Default for MockGeocodingService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GeocodingService for MockGeocodingService {
    async fn geocode(&self, address: &Address) -> Result<GeocodeResult, GeocodingError> {
        // Simulate response delay
        tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        
        // Simulate random failures
        if rand::random::<f64>() < self.fail_rate {
            return Err(GeocodingError::ServiceUnavailable("Mock failure".to_string()));
        }
        
        // Mock geocoding result
        Ok(GeocodeResult {
            request_id: Uuid::new_v4(),
            input_address: address.clone(),
            coordinates: Coordinates::new(37.7749, -122.4194), // San Francisco
            confidence_score: 0.95,
            precision_level: PrecisionLevel::Street,
            formatted_address: address.clone(),
            additional_info: GeocodeInfo {
                provider: "MockProvider".to_string(),
                response_time_ms: self.response_delay_ms,
                rate_limit_remaining: Some(1000),
                geocoding_method: GeocodingMethod::RealTime,
                data_sources: vec!["mock_database".to_string()],
            },
        })
    }
    
    async fn reverse_geocode(&self, coordinates: &Coordinates) -> Result<ReverseGeocodeResult, GeocodingError> {
        // Simulate response delay
        tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        
        // Simulate random failures
        if rand::random::<f64>() < self.fail_rate {
            return Err(GeocodingError::ServiceUnavailable("Mock failure".to_string()));
        }
        
        // Mock reverse geocoding result
        let mock_address = Address::new(
            "123 Mock Street".to_string(),
            "San Francisco".to_string(),
            "CA".to_string(),
            "US".to_string(),
            "94102".to_string(),
        );
        
        Ok(ReverseGeocodeResult {
            request_id: Uuid::new_v4(),
            input_coordinates: coordinates.clone(),
            address: mock_address,
            confidence_score: 0.90,
            precision_level: PrecisionLevel::Street,
            additional_info: GeocodeInfo {
                provider: "MockProvider".to_string(),
                response_time_ms: self.response_delay_ms,
                rate_limit_remaining: Some(999),
                geocoding_method: GeocodingMethod::RealTime,
                data_sources: vec!["mock_database".to_string()],
            },
        })
    }
    
    async fn batch_geocode(&self, addresses: &[Address]) -> Result<Vec<GeocodeResult>, GeocodingError> {
        let mut results = Vec::new();
        
        for address in addresses {
            match self.geocode(address).await {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    async fn validate_address(&self, address: &Address) -> Result<AddressValidationResult, GeocodingError> {
        // Simulate response delay
        tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        
        // Mock validation - assume valid if street is present
        let is_valid = !address.street1.is_empty();
        let validation_issues = if !is_valid {
            vec![ValidationIssue {
                issue_type: ValidationIssueType::MissingField,
                field: "street".to_string(),
                message: "Street address is required".to_string(),
                severity: ValidationSeverity::Critical,
            }]
        } else {
            vec![]
        };
        
        Ok(AddressValidationResult {
            request_id: Uuid::new_v4(),
            input_address: address.clone(),
            is_valid,
            validation_issues,
            suggested_corrections: vec![],
            confidence_score: if is_valid { 0.95 } else { 0.20 },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::Coordinates;

    #[tokio::test]
    async fn test_mock_geocoding_service() {
        let service = MockGeocodingService::new();
        let address = Address::new(
            Some("123 Test Street".to_string()),
            Some("Test City".to_string()),
            Some("CA".to_string()),
            Some("12345".to_string()),
            Some("US".to_string()),
        );
        
        let result = service.geocode(&address).await.unwrap();
        
        assert_eq!(result.input_address, address);
        assert!(result.confidence_score > 0.0);
        assert_eq!(result.additional_info.provider, "MockProvider");
    }
    
    #[tokio::test]
    async fn test_reverse_geocoding() {
        let service = MockGeocodingService::new();
        let coordinates = Coordinates::new(37.7749, -122.4194).unwrap();
        
        let result = service.reverse_geocode(&coordinates).await.unwrap();
        
        assert_eq!(result.input_coordinates, coordinates);
        assert!(result.confidence_score > 0.0);
        assert!(result.address.street.is_some());
    }
    
    #[tokio::test]
    async fn test_address_validation() {
        let service = MockGeocodingService::new();
        
        // Valid address
        let valid_address = Address::new(
            Some("123 Test Street".to_string()),
            Some("Test City".to_string()),
            Some("CA".to_string()),
            Some("12345".to_string()),
            Some("US".to_string()),
        );
        
        let result = service.validate_address(&valid_address).await.unwrap();
        assert!(result.is_valid);
        assert!(result.validation_issues.is_empty());
        
        // Invalid address (missing street)
        let invalid_address = Address::new(
            None,
            Some("Test City".to_string()),
            Some("CA".to_string()),
            Some("12345".to_string()),
            Some("US".to_string()),
        );
        
        let result = service.validate_address(&invalid_address).await.unwrap();
        assert!(!result.is_valid);
        assert!(!result.validation_issues.is_empty());
    }
    
    #[tokio::test]
    async fn test_batch_geocoding() {
        let service = MockGeocodingService::new();
        let addresses = vec![
            Address::new(
                Some("123 First Street".to_string()),
                Some("Test City".to_string()),
                Some("CA".to_string()),
                Some("12345".to_string()),
                Some("US".to_string()),
            ),
            Address::new(
                Some("456 Second Street".to_string()),
                Some("Test City".to_string()),
                Some("CA".to_string()),
                Some("12345".to_string()),
                Some("US".to_string()),
            ),
        ];
        
        let results = service.batch_geocode(&addresses).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].input_address, addresses[0]);
        assert_eq!(results[1].input_address, addresses[1]);
    }
    
    #[tokio::test]
    async fn test_service_failure_simulation() {
        let service = MockGeocodingService::new().with_fail_rate(1.0); // Always fail
        let address = Address::new(
            Some("123 Test Street".to_string()),
            Some("Test City".to_string()),
            Some("CA".to_string()),
            Some("12345".to_string()),
            Some("US".to_string()),
        );
        
        let result = service.geocode(&address).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            GeocodingError::ServiceUnavailable(_) => (),
            _ => panic!("Expected ServiceUnavailable error"),
        }
    }
}