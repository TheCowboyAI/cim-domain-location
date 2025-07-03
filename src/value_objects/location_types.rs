//! Location type definitions

use serde::{Deserialize, Serialize};

/// Types of locations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocationType {
    /// Physical location with real-world presence
    Physical,
    /// Virtual location (e.g., online meeting room, game world)
    Virtual,
    /// Logical location (e.g., department, zone)
    Logical,
    /// Hybrid location with both physical and virtual aspects
    Hybrid,
}

impl LocationType {
    /// Check if location can have physical attributes
    pub fn can_have_physical_attributes(&self) -> bool {
        matches!(self, LocationType::Physical | LocationType::Hybrid)
    }

    /// Check if location can have virtual attributes
    pub fn can_have_virtual_attributes(&self) -> bool {
        matches!(self, LocationType::Virtual | LocationType::Hybrid)
    }
}
