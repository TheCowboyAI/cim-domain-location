//! Location domain handlers

pub mod authentication_event_handler;
pub mod location_command_handler;
pub mod location_query_handler;

pub use authentication_event_handler::*;
pub use location_command_handler::*;
pub use location_query_handler::*;

// Re-export common types for convenience
pub use crate::commands::*;
pub use crate::events::*;
pub use crate::value_objects::{Address, GeoCoordinates, LocationType, VirtualLocation};
