//! Location domain for the Composable Information Machine
//!
//! This crate provides location-related functionality including:
//! - Physical locations with addresses and coordinates
//! - Virtual locations on various platforms
//! - Logical locations for organizational structures
//! - Hierarchical location relationships

pub mod aggregate;
pub mod commands;
pub mod domain_events;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// Re-export main types
pub use aggregate::*;
pub use commands::*;
pub use domain_events::*;
pub use events::*;
pub use handlers::*;
pub use projections::*;
pub use queries::*;
pub use value_objects::*;

// Re-export core domain types that are commonly used
pub use cim_domain::{
    AggregateRepository, Command, CommandAcknowledgment, CommandEnvelope, CommandHandler,
    DomainError, DomainEvent, EventPublisher,
};
