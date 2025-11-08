# CIM Domain Location - Conversion Assessment

**Date**: 2025-11-07
**Version**: 0.7.8 → 0.8.0
**Status**: READY FOR CONVERSION

## Executive Summary

The Location domain is **architecturally ready** for pure functional CT/FRP conversion. The domain maintains **excellent boundary purity** with no cross-domain violations found. The conversion will be a **straightforward architectural transformation** following the same pattern successfully applied to `cim-domain-organization` and `cim-domain-person`.

## Current State Analysis

### Domain Purity: ✅ EXCELLENT

**What Location Domain Owns** (Pure organizational concepts):
- **Physical Addresses**: street1, street2, locality, region, country, postal_code
- **Geographic Coordinates**: latitude, longitude, altitude, coordinate_system
- **Virtual Locations**: meeting rooms, online platforms, network info, IP addresses
- **Location Hierarchy**: parent/child relationships between locations
- **Location Metadata**: key-value pairs for location-specific attributes
- **Location Types**: Physical, Virtual, Logical, Hybrid

**No Cross-Domain Violations Found**:
- ❌ No embedded person data (people reference locations, not vice versa)
- ❌ No embedded organization data (facilities reference locations, not vice versa)
- ❌ No facility management (that belongs in Organization domain)
- ✅ Pure location/address concepts only

### Current Architecture (v0.7.8)

**Aggregate Structure** (`src/aggregate/location.rs`):
```rust
pub struct Location {
    entity: Entity<LocationMarker>,
    version: u64,
    name: String,
    location_type: LocationType,
    address: Option<Address>,
    coordinates: Option<GeoCoordinates>,
    virtual_location: Option<VirtualLocation>,
    parent_id: Option<EntityId<LocationMarker>>,
    metadata: HashMap<String, String>,
    archived: bool,
}
```

**Current Pattern**: Mutable methods like `set_address(&mut self)`, `set_coordinates(&mut self)`

**Events** (`src/events/events.rs` and `src/domain_events.rs`):
- ✅ LocationDefined - location created
- ✅ LocationUpdated - details changed
- ✅ ParentLocationSet - hierarchy established
- ✅ ParentLocationRemoved - hierarchy removed
- ✅ LocationMetadataAdded - metadata added
- ✅ LocationArchived - soft deleted

**Value Objects** (`src/value_objects/`):
- ✅ Address - physical address components
- ✅ GeoCoordinates - latitude, longitude, altitude
- ✅ VirtualLocation - online/network location details
- ✅ LocationType - enum of location types

**Infrastructure**:
- ✅ Command handlers exist (`src/handlers/location_command_handler.rs`)
- ✅ Query handlers exist (`src/handlers/location_query_handler.rs`)
- ✅ NATS subjects defined (`src/nats/subjects.rs`)
- ✅ MessageIdentity pattern (`src/nats/message_identity.rs`)
- ❌ No event sourcing implementation (events exist but not applied)
- ❌ No NATS service binary
- ❌ No container deployment

### Compilation Status

```bash
✅ cargo build - Success (9 warnings, all non-critical unused variables)
```

## Conversion Requirements

Following the pattern from `cim-domain-organization` v0.8.0:

### 1. Pure Functional Architecture (CT/FRP)

**Add to Location aggregate**:
```rust
impl Location {
    /// Pure event application (Category Theory approach)
    pub fn apply_event_pure(&self, event: &LocationDomainEvent) -> DomainResult<Self> {
        let mut new_aggregate = self.clone();
        match event {
            LocationDomainEvent::LocationDefined(e) => { /* ... */ }
            LocationDomainEvent::LocationUpdated(e) => { /* ... */ }
            LocationDomainEvent::ParentLocationSet(e) => { /* ... */ }
            LocationDomainEvent::ParentLocationRemoved(e) => { /* ... */ }
            LocationDomainEvent::LocationMetadataAdded(e) => { /* ... */ }
            LocationDomainEvent::LocationArchived(e) => { /* ... */ }
        }
        Ok(new_aggregate)
    }

    /// Backward-compatible mutable wrapper
    pub fn apply_event(&mut self, event: &LocationDomainEvent) -> DomainResult<()> {
        *self = self.apply_event_pure(event)?;
        Ok(())
    }
}
```

**Impact**: All 6 event types need pure handlers

### 2. Event Sourcing Integration

**Add infrastructure** (following organization domain pattern):

1. **Event Store** (`src/infrastructure/nats_event_store.rs`):
   - NatsEventStore with JetStream integration
   - Event append, load, snapshot support

2. **Repository** (`src/infrastructure/location_repository.rs`):
   - LocationRepository with event sourcing
   - Aggregate reconstruction from events
   - Snapshot management (configurable frequency)

3. **Event Publisher** (`src/adapters/nats_event_publisher.rs`):
   - Publish to NATS JetStream
   - Event correlation and causation tracking
   - Subject pattern: `events.location.{location_id}.{event_type}`

4. **Command Handler Updates** (`src/infrastructure/command_handler.rs`):
   - Convert from direct mutation to event emission
   - Return events instead of mutating aggregate

### 3. NATS Service Binary

**Create** `src/bin/location-service.rs`:
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Environment configuration
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let stream_name = env::var("STREAM_NAME").unwrap_or_else(|_| "LOCATION_EVENTS".to_string());

    // Initialize NATS
    let client = async_nats::connect(&nats_url).await?;
    let jetstream = async_nats::jetstream::new(client.clone());

    // Create event store and repository
    let event_store = Arc::new(NatsEventStore::new(jetstream, stream_name).await?);
    let repository = Arc::new(LocationRepository::new(event_store.clone()));

    // Create command handler
    let command_handler = LocationCommandHandler::new(repository.clone());

    // Subscribe to commands: location.commands.>
    // Process commands, emit events
    // Handle graceful shutdown

    Ok(())
}
```

**Environment Variables**:
- `NATS_URL` - NATS server URL (default: localhost:4222)
- `STREAM_NAME` - JetStream stream name (default: LOCATION_EVENTS)
- `LOG_LEVEL` - Logging level (default: info)
- `SNAPSHOT_FREQUENCY` - Event count between snapshots (default: 100)

### 4. Container Deployment

**Add files** (following organization domain pattern):

1. **`deployment/nix/container.nix`** - NixOS container module
2. **`deployment/nix/lxc.nix`** - Proxmox LXC configuration
3. **`deployment/nix/darwin.nix`** - macOS launchd service
4. **`deployment/CONTAINER_DEPLOYMENT.md`** - Deployment guide

**Update `flake.nix`**:
```nix
outputs = { self, nixpkgs, cim-domain, ... }: {
  nixosModules = {
    default = ./deployment/nix/container.nix;
    location-service = ./deployment/nix/container.nix;
    container = ./deployment/nix/container.nix;
  };

  nixosConfigurations = {
    location-container = /* NixOS container config */;
    location-lxc = /* Proxmox LXC config */;
  };

  packages = {
    x86_64-linux.location-lxc = /* LXC tarball builder */;
    aarch64-darwin.location-service-darwin = /* macOS binary */;
    x86_64-darwin.location-service-darwin = /* macOS binary */;
  };

  darwinModules.default = ./deployment/nix/darwin.nix;
};
```

### 5. Dependencies

**Add to `Cargo.toml`**:
```toml
[dependencies]
async-nats = "0.38"
futures = "0.3"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 6. Testing Updates

**Update tests**:
- Add pure event application tests
- Add event sourcing integration tests
- Add NATS service tests (using embedded NATS server)
- Keep all existing domain logic tests

## Migration Impact

### Breaking Changes

❌ **NONE** - Fully backward compatible

**Why**:
- Pure functional architecture is internal implementation
- Mutable `apply_event(&mut self)` wrapper maintained
- Existing command/query handlers unchanged
- Events unchanged (already defined)
- No domain boundary changes (already pure)

### Compatibility

✅ **100% Backward Compatible**

Users can:
- Continue using existing Location aggregate
- Continue using existing commands
- Continue using existing queries
- Optionally adopt NATS service deployment
- Optionally use pure functional patterns

## Timeline Estimate

Based on `cim-domain-organization` conversion experience:

| Task | Estimated Time |
|------|----------------|
| Pure functional architecture | 2-3 hours |
| Event sourcing infrastructure | 3-4 hours |
| NATS service binary | 2-3 hours |
| Container deployment | 2-3 hours |
| Testing & validation | 2-3 hours |
| Documentation | 1-2 hours |
| **Total** | **12-18 hours** |

## Risk Assessment

### Low Risk ✅

**Why**:
- Domain boundaries already pure (no refactoring needed)
- Events already defined (just need application logic)
- Pattern proven successful in Organization domain
- Backward compatible (no breaking changes)
- Can be done incrementally

### Mitigation

- Follow exact pattern from `cim-domain-organization`
- Test each component independently
- Maintain backward compatibility throughout
- Document all changes in CHANGELOG

## Success Criteria

### Must Have (v0.8.0 Release)

- ✅ Pure functional event application (`apply_event_pure`)
- ✅ Event sourcing with NATS JetStream
- ✅ Location service binary
- ✅ Container deployment support (NixOS, Proxmox LXC, nix-darwin)
- ✅ All tests passing
- ✅ Clean compilation
- ✅ Comprehensive CHANGELOG

### Nice to Have (Future)

- 🔄 Performance benchmarks vs v0.7.8
- 🔄 Migration guide with code examples
- 🔄 Architecture documentation
- 🔄 Deployment examples

## Conversion Steps

1. **Create conversion branch** `ct-frp-conversion`
2. **Add pure event application** to Location aggregate
3. **Implement event sourcing** infrastructure
4. **Create NATS service** binary
5. **Add container deployment** configurations
6. **Update tests** for new patterns
7. **Create CHANGELOG** for v0.8.0
8. **Update documentation**
9. **Test deployment** scenarios
10. **Merge to main** and tag v0.8.0

## References

- **Template**: `cim-domain-organization` v0.8.0
- **Pattern**: `cim-domain-person` v0.8.0
- **Standards**: CIM pure functional architecture (CT/FRP)
- **Deployment**: NATS JetStream event sourcing
- **Containers**: NixOS, Proxmox LXC, nix-darwin

## Conclusion

The Location domain is **ideal for conversion** to v0.8.0:

✅ Clean domain boundaries (no violations)
✅ Events already defined (6 event types)
✅ Value objects well-structured
✅ NATS infrastructure partially in place
✅ Proven conversion pattern available
✅ Backward compatible approach

**Recommendation**: Proceed with conversion following the `cim-domain-organization` pattern. Expected completion: 12-18 hours.
