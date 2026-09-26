# Written with `|>`: evaluating it needs
# `extra-experimental-features = pipe-operators` (CI sets it in every job).
{
  description = "hakmem: bit tricks as a lawful algebra";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    # Every .nix under nix/ is a flake-parts module.
    import-tree.url = "github:vic/import-tree";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Cargo dependencies as derivations from Cargo.lock: checks run in the
    # sandbox without network.
    crane.url = "github:ipetkov/crane";
    # Formatting as a check and as `nix fmt`: one set of tools locally and in CI.
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # RustSec advisory DB for `cargo audit` in the sandbox; kept fresh by the
    # weekly update-flake-lock PR (.github/workflows/update-flake-lock.yml).
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    # Chart.js for the bench page (nix/bench.nix), served from gh-pages next
    # to it. `@4` is a jsdelivr semver range, so the same weekly PR bumps it
    # to the newest 4.x and the lock pins the bytes.
    chartjs = {
      url = "file+https://cdn.jsdelivr.net/npm/chart.js@4/dist/chart.umd.js";
      flake = false;
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} (inputs.import-tree ./nix);
}
