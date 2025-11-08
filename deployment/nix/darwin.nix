# nix-darwin Module for Location Service
#
# This module provides a launchd service configuration for running
# the location-service on macOS via nix-darwin.
#
# Usage in darwin-configuration.nix:
#
# ```nix
# {
#   imports = [ ./path/to/cim-domain-location/deployment/nix/darwin.nix ];
#
#   services.location-service = {
#     enable = true;
#     natsUrl = "nats://localhost:4222";
#     streamName = "LOCATION_EVENTS";
#     logLevel = "info";
#   };
# }
# ```

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.location-service;

  # Location service package for macOS
  location-service = pkgs.rustPlatform.buildRustPackage rec {
    pname = "location-service";
    version = "0.8.0";

    src = ../..;

    cargoLock = {
      lockFile = ../../Cargo.lock;
    };

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = lib.optionals pkgs.stdenv.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.Security
      pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    ];

    # Build only the service binary
    cargoBuildFlags = [ "--bin" "location-service" ];

    meta = {
      description = "CIM Location Domain Service with NATS event sourcing";
      homepage = "https://github.com/thecowboyai/cim-domain-location";
      license = lib.licenses.mit;
      platforms = lib.platforms.darwin;
    };
  };

in {
  options.services.location-service = {
    enable = mkEnableOption "Location Service with NATS event sourcing";

    package = mkOption {
      type = types.package;
      default = location-service;
      description = "The location-service package to use";
    };

    natsUrl = mkOption {
      type = types.str;
      default = "nats://localhost:4222";
      description = "NATS server URL";
      example = "nats://nats-server.example.com:4222";
    };

    streamName = mkOption {
      type = types.str;
      default = "LOCATION_EVENTS";
      description = "JetStream stream name for location events";
    };

    logLevel = mkOption {
      type = types.enum [ "trace" "debug" "info" "warn" "error" ];
      default = "info";
      description = "Log level for the service";
    };

    snapshotFrequency = mkOption {
      type = types.int;
      default = 100;
      description = "Number of events between snapshots";
    };
  };

  config = mkIf cfg.enable {
    # Create launchd service
    launchd.daemons.location-service = {
      serviceConfig = {
        ProgramArguments = [ "${cfg.package}/bin/location-service" ];

        EnvironmentVariables = {
          NATS_URL = cfg.natsUrl;
          STREAM_NAME = cfg.streamName;
          LOG_LEVEL = cfg.logLevel;
          SNAPSHOT_FREQUENCY = toString cfg.snapshotFrequency;
        };

        # Keep service running
        KeepAlive = true;
        RunAtLoad = true;

        # Restart on failure
        SuccessfulExit = false;

        # Logging
        StandardOutPath = "/var/log/location-service.log";
        StandardErrorPath = "/var/log/location-service.error.log";

        # Working directory
        WorkingDirectory = "/var/lib/location-service";
      };
    };

    # Ensure log directory exists
    system.activationScripts.preActivation.text = ''
      mkdir -p /var/log
      mkdir -p /var/lib/location-service
    '';
  };
}
