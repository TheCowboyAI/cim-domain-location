# CIM Location Domain

[![Version](https://img.shields.io/badge/version-0.8.0-blue.svg)](https://github.com/thecowboyai/cim-domain-location)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Pure functional location management domain with NATS event sourcing for CIM (Composable Information Machine) systems.

## Features

- **Pure Functional Architecture**: Category Theory / FRP-based event sourcing
- **NATS JetStream Integration**: Durable event streaming and storage
- **Production-Ready Service**: NATS-enabled location-service binary
- **Easy Deployment**: Unified leaf node module for NixOS/nix-darwin
- **Backward Compatible**: 100% compatible with v0.7.x
- **Security Hardened**: Built-in systemd security features

## Quick Start

### For Leaf Nodes (Recommended)

Add to your NixOS or nix-darwin configuration:

```nix
{
  inputs.cim-domain-location.url = "github:thecowboyai/cim-domain-location/v0.8.0";

  outputs = { cim-domain-location, ... }: {
    # NixOS leaf node
    nixosConfigurations.my-leaf = {
      modules = [
        cim-domain-location.leafModule
        { services.location-service.enable = true; }
      ];
    };

    # macOS leaf node (nix-darwin)
    darwinConfigurations.my-mac = {
      modules = [
        cim-domain-location.leafModule
        { services.location-service.enable = true; }
      ];
    };
  };
}
```

See [Leaf Node Deployment Guide](deployment/LEAF_NODE_DEPLOYMENT.md) for complete instructions.

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
cim-domain-location = { git = "https://github.com/thecowboyai/cim-domain-location", tag = "v0.8.0" }
```

## What's New in v0.8.0

### Pure Functional Event Sourcing

Locations are now reconstructed from immutable event history:

```rust
use cim_domain_location::aggregate::Location;
use cim_domain_location::events::LocationDomainEvent;

// Pure functional event application
let new_location = location.apply_event_pure(&event)?;

// Event sourcing repository
let repository = LocationRepository::new(event_store);
let location = repository.load(location_id).await?;
```

### NATS Service Binary

Production-ready NATS-enabled service:

```bash
# Environment configuration
export NATS_URL="nats://localhost:4222"
export STREAM_NAME="LOCATION_EVENTS"
export LOG_LEVEL="info"

# Run the service
location-service
```

**NATS Subject Patterns:**
- Commands: `location.commands.{define|update|set_parent|...}`
- Events: `events.location.{location_id}.{event_type}`

### Deployment Options

**Leaf Nodes** (Primary):
- Single module works on NixOS and nix-darwin
- Automatic platform detection
- See [LEAF_NODE_DEPLOYMENT.md](deployment/LEAF_NODE_DEPLOYMENT.md)

**Containers** (Alternative):
- NixOS systemd containers
- Proxmox LXC containers
- macOS launchd services
- See [CONTAINER_DEPLOYMENT.md](deployment/CONTAINER_DEPLOYMENT.md)

## Domain Concepts

### Location Types

**Physical Locations:**
```rust
Location::new_physical(
    name: String,
    address: Address,
    coordinates: Option<Coordinates>,
    location_type: LocationType,
    parent_id: Option<EntityId<LocationMarker>>,
)
```

**Virtual Locations:**
```rust
Location::new_virtual(
    name: String,
    virtual_location: VirtualLocation,
    location_type: LocationType,
)
```

### Location Hierarchy

Locations support parent-child relationships:

```rust
location.set_parent_location(parent_id)?;
location.remove_parent_location()?;
```

### Events

All state changes produce events:

- `LocationDefined` - New location created
- `LocationUpdated` - Details modified
- `ParentLocationSet` - Hierarchy established
- `ParentLocationRemoved` - Hierarchy removed
- `LocationMetadataAdded` - Metadata attached
- `LocationArchived` - Location archived

## Architecture

### Hexagonal Architecture (Ports & Adapters)

```
┌─────────────────────────────────────┐
│         Domain Core                 │
│  ┌──────────────────────────────┐  │
│  │  Location Aggregate          │  │
│  │  - Pure Functions            │  │
│  │  - Event Application         │  │
│  │  - Business Logic            │  │
│  └──────────────────────────────┘  │
│              ▼                      │
│  ┌──────────────────────────────┐  │
│  │  Ports (Interfaces)          │  │
│  │  - EventPublisher            │  │
│  │  - EventStore                │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
                ▼
┌─────────────────────────────────────┐
│  Adapters (Infrastructure)          │
│  - NatsEventPublisher               │
│  - NatsEventStore                   │
│  - LocationRepository               │
└─────────────────────────────────────┘
```

### Event Sourcing Flow

```
Command → Service → Aggregate → Event → EventStore → JetStream
                        ▲                     │
                        └─────────────────────┘
                          Reconstruction
```

## Configuration

### Service Options

| Option | Default | Description |
|--------|---------|-------------|
| `natsUrl` | `nats://localhost:4222` | NATS server URL |
| `streamName` | `LOCATION_EVENTS` | JetStream stream name |
| `logLevel` | `info` | Log level (trace/debug/info/warn/error) |
| `snapshotFrequency` | `100` | Events between snapshots |

### NATS Stream Configuration

Automatically created with:
- **Subjects**: `events.location.>` (all location events)
- **Retention**: 1 year
- **Storage**: File-based (durable)
- **Replicas**: 1 (configurable for HA)

## Development

### Build from Source

```bash
# Clone repository
git clone https://github.com/thecowboyai/cim-domain-location
cd cim-domain-location

# Enter development environment
nix develop

# Build library
cargo build

# Build service binary
cargo build --bin location-service

# Run tests
cargo test

# Build with Nix
nix build .#location-service
```

### Run Tests

```bash
cargo test
```

Current test coverage:
- ✅ 14/14 aggregate tests passing
- ✅ Address validation
- ✅ Coordinate validation
- ✅ Location hierarchy
- ✅ Event sourcing
- ✅ Pure event application

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history and changes
- [LEAF_NODE_DEPLOYMENT.md](deployment/LEAF_NODE_DEPLOYMENT.md) - Leaf node deployment guide
- [CONTAINER_DEPLOYMENT.md](deployment/CONTAINER_DEPLOYMENT.md) - Container deployment options
- [CONVERSION_COMPLETE.md](CONVERSION_COMPLETE.md) - v0.8.0 conversion details
- [CONVERSION_ASSESSMENT.md](CONVERSION_ASSESSMENT.md) - Pre-conversion analysis

## Migration from v0.7.x

### No Breaking Changes

v0.8.0 is 100% backward compatible with v0.7.x:

```rust
// This still works exactly the same
let mut location = Location::new_physical(/* ... */)?;
location.update_details(/* ... */)?;
```

### Optional: Adopt New Patterns

```rust
// Use pure functional approach
let new_location = location.apply_event_pure(&event)?;

// Use event sourcing
let repository = LocationRepository::new(event_store);
let location = repository.load(location_id).await?;

// Deploy as NATS service
$ location-service
```

## Production Deployment

### Leaf Node Cluster

For high availability, deploy on 3+ leaf nodes:

```nix
# Each leaf node
services.location-service = {
  enable = true;
  natsUrl = "nats://localhost:4222";  # Local NATS
  streamName = "LOCATION_EVENTS";     # Shared stream
  logLevel = "info";
  snapshotFrequency = 100;
};

# Configure NATS clustering
services.nats = {
  enable = true;
  jetstream = true;
  serverConfig.cluster = {
    name = "location-cluster";
    routes = [
      "nats://leaf-1:6222"
      "nats://leaf-2:6222"
      "nats://leaf-3:6222"
    ];
  };
};
```

### Monitoring

**NixOS:**
```bash
systemctl status location-service
journalctl -u location-service -f
```

**macOS:**
```bash
tail -f /var/log/location-service.log
```

**NATS:**
```bash
nats stream info LOCATION_EVENTS
nats sub "events.location.>"
```

## Performance

### Expected Characteristics

- **Memory**: Neutral (Rust move semantics optimize cloning)
- **CPU**: Neutral (compiler optimizations)
- **Latency**: Low (NATS is fast)
- **Throughput**: High (NATS enables horizontal scaling)
- **Concurrency**: Excellent (no locks needed with pure functions)

### Tuning

**Snapshot Frequency:**
- Lower (50-100): Faster loads, more memory
- Higher (200-1000): Slower loads, less memory
- Recommended: 100 for balanced performance

## Security

### NixOS Hardening

Automatic security features:
- `ProtectSystem=strict` - Read-only system
- `ProtectHome=true` - No home directory access
- `PrivateTmp=true` - Isolated /tmp
- `NoNewPrivileges=true` - No privilege escalation
- `RestrictAddressFamilies=AF_INET AF_INET6` - Network only

### Production Checklist

- [ ] Enable TLS for NATS connections
- [ ] Configure NATS authentication (JWT)
- [ ] Restrict firewall ports
- [ ] Use private network for clusters
- [ ] Enable monitoring and alerting
- [ ] Configure backup strategy
- [ ] Document disaster recovery

## Examples

### Using the Library

```rust
use cim_domain_location::prelude::*;

// Create physical location
let location = Location::new_physical(
    "Office Building".to_string(),
    Address {
        street: "123 Main St".to_string(),
        locality: "Springfield".to_string(),
        region: "IL".to_string(),
        country: "US".to_string(),
        postal_code: Some("62701".to_string()),
    },
    Some(Coordinates {
        latitude: 39.7817,
        longitude: -89.6501,
        altitude: None,
    }),
    LocationType::Facility,
    None,
)?;

// Add metadata
let location = location.add_metadata(
    "building_code".to_string(),
    "BLD-001".to_string(),
)?;

// Archive location
let location = location.archive()?;
```

### Using with NATS

```bash
# Subscribe to events
nats sub "events.location.>"

# Send command
nats req location.commands.define '{
  "location_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "New Office",
  "location_type": "Facility",
  "address": {
    "street": "456 Oak Ave",
    "locality": "Chicago",
    "region": "IL",
    "country": "US",
    "postal_code": "60601"
  }
}'
```

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

- **Issues**: https://github.com/thecowboyai/cim-domain-location/issues
- **Discussions**: https://github.com/thecowboyai/cim-domain-location/discussions
- **Documentation**: https://github.com/thecowboyai/cim-domain-location

## See Also

- [cim-domain](https://github.com/thecowboyai/cim-domain) - Core CIM domain framework
- [cim-domain-organization](https://github.com/thecowboyai/cim-domain-organization) - Organization domain
- [NATS Documentation](https://docs.nats.io/) - NATS messaging system
- [NixOS](https://nixos.org/) - Declarative Linux distribution
- [nix-darwin](https://github.com/LnL7/nix-darwin) - Nix for macOS

---

**Version**: 0.8.0
**Architecture**: Pure Functional CT/FRP with NATS Event Sourcing
**Deployment**: Unified Leaf Node Module for NixOS/nix-darwin
**Status**: Production Ready ✅
