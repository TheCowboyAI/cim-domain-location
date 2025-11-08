//! Port definitions for location domain
//!
//! Ports define interfaces that infrastructure adapters implement,
//! following the Hexagonal Architecture pattern.

pub mod event_publisher;

pub use event_publisher::*;
