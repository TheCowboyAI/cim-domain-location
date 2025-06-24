//! Location domain for the Composable Information Machine
//!
//! This crate provides location-related functionality including:
//! - Physical locations with addresses and coordinates
//! - Virtual locations on various platforms
//! - Logical locations for organizational structures
//! - Hierarchical location relationships

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod value_objects;
pub mod domain_events;

// Re-export main types
pub use aggregate::*;
pub use commands::*;
pub use events::*;
pub use handlers::*;
pub use domain_events::*;
pub use value_objects::*;

// Re-export core domain types that are commonly used
pub use cim_domain::{
    DomainError, DomainEvent, Command, CommandEnvelope,
    AggregateRepository, EventPublisher, CommandHandler, CommandAcknowledgment,
};
