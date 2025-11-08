//! Adapter implementations for location domain
//!
//! Adapters implement ports using specific technologies (NATS, HTTP, etc.)

pub mod nats_event_publisher;

pub use nats_event_publisher::*;
