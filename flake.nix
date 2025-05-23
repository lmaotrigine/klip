{
  description = "klip";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };
  outputs = { self, nixpkgs, rust-overlay, crane }:
    let
      eachSystem = systems: f:
        let
          op = attrs: system:
            let
              ret = f system;
              op = attrs: key: attrs // {
                ${key} = (attrs.${key} or { }) // {
                  ${system} = ret.${key};
                };
              };
            in
            builtins.foldl' op attrs (builtins.attrNames ret);
        in
        builtins.foldl' op { } systems;
      eachDefaultSystem = eachSystem [ "aarch64-linux" "aarch64-darwin" "x86_64-linux" "x86_64-darwin" ];
    in
    eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          nativeBuildInputs = [ rustToolchain pkgs.llvmPackages.bintools pkgs.clang pkgs.git ];
          craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./.)
              (pkgs.lib.fileset.maybeMissing ./completions)
              (pkgs.lib.fileset.maybeMissing ./doc)
            ];
          };
          common = {
            inherit src nativeBuildInputs;
            doCheck = false;
          };
          cargoArtifacts = craneLib.buildDepsOnly common;
          klip = craneLib.buildPackage
            (common // {
              inherit cargoArtifacts;
              nativeBuildInputs = nativeBuildInputs ++ [ pkgs.installShellFiles ];
              preConfigurePhases = [ "set_hash" ];
              set_hash = ''
                export KLIP_BUILD_GIT_HASH=${builtins.substring 0 7 (if self ? rev then self.rev else "skip")}
              '';
              postInstall = ''
                installShellCompletion \
                  --bash completions/klip.bash \
                  --fish completions/klip.fish \
                  --zsh completions/_klip
                installManPage doc/klip.1
              '';
            });
        in
        {
          packages = {
            inherit klip;
            default = klip;
          };
          devShells.default = pkgs.mkShell {
            inherit nativeBuildInputs;
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
        cmdline = c: s: nixpkgs.lib.escapeShellArgs (mkCmd c s);
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
            let cfg = config.services.klip; in {
              options.services.klip = moduleOptions;
              config = nixpkgs.lib.mkIf cfg.enable {
                users.users.klip = { isSystemUser = true; group = "klip"; };
                users.groups.klip = { };
                systemd.services.klip = {
                  description = "klip staging server";
                  documentation = [ "man:klip(1)" ];
                  after = [ "multi-user.target" "network-online.target" ];
                  wantedBy = [ "multi-user.target" ];
                  wants = [ "network-online.target" ];
                  serviceConfig = baseServiceConfig // {
                    ExecStart = cmdline cfg.configFile pkgs.system;
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
                    Documentation = [ "man:klip(1)" ];
                    After = [ "network-online.target" ];
                    Wants = [ "network-online.target" ];
                  };
                  Service = baseServiceConfig // {
                    ExecStart = cmdline cfg.configFile pkgs.system;
                  };
                  Install = {
                    WantedBy = [ "default.target" ];
                  };
                };
              };
            };
        };
        darwinModules.default = { config, pkgs, ... }:
          let cfg = config.services.klip; in {
            options.services.klip = moduleOptions;
            config = nixpkgs.lib.mkIf cfg.enable {
              launchd.user.agents.klip = {
                serviceConfig = {
                  ProgramArguments = cmdline cfg.configFile pkgs.system;
                  RunAtLoad = true;
                  KeepAlive = true;
                };
              };
            };
          };
      }
    );
}
