//! Location command and event handlers

pub mod authentication_event_handler;
mod location_command_handler;

pub use location_command_handler::*;

pub use authentication_event_handler::{
    AuthenticationEventHandler,
    LocationValidationRequested,
    LocationValidated,
    LocationValidationResult,
    LocationType,
    RiskIndicator,
    RiskLevel,
};
