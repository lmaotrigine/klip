{
  description = "klip";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          targetArgs =
            if system == "aarch64-linux" then {
              targets = [ "aarch64-unknown-linux-musl" ];
              CARGO_BUILD_TARGET = "aarch64-unknown-linux-musl";
            } else if system == "x86_64-linux" then {
              targets = [ "x86_64-unknown-linux-musl" ];
              CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
            } else { };
          _rustToolchain = (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
          rustToolchain = if system == "aarch64-linux" || system == "x86_64-linux" then _rustToolchain.override { targets = targetArgs.targets; } else _rustToolchain;
          nativeBuildInputs = [ rustToolchain pkgs.lld pkgs.clang pkgs.git ];
          craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);
          src = craneLib.cleanCargoSource ./.;
          common = {
            inherit src nativeBuildInputs;
            doCheck = false;
          };
          cargoArtifacts = craneLib.buildDepsOnly common;
          klip = craneLib.buildPackage
            (common // {
              inherit cargoArtifacts;
              preConfigurePhases = [ "set_hash" ];
              set_hash = ''
                export KLIP_BUILD_GIT_HASH=${builtins.substring 0 7 (if self ? rev then self.rev else "skip")}
              '';
            } // targetArgs);
          docker =
            pkgs.dockerTools.streamLayeredImage
              {
                name = "klip";
                tag = "latest";
                contents = [ klip ];
                config.Cmd = [ "${klip}/bin/klip" ];
              };
        in
        {

          packages = {
            inherit klip docker;
            default = klip;
          };
          devShells.default = pkgs.mkShell {
            inputsFrom = [ klip ];
          };

        }
      ) // (
      let
        moduleOptions = {
          enable = nixpkgs.lib.mkOption {
            description = "Whether to enable the klip service.";
            type = nixpkgs.lib.types.bool;
            default = false;
            example = true;
          };
          configFile = nixpkgs.lib.mkOption {
            description = "Configuration file to use.";
            type = nixpkgs.lib.types.str;
          };
        };
        mkCmd = c: s: [ "${self.packages.${s}.klip}/bin/klip" "-c" c "serve" ];
        baseServiceConfig = {
          Restart = "on-failure";
          Type = "idle";
          RestartSec = 10;
          TimeoutStopSec = 10;
          SystemCallFilter = [ "@system-service" "~@privileged" "~@resources" ];
          SystemCallErrorNumber = "EPERM";
          PrivateTmp = true;
          NoNewPrivileges = true;
          ProtectSystem = "strict";
          RestrictNamespaces = "uts ipc pid cgroup";
          ProtectProc = "invisible";
          ProtectKernelTunables = true;
          ProtectKernelModules = true;
          ProtectControlGroups = true;
          PrivateDevices = true;
          RestrictSUIDSGID = true;
          RestrictAddressFamilies = "AF_INET AF_INET6";
          PrivateIPC = true;
          SystemCallArchitectures = "native";
          CapabilityBoundingSet = [
            "~CAP_SYS_ADMIN"
            "~CAP_CHOWN"
            "~CAP_SETUID"
            "~CAP_SETGID"
            "~CAP_FOWNER"
            "~CAP_SETPCAP"
            "~CAP_SYS_PTRACE"
            "~CAP_FSETID"
            "~CAP_SETFCAP"
            "~CAP_SYS_TIME"
            "~CAP_DAC_READ_SEARCH"
            "~CAP_DAC_OVERRIDE"
            "~CAP_IPC_OWNER"
            "~CAP_NET_ADMIN"
            "~CAP_SYS_NICE"
            "~CAP_SYS_RESOURCE"
            "~CAP_KILL"
            "~CAP_SYS_PACCT"
            "~CAP_LINUX_IMMUTABLE"
            "~CAP_IPC_LOCK"
            "~CAP_BPF"
            "~CAP_SYS_TTY_CONFIG"
            "~CAP_SYS_BOOT"
            "~CAP_SYS_CHROOT"
            "~CAP_LEASE"
            "~CAP_BLOCK_SUSPEND"
            "~CAP_AUDIT_CONTROL"
          ];
          ProtectHostname = true;
          ProtectKernelLogs = true;
          PrivateUsers = true;
          ProtectClock = true;
          ProtectHome = "read-only";
          ProcSubset = "pid";
        };
      in
      {
        overlays.default = final: prev: {
          klip = self.packages.default.${prev.system};
        };
        nixosModules = {
          default = { config, pkgs, ... }:
            let
              cfg = config.services.klip;
            in
            {
              options.services.klip = moduleOptions;
              config = nixpkgs.lib.mkIf cfg.enable {
                users.users.klip = { isSystemUser = true; group = "klip"; };
                users.groups.klip = { };
                systemd.services.klip = {
                  description = "klip staging server";
                  after = [ "multi-user.target" "network-online.target" ];
                  wantedBy = [ "multi-user.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = baseServiceConfig // {
                    ExecStart = nixpkgs.lib.escapeShellArgs (mkCmd cfg.configFile pkgs.system);
                    User = "klip";
                    Group = "klip";
                  };
                };
              };
            };
          homeManager = { config, pkgs, ... }:
            let cfg = config.services.klip; in {
              options.services.klip = moduleOptions;
              config = nixpkgs.lib.mkIf cfg.enable {
                systemd.user.services.klip = {
                  Unit = {
                    Description = "klip staging server";
                    After = [ "network-online.target" ];
                    Wants = [ "network-online.target" ];
                  };
                  Service = baseServiceConfig // {
                    ExecStart = nixpkgs.lib.escapeShellArgs (mkCmd cfg.configFile pkgs.system);
                  };
                };
              };
            };
        };
        darwinModules.default = { config, pkgs, ... }:
          let cfg = config.services.klip;
          in {
            options.services.klip = moduleOptions;
            config = nixpkgs.lib.mkIf cfg.enable {
              launchd.user.agents.klip = {
                serviceConfig = {
                  ProgramArguments = nixpkgs.lib.escapeShellArgs (mkCmd cfg.configFile pkgs.system);
                  RunAtLoad = true;
                  KeepAlive = true;
                };
              };
            };
          };
      }
    );
}
