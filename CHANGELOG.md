# Changelog

All notable changes to the CIM Location Domain will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2025-11-07

### Architecture Philosophy: Pure Functional CT/FRP

**Critical Design Decision**: The Location domain now implements pure functional architecture following Category Theory (CT) and Functional Reactive Programming (FRP) principles.

**What This Means**:
- Event sourcing: All state changes are derived from immutable events
- Pure functions: No side effects in domain logic (`apply_event_pure`)
- Time travel: Can replay events to any point in history
- Audit trails: Complete history of all changes
- Concurrency safe: No shared mutable state

### Added

- **Pure Functional Architecture**: Complete conversion to pure functions following CT/FRP principles
  - `apply_event_pure(&self) → Result<Self>` method for pure event application
  - All domain logic now side-effect free
  - Backward-compatible `apply_event(&mut self)` wrapper
  - Handles all 6 LocationDomainEvent types:
    - LocationDefined - location creation
    - LocationUpdated - detail modifications
    - ParentLocationSet - hierarchy establishment
    - ParentLocationRemoved - hierarchy removal
    - LocationMetadataAdded - metadata additions
    - LocationArchived - soft deletion

- **NATS JetStream Event Sourcing**: Production-ready event-sourced architecture
  - `NatsEventStore` with JetStream integration
  - Durable event storage with 1-year retention
  - Event append and load operations
  - Subject pattern: `events.location.{location_id}.{event_type}`

- **Event Sourcing Repository**: Aggregate reconstruction from events
  - `LocationRepository` with event sourcing
  - Aggregate reconstruction from event history
  - Snapshot support (configurable frequency, default: 100)
  - Event sequence validation (LocationDefined must be first)

- **Hexagonal Architecture Ports**: Clean separation of concerns
  - `EventPublisher` port for publishing events
  - Async publish interface with batch support
  - Query by correlation ID, aggregate ID, time range
  - Port/Adapter pattern for infrastructure flexibility

- **NATS Publisher Adapter**: Concrete NATS implementation
  - `NatsEventPublisher` implementing EventPublisher port
  - Event metadata in headers (event-type, aggregate-id)
  - JetStream integration for durable messaging

- **Location Service Binary**: Production-ready NATS service
  - `location-service` binary for NATS-based deployment
  - Command handlers for all 6 location commands:
    - `location.commands.define` - Define location
    - `location.commands.update` - Update details
    - `location.commands.set_parent` - Set parent
    - `location.commands.remove_parent` - Remove parent
    - `location.commands.add_metadata` - Add metadata
    - `location.commands.archive` - Archive location
  - Request/reply pattern for synchronous operations
  - Event publishing to JetStream
  - Environment-based configuration (NATS_URL, STREAM_NAME, LOG_LEVEL, SNAPSHOT_FREQUENCY)
  - Graceful shutdown with signal handling
  - Comprehensive tracing/logging support

### Changed

- **Event Application**: Migrated from mutable to pure functional approach
  - All 6 event handlers now use pure functions
  - No more `&mut self` mutations in domain logic
  - Events return new aggregate instances instead of mutating existing ones

- **Dependencies**: Added infrastructure dependencies
  - `async-nats 0.38` - NATS client library
  - `futures 0.3` - Async stream utilities

- **Module Structure**: Added hexagonal architecture layers
  - `src/infrastructure/` - Event store and repository implementations
  - `src/adapters/` - NATS publisher adapter
  - `src/ports/` - Port definitions (EventPublisher)
  - `src/bin/` - Service binary

### Fixed

- None (new functionality, no bugs fixed)

### Architecture

This release represents a fundamental architectural shift to pure functional programming:

**Before (0.7.8)**:
- Mutable aggregates with `&mut self` methods
- Side effects mixed with business logic
- No event sourcing
- No NATS service deployment

**After (0.8.0)**:
- 100% pure functions in domain layer
- Event sourcing with NATS JetStream
- Production-ready service binary
- Horizontal scaling via NATS

### Migration Guide

For users upgrading from 0.7.x:

#### Backward Compatible (No Breaking Changes)

1. **Existing Code**: Continues to work via backward-compatible wrappers
2. **New Code**: Use `apply_event_pure()` for pure functional approach
3. **NATS Deployment**: Optional - library can still be used standalone
4. **Service Binary**: Optional - use if deploying as NATS service

#### Adopting New Patterns (Optional)

**Event Sourcing**:
```rust
use cim_domain_location::{LocationRepository, NatsEventStore};

// Create event store
let event_store = Arc::new(
    NatsEventStore::new(jetstream, "LOCATION_EVENTS".to_string()).await?
);

// Create repository
let repository = Arc::new(
    LocationRepository::new(event_store)
        .with_snapshot_frequency(100)
);

// Load location from events
let location = repository.load(location_id).await?;
```

**Pure Functional Pattern**:
```rust
use cim_domain_location::{Location, LocationDomainEvent};

// Old (mutable)
let mut location = /* ... */;
location.apply_event(&event)?;

// New (pure functional)
let location = /* ... */;
let new_location = location.apply_event_pure(&event)?;
```

**NATS Service Deployment**:
```bash
# Start location service
NATS_URL=nats://localhost:4222 \
STREAM_NAME=LOCATION_EVENTS \
LOG_LEVEL=info \
SNAPSHOT_FREQUENCY=100 \
location-service
```

### Domain Purity

**What Location Domain Owns** (Pure location concepts):
- ✅ Physical addresses (street, locality, region, country, postal code)
- ✅ Geographic coordinates (latitude, longitude, altitude, coordinate system)
- ✅ Virtual locations (meeting rooms, online platforms, network info)
- ✅ Location hierarchy (parent/child relationships)
- ✅ Location metadata (key-value pairs)
- ✅ Location types (Physical, Virtual, Logical, Hybrid)

**No Cross-Domain Violations**:
- ❌ No embedded person data (people reference locations, not vice versa)
- ❌ No embedded organization data (facilities reference locations, not vice versa)
- ❌ No facility management (that belongs in Organization domain)

This domain maintains **excellent boundary purity** with no architectural refactoring needed.

### References

- **Conversion Assessment**: `CONVERSION_ASSESSMENT.md`
- **Pattern**: Following `cim-domain-organization` v0.8.0
- **Standards**: CIM pure functional architecture (CT/FRP)
- **Deployment**: NATS JetStream event sourcing

### Performance

No performance regressions expected. Pure functional architecture may show:
- **Improved**: Concurrency safety (no locks needed)
- **Improved**: Testing (pure functions easier to test)
- **Neutral**: Memory usage (cloning mitigated by Rust's move semantics)
- **Neutral**: CPU usage (compiler optimizations)

### Security

- TLS for NATS connections (configured via NATS server)
- Event integrity via JetStream
- Immutable event history (audit trail)
- No credentials stored in events

## [0.7.8] - Previous Release

Previous functionality maintained for backward compatibility.

---

**Note**: This release maintains 100% backward compatibility while introducing pure functional architecture. Existing code using mutable methods will continue to work.
