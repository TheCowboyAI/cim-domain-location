//! Authentication event handler for the Location domain
//!
//! Handles authentication-related events from the Policy domain and
//! performs location validation operations.

use crate::aggregate::{Location, LocationMarker};
use cim_domain::{
    DomainResult,
    AggregateRepository, EntityId, DomainEvent,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Location validation requested event from Policy domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationValidationRequested {
    pub request_id: Uuid,
    pub location_context: LocationContext,
    pub validation_type: LocationValidationType,
    pub requested_at: chrono::DateTime<chrono::Utc>,
}

/// Location context for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationContext {
    pub ip_address: Option<String>,
    pub coordinates: Option<(f64, f64)>,
    pub country: Option<String>,
    pub network_type: Option<String>,
    pub device_id: Option<String>,
}

/// Location validation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationValidationType {
    Authentication,
    AccessControl,
    RiskAssessment,
}

/// Location validated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationValidated {
    pub request_id: Uuid,
    pub location_id: Option<EntityId<LocationMarker>>,
    pub validation_result: LocationValidationResult,
    pub risk_indicators: Vec<RiskIndicator>,
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

impl DomainEvent for LocationValidated {
    fn subject(&self) -> String {
        format!("location.{}.validated", self.request_id)
    }

    fn aggregate_id(&self) -> Uuid {
        self.request_id
    }

    fn event_type(&self) -> &'static str {
        "LocationValidated"
    }
}

/// Location validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationValidationResult {
    pub is_valid: bool,
    pub is_trusted: bool,
    pub location_type: LocationType,
    pub confidence_score: f32,
}

/// Location type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LocationType {
    Corporate,
    Home,
    Public,
    VPN,
    Unknown,
}

/// Risk indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskIndicator {
    pub indicator_type: String,
    pub risk_level: RiskLevel,
    pub description: String,
}

/// Risk level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Authentication event handler for Location domain
pub struct AuthenticationEventHandler<L: AggregateRepository<Location>> {
    location_repository: L,
    trusted_networks: Vec<NetworkRange>,
    geo_restrictions: Vec<GeoRestriction>,
}

/// Network range for trusted networks
#[derive(Debug, Clone)]
pub struct NetworkRange {
    pub name: String,
    pub cidr: String,
    pub location_type: LocationType,
}

/// Geographic restriction
#[derive(Debug, Clone)]
pub struct GeoRestriction {
    pub country_code: String,
    pub allowed: bool,
    pub risk_level: RiskLevel,
}

impl<L: AggregateRepository<Location>> AuthenticationEventHandler<L> {
    /// Create a new authentication event handler
    pub fn new(
        location_repository: L,
        trusted_networks: Vec<NetworkRange>,
        geo_restrictions: Vec<GeoRestriction>,
    ) -> Self {
        Self {
            location_repository,
            trusted_networks,
            geo_restrictions,
        }
    }

    /// Handle location validation requested event
    pub async fn handle_location_validation_requested(
        &self,
        event: LocationValidationRequested,
    ) -> DomainResult<Vec<Box<dyn cim_domain::DomainEvent>>> {
        let mut events = Vec::new();
        let mut risk_indicators = Vec::new();

        // Validate IP address
        let (is_trusted_network, location_type) = if let Some(ip) = &event.location_context.ip_address {
            self.validate_ip_address(ip, &mut risk_indicators)
        } else {
            risk_indicators.push(RiskIndicator {
                indicator_type: "missing_ip".to_string(),
                risk_level: RiskLevel::Medium,
                description: "No IP address provided".to_string(),
            });
            (false, LocationType::Unknown)
        };

        // Validate geographic location
        let is_geo_valid = if let Some(country) = &event.location_context.country {
            self.validate_geographic_location(country, &mut risk_indicators)
        } else {
            risk_indicators.push(RiskIndicator {
                indicator_type: "missing_country".to_string(),
                risk_level: RiskLevel::Low,
                description: "No country information provided".to_string(),
            });
            true // Default to allowing if no country specified
        };

        // Check for VPN/proxy
        if let Some(network_type) = &event.location_context.network_type {
            if network_type == "vpn" || network_type == "proxy" {
                risk_indicators.push(RiskIndicator {
                    indicator_type: "vpn_detected".to_string(),
                    risk_level: RiskLevel::Medium,
                    description: format!("{} connection detected", network_type),
                });
            }
        }

        // Calculate confidence score
        let confidence_score = self.calculate_confidence_score(&event.location_context);

        // Create validation result
        let validation_result = LocationValidationResult {
            is_valid: is_geo_valid,
            is_trusted: is_trusted_network,
            location_type,
            confidence_score,
        };

        // Try to find or create location aggregate
        let location_id = if validation_result.is_valid {
            self.find_or_create_location(&event.location_context).await.ok()
        } else {
            None
        };

        // Create location validated event
        events.push(Box::new(LocationValidated {
            request_id: event.request_id,
            location_id,
            validation_result,
            risk_indicators,
            validated_at: chrono::Utc::now(),
        }) as Box<dyn cim_domain::DomainEvent>);

        Ok(events)
    }

    /// Validate IP address against trusted networks
    fn validate_ip_address(
        &self,
        ip: &str,
        risk_indicators: &mut Vec<RiskIndicator>,
    ) -> (bool, LocationType) {
        // Simple implementation - in real system would use proper IP range checking
        for network in &self.trusted_networks {
            if network.name.contains("corporate") && ip.starts_with("10.") {
                return (true, LocationType::Corporate);
            }
            if network.name.contains("vpn") && ip.starts_with("172.") {
                return (true, LocationType::VPN);
            }
        }

        // Check for suspicious IPs
        if ip.starts_with("192.168.") {
            return (false, LocationType::Home);
        }

        risk_indicators.push(RiskIndicator {
            indicator_type: "untrusted_network".to_string(),
            risk_level: RiskLevel::Low,
            description: "Connection from untrusted network".to_string(),
        });

        (false, LocationType::Public)
    }

    /// Validate geographic location
    fn validate_geographic_location(
        &self,
        country: &str,
        risk_indicators: &mut Vec<RiskIndicator>,
    ) -> bool {
        for restriction in &self.geo_restrictions {
            if restriction.country_code == country {
                if !restriction.allowed {
                    risk_indicators.push(RiskIndicator {
                        indicator_type: "geo_restriction".to_string(),
                        risk_level: restriction.risk_level.clone(),
                        description: format!("Access from restricted country: {}", country),
                    });
                    return false;
                }
                return true;
            }
        }

        // Default allow if not in restriction list
        true
    }

    /// Calculate confidence score for location
    fn calculate_confidence_score(&self, context: &LocationContext) -> f32 {
        let mut score = 0.0;
        let mut factors = 0;

        if context.ip_address.is_some() {
            score += 0.3;
            factors += 1;
        }

        if context.coordinates.is_some() {
            score += 0.3;
            factors += 1;
        }

        if context.country.is_some() {
            score += 0.2;
            factors += 1;
        }

        if context.device_id.is_some() {
            score += 0.2;
            factors += 1;
        }

        if factors > 0 {
            score / factors as f32
        } else {
            0.0
        }
    }

    /// Find or create location aggregate
    async fn find_or_create_location(
        &self,
        _context: &LocationContext,
    ) -> DomainResult<EntityId<LocationMarker>> {
        // In a real implementation, would search for existing location
        // or create a new one based on the context
        Ok(EntityId::<LocationMarker>::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_domain::InMemoryRepository;

    #[tokio::test]
    async fn test_location_validation() {
        let location_repo = InMemoryRepository::<Location>::new();
        let trusted_networks = vec![
            NetworkRange {
                name: "corporate".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                location_type: LocationType::Corporate,
            },
        ];
        let geo_restrictions = vec![
            GeoRestriction {
                country_code: "CN".to_string(),
                allowed: false,
                risk_level: RiskLevel::High,
            },
        ];

        let handler = AuthenticationEventHandler::new(
            location_repo,
            trusted_networks,
            geo_restrictions,
        );

        let event = LocationValidationRequested {
            request_id: Uuid::new_v4(),
            location_context: LocationContext {
                ip_address: Some("10.0.0.1".to_string()),
                coordinates: None,
                country: Some("US".to_string()),
                network_type: None,
                device_id: None,
            },
            validation_type: LocationValidationType::Authentication,
            requested_at: chrono::Utc::now(),
        };

        let events = handler.handle_location_validation_requested(event).await.unwrap();
        assert_eq!(events.len(), 1);

        // Verify event type
        assert_eq!(events[0].event_type(), "LocationValidated");
        
        // We can't downcast Box<dyn DomainEvent> directly, so we'll just verify the event type
        // In a real implementation, we'd use an enum or other pattern for event handling
    }
}
