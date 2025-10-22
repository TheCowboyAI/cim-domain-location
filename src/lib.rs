//! Location domain for the Composable Information Machine
//!
//! This crate provides location-related functionality including:
//! - Physical locations with addresses and coordinates
//! - Virtual locations on various platforms
//! - Logical locations for organizational structures
//! - Hierarchical location relationships
//! - NATS-first communication infrastructure
//! - Geospatial services and workflows

pub mod aggregate;
pub mod commands;
pub mod domain_events;
pub mod events;
pub mod handlers;
pub mod nats;
pub mod projections;
pub mod queries;
pub mod services;
pub mod value_objects;
pub mod workflow;

// Re-export main types
pub use aggregate::*;
pub use commands::*;
// Export only the enum from domain_events to avoid conflicts
pub use domain_events::LocationDomainEvent;
// Export all event types from events module
pub use events::*;
// Export command handler from handlers
pub use handlers::LocationCommandHandler;
// Export NATS communication types
pub use nats::*;
// Export projections
pub use projections::*;
// Export queries
pub use queries::{FindNearbyLocations, GetLocation, GetLocationHierarchy};
// Export query handler separately to avoid conflicts
pub use queries::LocationQueryHandler as QueryHandler;
// Export services
pub use services::*;
// Export value objects
pub use value_objects::*;
// Export workflow types
pub use workflow::*;

// Re-export core domain types that are commonly used
pub use cim_domain::{
    AggregateRepository, Command, CommandAcknowledgment, CommandEnvelope, CommandHandler,
    DomainError, DomainEvent, EventPublisher,
};
