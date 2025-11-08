//! Infrastructure implementations for location domain
//!
//! This module contains concrete implementations of ports,
//! including NATS JetStream integration and event sourcing.

pub mod nats_integration;
pub mod location_repository;

pub use nats_integration::*;
pub use location_repository::*;
