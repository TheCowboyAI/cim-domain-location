//! Location value objects

// Value objects are defined in the aggregate module for now
// This module is reserved for future value object extractions

mod address;
mod coordinates;
mod location_types;
mod virtual_location;

pub use address::*;
pub use coordinates::*;
pub use location_types::*;
pub use virtual_location::*;

// Type aliases for backward compatibility
pub use coordinates::GeoCoordinates as Coordinates;
pub use location_types::LocationType as LocationTypes;
