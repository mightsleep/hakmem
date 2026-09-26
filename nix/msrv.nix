# MSRV check over what a user gets: the `cargo package` tarball, built by
# the stable toolchain at the declared MSRV. Not `cargo +msrv test` in the
# repository, whose tooling is nightly-first.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config.rust) craneLib craneLibMsrv;
    # Every tracked file, and `exclude` in Cargo.toml decides, as it does
    # for `cargo publish`: a filter here once left the licences and the
    # changelog out of the tarball this cell tests while the real one
    # carried the public-api snapshots. The price is a rebuild of this
    # cell on a docs-only change.
    src = lib.cleanSource ./..;

    # 1. the tarball (nightly cargo, offline through crane's vendor dir)
    crate = craneLib.mkCargoDerivation {
      inherit src;
      pname = "hakmem-crate";
      version = "0";
      cargoArtifacts = null;
      buildPhaseCargoCommand = "cargo package --no-verify --offline";
      installPhaseCommand = ''
        mkdir -p $out
        cp target/package/hakmem-*.crate $out/
      '';
      doCheck = false;
    };

    # 2. the unpacked tarball: a standalone crate with its own Cargo.lock
    crateSrc = pkgs.runCommand "hakmem-crate-src" {} ''
      mkdir -p $out
      tar xf ${crate}/hakmem-*.crate -C $out --strip-components=1
    '';

    msrvArgs = {
      src = crateSrc;
      pname = "hakmem-msrv";
      version = "0";
      strictDeps = true;
      CARGO_PROFILE = "release";
      # Vendor from the repository lock (a superset): reading the tarball's
      # Cargo.lock would mean import-from-derivation. The tarball's versions
      # are a subset; cargo resolves them offline from the vendor dir.
      cargoVendorDir = craneLibMsrv.vendorCargoDeps {cargoLock = ../Cargo.lock;};
    };
    deps = craneLibMsrv.buildDepsOnly msrvArgs;
  in {
    checks.hakmem-msrv = craneLibMsrv.cargoTest (msrvArgs // {cargoArtifacts = deps;});
    packages.hakmem-crate = crate;
  };
}
