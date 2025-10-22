//! NATS Communication Layer for Location Domain
//!
//! This module provides NATS-first communication infrastructure for the Location domain,
//! implementing CIM principles for perfect domain isolation and event-driven architecture.

pub mod subjects;
pub mod message_identity;

pub use subjects::*;
pub use message_identity::*;