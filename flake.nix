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
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        nativeBuildInputs = [ rustToolchain pkgs.lld pkgs.clang ];
        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;
        common = {
          inherit src nativeBuildInputs;
          doCheck = false;
        };
        cargoArtifacts = craneLib.buildDepsOnly common;
        klip = craneLib.buildPackage (common // {
          inherit cargoArtifacts;
        });
      in
      {
        packages = {
          inherit klip;
          default = klip;
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = [ klip ];
        };
      }
    );
}
