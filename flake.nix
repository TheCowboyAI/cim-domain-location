{
  description = "CIM Module - Location domain with NATS event sourcing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    let
      # NixOS modules (system-independent)
      nixosModules = {
        default = import ./deployment/nix/container.nix;
        location-service = import ./deployment/nix/container.nix;
        container = import ./deployment/nix/container.nix;
      };

      # Darwin modules (system-independent)
      darwinModules = {
        default = import ./deployment/nix/darwin.nix;
        location-service = import ./deployment/nix/darwin.nix;
      };

      # NixOS configurations (system-independent)
      nixosConfigurations = {
        location-container = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            ./deployment/nix/container.nix
            {
              services.location-service = {
                enable = true;
                natsUrl = "nats://localhost:4222";
                streamName = "LOCATION_EVENTS";
                logLevel = "info";
              };
            }
          ];
        };

        location-lxc = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            ./deployment/nix/lxc.nix
          ];
        };
      };
    in
    {
      # Expose modules and configurations
      inherit nixosModules darwinModules nixosConfigurations;
    } //
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        buildInputs = with pkgs; [
          openssl
          pkg-config
          protobuf
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        nativeBuildInputs = with pkgs; [
          rustVersion
          cargo-edit
          cargo-watch
        ];

        # Location service binary package
        location-service = pkgs.rustPlatform.buildRustPackage {
          pname = "location-service";
          version = "0.8.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs;
          nativeBuildInputs = [ pkgs.pkg-config ];

          cargoBuildFlags = [ "--bin" "location-service" ];

          meta = with pkgs.lib; {
            description = "CIM Location Domain Service with NATS event sourcing";
            homepage = "https://github.com/thecowboyai/cim-domain-location";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      in
      {
        packages = {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "cim-domain-location";
            version = "0.8.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            inherit buildInputs nativeBuildInputs;

            checkType = "debug";
            doCheck = false;
          };

          location-service = location-service;

          # LXC container tarball for Proxmox
          location-lxc = nixosConfigurations.location-lxc.config.system.build.tarball;
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          shellHook = ''
            echo "CIM Location Domain development environment"
            echo "Rust version: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build --bin location-service  # Build NATS service"
            echo "  nix build .#location-service         # Build service with Nix"
            echo "  nix build .#location-lxc             # Build LXC container"
          '';
        };
      });
}