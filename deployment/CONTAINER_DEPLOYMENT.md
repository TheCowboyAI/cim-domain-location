# Location Service Container Deployment Guide

This guide covers deploying the Location Service with NATS event sourcing using NixOS containers, Proxmox LXC, or nix-darwin on macOS.

## Quick Start

The Location Service supports three deployment methods:

1. **NixOS Container** - systemd-nspawn container on NixOS
2. **Proxmox LXC** - LXC container for Proxmox VE
3. **nix-darwin** - macOS launchd service

All methods use the same Nix modules for consistent configuration.

## Prerequisites

### For NixOS/LXC Deployments
- NixOS system or Proxmox VE server
- NATS server accessible from the container
- Network connectivity configured

### For macOS Deployment
- nix-darwin configured
- NATS server accessible from the Mac
- Nix with flakes enabled

## Deployment Method 1: NixOS Container

### Using as a NixOS Module

Add to your `configuration.nix`:

```nix
{
  imports = [
    (builtins.fetchGit {
      url = "https://github.com/thecowboyai/cim-domain-location";
      ref = "main";
    } + "/deployment/nix/container.nix")
  ];

  services.location-service = {
    enable = true;
    natsUrl = "nats://nats-server:4222";
    streamName = "LOCATION_EVENTS";
    logLevel = "info";
    snapshotFrequency = 100;
  };
}
```

### Using the Flake

```nix
{
  inputs.cim-domain-location.url = "github:thecowboyai/cim-domain-location";

  outputs = { self, nixpkgs, cim-domain-location }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [
        cim-domain-location.nixosModules.location-service
        {
          services.location-service = {
            enable = true;
            natsUrl = "nats://nats-server:4222";
          };
        }
      ];
    };
  };
}
```

### Building a Container

```bash
# Build the container configuration
nix build github:thecowboyai/cim-domain-location#nixosConfigurations.location-container.config.system.build.toplevel

# Or use the pre-configured container
nixos-container create location-service \
  --flake github:thecowboyai/cim-domain-location#location-container

# Start the container
nixos-container start location-service

# Check status
nixos-container status location-service

# View logs
journalctl -M location-service -u location-service
```

## Deployment Method 2: Proxmox LXC

### Building the LXC Tarball

```bash
# Build the LXC tarball
nix build github:thecowboyai/cim-domain-location#location-lxc

# The tarball will be at:
# result/tarball/nixos-system-x86_64-linux.tar.xz
```

### Importing to Proxmox

```bash
# Copy tarball to Proxmox host
scp result/tarball/nixos-system-x86_64-linux.tar.xz root@proxmox:/var/lib/vz/template/cache/

# Create LXC container
pct create 100 /var/lib/vz/template/cache/nixos-system-x86_64-linux.tar.xz \
  --hostname location-service \
  --memory 512 \
  --cores 2 \
  --net0 name=eth0,bridge=vmbr0,ip=dhcp \
  --storage local-lvm \
  --unprivileged 1

# Start container
pct start 100

# View logs
pct enter 100
journalctl -u location-service -f
```

### Configuration

The LXC container includes:
- SSH access (key-based only)
- Minimal system packages (vim, htop, curl, jq)
- Automatic service startup
- Journal log rotation (7 days, 100MB max)

Default NATS URL: `nats://nats-server:4222`

To customize, modify `deployment/nix/lxc.nix`:

```nix
services.location-service = {
  enable = true;
  natsUrl = "nats://your-nats-server:4222";
  streamName = "LOCATION_EVENTS";
  logLevel = "debug";  # or trace, info, warn, error
  snapshotFrequency = 100;
};
```

Then rebuild the tarball.

## Deployment Method 3: macOS (nix-darwin)

### Installation

Add to your `darwin-configuration.nix`:

```nix
{
  imports = [
    (builtins.fetchGit {
      url = "https://github.com/thecowboyai/cim-domain-location";
      ref = "main";
    } + "/deployment/nix/darwin.nix")
  ];

  services.location-service = {
    enable = true;
    natsUrl = "nats://localhost:4222";
    streamName = "LOCATION_EVENTS";
    logLevel = "info";
    snapshotFrequency = 100;
  };
}
```

Or using flakes:

```nix
{
  inputs.cim-domain-location.url = "github:thecowboyai/cim-domain-location";

  outputs = { self, darwin, cim-domain-location }: {
    darwinConfigurations.mymac = darwin.lib.darwinSystem {
      modules = [
        cim-domain-location.darwinModules.location-service
        {
          services.location-service.enable = true;
        }
      ];
    };
  };
}
```

### Managing the Service

```bash
# Rebuild darwin configuration
darwin-rebuild switch --flake .

# Check status
launchctl list | grep location-service

# View logs
tail -f /var/log/location-service.log
tail -f /var/log/location-service.error.log

# Restart service
launchctl kickstart -k system/org.nixos.location-service

# Stop service
launchctl stop org.nixos.location-service

# Start service
launchctl start org.nixos.location-service
```

## Configuration Options

All deployment methods support the same configuration options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | false | Enable the Location Service |
| `package` | package | auto | Package to use (usually auto-detected) |
| `natsUrl` | string | `nats://localhost:4222` | NATS server URL |
| `streamName` | string | `LOCATION_EVENTS` | JetStream stream name |
| `logLevel` | enum | `info` | Log level: trace, debug, info, warn, error |
| `snapshotFrequency` | int | 100 | Events between snapshots |
| `user` | string | `location-service` | Service user (NixOS only) |
| `group` | string | `location-service` | Service group (NixOS only) |

## NATS Configuration

### JetStream Stream

The service automatically creates a JetStream stream with:
- **Name**: Configured via `streamName` option
- **Subjects**: `events.location.>` (all location events)
- **Retention**: 1 year
- **Storage**: File-based (durable)
- **Replicas**: 1 (increase for HA)

### Subject Patterns

**Commands** (request/reply):
- `location.commands.define` - Define new location
- `location.commands.update` - Update location details
- `location.commands.set_parent` - Set parent location
- `location.commands.remove_parent` - Remove parent location
- `location.commands.add_metadata` - Add metadata entry
- `location.commands.archive` - Archive location

**Events** (publish/subscribe):
- `events.location.{location_id}.defined`
- `events.location.{location_id}.updated`
- `events.location.{location_id}.parent_set`
- `events.location.{location_id}.parent_removed`
- `events.location.{location_id}.metadata_added`
- `events.location.{location_id}.archived`

### Example: Testing with NATS CLI

```bash
# Subscribe to all location events
nats sub "events.location.>"

# Send a command (in another terminal)
nats req location.commands.define '{
  "location_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Test Location",
  "location_type": "Physical",
  "address": {
    "street": "123 Main St",
    "locality": "Springfield",
    "region": "IL",
    "country": "US",
    "postal_code": "62701"
  }
}'

# View event stream
nats stream view LOCATION_EVENTS
```

## Security Hardening (NixOS)

The NixOS/LXC deployments include comprehensive security hardening:

- **File System**: `ProtectSystem=strict`, `ProtectHome=true`, `PrivateTmp=true`
- **Privileges**: `NoNewPrivileges=true`, `RestrictSUIDSGID=true`
- **Kernel**: `ProtectKernelTunables=true`, `ProtectKernelModules=true`
- **Network**: `RestrictAddressFamilies=AF_INET AF_INET6`
- **Namespaces**: `RestrictNamespaces=true`, `PrivateMounts=true`
- **Execution**: `LockPersonality=true`, `RestrictRealtime=true`

### macOS Security

The macOS launchd service runs with standard user permissions. For production:
1. Create dedicated service user
2. Configure firewall rules
3. Use TLS for NATS connections
4. Restrict file system access

## Monitoring and Operations

### Health Checks

The service automatically logs startup and shutdown:

```bash
# NixOS/LXC
journalctl -u location-service -f

# macOS
tail -f /var/log/location-service.log
```

### Performance Monitoring

Monitor key metrics:
- **Event processing latency**: Time from command to event
- **Repository load time**: Aggregate reconstruction speed
- **NATS roundtrip**: Command request/reply latency
- **Memory usage**: Snapshot frequency affects memory

```bash
# NixOS/LXC
systemctl status location-service

# macOS
ps aux | grep location-service
top -pid $(pgrep location-service)
```

### Backup and Recovery

**Event Store Backup** (NATS JetStream):
```bash
# Backup JetStream state
nats stream backup LOCATION_EVENTS /backup/location-events-$(date +%Y%m%d).tar.gz

# Restore from backup
nats stream restore LOCATION_EVENTS /backup/location-events-20250107.tar.gz
```

**Snapshot Management**:
- Adjust `snapshotFrequency` based on event volume
- Higher frequency = faster loads, more storage
- Lower frequency = slower loads, less storage
- Recommended: 100-1000 events per snapshot

## Troubleshooting

### Service Won't Start

**Check NATS connectivity**:
```bash
# Test NATS connection
nats server ping --server=nats://your-server:4222
```

**Check logs**:
```bash
# NixOS/LXC
journalctl -u location-service -n 100

# macOS
tail -n 100 /var/log/location-service.error.log
```

**Verify configuration**:
```bash
# NixOS
systemctl cat location-service

# macOS
launchctl print system/org.nixos.location-service
```

### Events Not Publishing

**Check JetStream stream**:
```bash
nats stream info LOCATION_EVENTS
nats stream subjects LOCATION_EVENTS
```

**Check event count**:
```bash
nats stream view LOCATION_EVENTS
```

**Monitor in real-time**:
```bash
nats sub "events.location.>" --count=10
```

### High Memory Usage

**Reduce snapshot frequency**:
```nix
services.location-service.snapshotFrequency = 50;  # More frequent snapshots
```

**Check aggregate size**:
- Large metadata collections increase memory
- Many parent/child relationships increase memory
- Consider domain split if aggregates exceed 10MB

### Slow Aggregate Loads

**Increase snapshot frequency**:
```nix
services.location-service.snapshotFrequency = 200;  # Less frequent snapshots
```

**Check event count per aggregate**:
```bash
nats stream view LOCATION_EVENTS | grep location_id | sort | uniq -c
```

**Consider archival**:
- Archive old locations to reduce active event count
- Use separate streams for archived data

## Upgrading

### NixOS/LXC

```bash
# Pull latest changes
nix flake update cim-domain-location

# Rebuild container
nixos-container update location-service --flake .#location-container

# Or rebuild LXC
nix build github:thecowboyai/cim-domain-location#location-lxc
# Then import new tarball to Proxmox
```

### macOS

```bash
# Update flake input
nix flake update cim-domain-location

# Rebuild darwin configuration
darwin-rebuild switch --flake .
```

### Zero-Downtime Upgrade

For production environments:

1. **Start new instance** with updated version
2. **NATS load balancing** routes to both instances
3. **Verify new instance** processes commands correctly
4. **Drain old instance** (stop accepting new commands)
5. **Shutdown old instance** when all in-flight commands complete

## Production Checklist

Before deploying to production:

- [ ] NATS server configured with TLS
- [ ] JetStream storage sized appropriately
- [ ] Snapshot frequency tuned for workload
- [ ] Monitoring and alerting configured
- [ ] Backup strategy implemented
- [ ] Log rotation configured
- [ ] Resource limits set (memory, CPU)
- [ ] Network firewall rules configured
- [ ] Service user permissions reviewed
- [ ] High availability plan documented
- [ ] Disaster recovery tested
- [ ] Performance benchmarks established

## Examples

### Complete NixOS Configuration

```nix
{ config, pkgs, ... }:

{
  imports = [
    (builtins.fetchGit {
      url = "https://github.com/thecowboyai/cim-domain-location";
      ref = "v0.8.0";
    } + "/deployment/nix/container.nix")
  ];

  services.location-service = {
    enable = true;
    natsUrl = "nats://nats-cluster.internal:4222";
    streamName = "LOCATION_EVENTS_PROD";
    logLevel = "info";
    snapshotFrequency = 100;
  };

  # Firewall configuration
  networking.firewall.allowedTCPPorts = [ 22 ];  # SSH only

  # Monitoring
  services.prometheus.exporters.node.enable = true;
}
```

### Complete macOS Configuration

```nix
{ config, pkgs, ... }:

{
  imports = [
    (builtins.fetchGit {
      url = "https://github.com/thecowboyai/cim-domain-location";
      ref = "v0.8.0";
    } + "/deployment/nix/darwin.nix")
  ];

  services.location-service = {
    enable = true;
    natsUrl = "nats://localhost:4222";
    streamName = "LOCATION_EVENTS";
    logLevel = "debug";
    snapshotFrequency = 50;
  };

  # Local NATS server
  services.nats = {
    enable = true;
    jetstream = true;
  };
}
```

## Support

For issues and questions:
- **GitHub Issues**: https://github.com/thecowboyai/cim-domain-location/issues
- **Documentation**: https://github.com/thecowboyai/cim-domain-location
- **NATS Docs**: https://docs.nats.io

## See Also

- [Location Domain README](../README.md)
- [CHANGELOG](../CHANGELOG.md)
- [CONVERSION_COMPLETE](../CONVERSION_COMPLETE.md)
- [NATS JetStream Documentation](https://docs.nats.io/nats-concepts/jetstream)
- [NixOS Containers](https://nixos.org/manual/nixos/stable/index.html#ch-containers)
- [nix-darwin Documentation](https://github.com/LnL7/nix-darwin)
