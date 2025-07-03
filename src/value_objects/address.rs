//! Physical address value object

use cim_domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// Physical address value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    /// Street address (line 1)
    pub street1: String,

    /// Street address (line 2) - optional
    pub street2: Option<String>,

    /// City/locality
    pub locality: String,

    /// State/province/region
    pub region: String,

    /// Country
    pub country: String,

    /// Postal/ZIP code
    pub postal_code: String,
}

impl Address {
    /// Create a new address
    pub fn new(
        street1: String,
        locality: String,
        region: String,
        country: String,
        postal_code: String,
    ) -> Self {
        Self {
            street1,
            street2: None,
            locality,
            region,
            country,
            postal_code,
        }
    }

    /// Add second street line
    pub fn with_street2(mut self, street2: String) -> Self {
        self.street2 = Some(street2);
        self
    }

    /// Validate address invariants
    pub fn validate(&self) -> DomainResult<()> {
        if self.street1.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Street address cannot be empty".to_string(),
            ));
        }

        if self.locality.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Locality cannot be empty".to_string(),
            ));
        }

        if self.region.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Region cannot be empty".to_string(),
            ));
        }

        if self.country.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Country cannot be empty".to_string(),
            ));
        }

        if self.postal_code.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Postal code cannot be empty".to_string(),
            ));
        }

        // Additional validation could include:
        // - Country-specific postal code formats
        // - Valid country codes
        // - Region validation based on country

        Ok(())
    }

    /// Format as single-line string
    pub fn format_single_line(&self) -> String {
        let mut parts = vec![self.street1.clone()];

        if let Some(street2) = &self.street2 {
            parts.push(street2.clone());
        }

        parts.push(format!("{}, {} {}", self.locality, self.region, self.postal_code));
        parts.push(self.country.clone());

        parts.join(", ")
    }

    /// Format as multi-line string
    pub fn format_multi_line(&self) -> String {
        let mut lines = vec![self.street1.clone()];

        if let Some(street2) = &self.street2 {
            lines.push(street2.clone());
        }

        lines.push(format!("{}, {} {}", self.locality, self.region, self.postal_code));
        lines.push(self.country.clone());

        lines.join("\n")
    }
}
