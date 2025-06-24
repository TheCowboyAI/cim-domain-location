//! Location value objects

// Value objects are defined in the aggregate module for now
// This module is reserved for future value object extractions

mod location_types;
mod address;
mod coordinates;
mod virtual_location;

pub use location_types::*;
pub use address::*;
pub use coordinates::*;
pub use virtual_location::*;
