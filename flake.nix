{
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
          rustToolchain = (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
            targets = [ "x86_64-unknown-linux-musl" ];
          };
          nativeBuildInputs = [ rustToolchain pkgs.lld pkgs.clang pkgs.git ];
          craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);
          src = craneLib.cleanCargoSource ./.;
          common = {
            inherit src nativeBuildInputs;
            doCheck = false;
          };
          cargoArtifacts = craneLib.buildDepsOnly common;
          klip = craneLib.buildPackage (common // {
            inherit cargoArtifacts;
            preConfigurePhases = [ "set_hash" ];
            set_hash = ''
              export KLIP_BUILD_GIT_HASH=${builtins.substring 0 7 (if self ? rev then self.rev else "skip")}
            '';
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          });
          docker = pkgs.dockerTools.streamLayeredImage {
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
          configFile = nixpkgs.lib.mkOption {
            description = "Configuration file to use.";
            type = nixpkgs.lib.types.str;
          };
        };
        mkCmd = c: s: [ "${self.packages.default.${s}}/bin/klip" "-c" c "serve" ];
      in
      {
        overlay = oSelf: oSuper: {
          klip = self.packages.default.${oSuper.system};
        };
        nixOsModule = { config, pkgs, ... }:
          let
            cfg = config.services.klip;
          in
          {
            options.services.klip = moduleOptions;
            config = {
              users.users.klip = { isSystemUser = true; group = "klip"; };
              users.groups.klip = { };
              systemd.services.klip = {
                description = "Klip server";
                wantedBy = [ "multi-user.target" ];
                after = [ "network.target" ];
                serviceConfig = {
                  ExecStart = nixpkgs.lib.escapeShellArgs (mkCmd cfg.configFile pkgs.system);
                  Restart = "on-failure";
                  User = "klip";
                  Group = "klip";
                };
              };
            };
          };
        darwinModule = { config, pkgs, ... }:
          let cfg = config.services.klip; in {
            options.services.klip = moduleOptions;
            config = {
              launchd.user.agents.klip = {
                serviceConfig = {
                  ProgramArguments = mkCmd cfg.configFile pkgs.system;
                  RunAtLoad = true;
                  KeepAlive = true;
                };
              };
            };
          };
      }
    );
}
