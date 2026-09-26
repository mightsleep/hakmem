# Toolchains as per-system options: other modules read `config.rust.*`,
# nobody touches fenix directly. Nightly is pinned through the fenix lock
# (bumped by the weekly update-flake-lock PR); the MSRV is a concrete
# channel with a hash.
{
  inputs,
  lib,
  flake-parts-lib,
  ...
}: let
  inherit (flake-parts-lib) mkPerSystemOption;
in {
  options.perSystem = mkPerSystemOption ({
    config,
    pkgs,
    system,
    ...
  }: let
    fenixPkgs = inputs.fenix.packages.${system};
    t = lib.types;
  in {
    options.rust = {
      nightly = lib.mkOption {
        type = t.package;
        description = "Nightly toolchain: rustc, cargo, clippy, rustfmt (nightly options in rustfmt.toml), rust-src, rust-analyzer.";
      };
      nightlyDev = lib.mkOption {
        type = t.package;
        description = "The nightly toolchain plus the aarch64 std, for the dev shell: cross `cargo check` of the NEON paths.";
      };
      cross = lib.mkOption {
        type = t.package;
        description = "The nightly toolchain plus the std of every target in `apiTargets`: rustdoc for each target's public API from either host.";
      };
      apiTargets = lib.mkOption {
        type = t.listOf t.str;
        default = ["x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu"];
        description = "The targets whose public API is checked in, one file each under public-api/.";
      };
      bare = lib.mkOption {
        type = t.package;
        description = "Nightly rustc, cargo and clippy with the core and alloc of the bare-metal targets, for the soft-float check (nix/none.nix).";
      };
      miri = lib.mkOption {
        type = t.package;
        description = "Nightly toolchain with the miri component.";
      };
      msrv = lib.mkOption {
        type = t.package;
        description = "Stable toolchain at hakmem's MSRV; builds the `cargo package` tarball.";
      };
      msrvVersion = lib.mkOption {
        type = t.str;
        default = "1.89.0";
        description = "hakmem's MSRV (edition 2024 needs 1.85, cast_signed 1.87, the GFNI intrinsics 1.89). Must match rust-version in Cargo.toml.";
      };
      craneLib = lib.mkOption {
        type = t.raw;
        description = "crane lib with the nightly toolchain: builds and checks.";
      };
      craneLibMsrv = lib.mkOption {
        type = t.raw;
        description = "crane lib with the MSRV toolchain.";
      };
    };

    config.rust = {
      nightly = fenixPkgs.combine [
        fenixPkgs.latest.rustc
        fenixPkgs.latest.cargo
        fenixPkgs.latest.clippy
        fenixPkgs.latest.rustfmt
        fenixPkgs.latest.rust-src
        fenixPkgs.latest.rust-analyzer
      ];
      # The dev shell only: the CI derivations hash `nightly`, and a target
      # std they never use would rebuild every cell once.
      nightlyDev = fenixPkgs.combine [
        config.rust.nightly
        # `cargo check --target aarch64-unknown-linux-gnu`: the NEON branch
        # of `lanes` type-checks locally instead of in CI. No linker, check
        # only; running the tests still needs the arm runner.
        fenixPkgs.targets.aarch64-unknown-linux-gnu.latest.rust-std
      ];
      # The host's own std is in `nightly` already; the others join it.
      cross = fenixPkgs.combine (
        [config.rust.nightly]
        ++ map (target: fenixPkgs.targets.${target}.latest.rust-std)
        (lib.remove pkgs.stdenv.hostPlatform.rust.rustcTarget config.rust.apiTargets)
      );
      # Prebuilt core and alloc: no `-Zbuild-std`, no rust-src in the build.
      bare = fenixPkgs.combine [
        fenixPkgs.latest.rustc
        fenixPkgs.latest.cargo
        fenixPkgs.latest.clippy
        fenixPkgs.targets.x86_64-unknown-none.latest.rust-std
        fenixPkgs.targets.aarch64-unknown-none-softfloat.latest.rust-std
        fenixPkgs.targets.aarch64-unknown-none.latest.rust-std
      ];
      miri = fenixPkgs.latest.withComponents ["rustc" "cargo" "rust-src" "miri"];
      msrv =
        (fenixPkgs.toolchainOf {
          channel = config.rust.msrvVersion;
          sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
        })
        .minimalToolchain;
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (_: config.rust.nightly);
      craneLibMsrv = (inputs.crane.mkLib pkgs).overrideToolchain (_: config.rust.msrv);
    };
  });
}
