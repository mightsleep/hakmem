# The crate as a kernel or firmware gets it: bare-metal targets, where
# `x86_64-unknown-none` turns SSE off and rustc refuses any
# `#[target_feature]` that implies it (design notes section 3). Nothing on the
# host would notice that breaking, so this builds it. Library only: the
# tests want `std`. One derivation, cells in a loop; the crate has no
# dependencies to cache between them.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    ...
  }: let
    craneLib = (config.rust.craneLib).overrideToolchain (_: config.rust.bare);
    src = lib.cleanSourceWith {
      src = ./..;
      filter = path: type: craneLib.filterCargoSources path type || baseNameOf path == "README.md";
      name = "source";
    };
    none = craneLib.mkCargoDerivation {
      inherit src;
      pname = "hakmem-none";
      version = "0";
      cargoArtifacts = null;
      buildPhaseCargoCommand = ''
        for target in x86_64-unknown-none aarch64-unknown-none-softfloat aarch64-unknown-none; do
          for features in "" "--no-default-features" "--features portable"; do
            echo "== $target $features"
            cargo build --lib --release --target $target $features
            cargo clippy --lib --target $target $features -- -D warnings
          done
        done

        # A kernel built with BMI2: PEXT through `Native`, general
        # registers only. No PEXT here means the cfg lost the path.
        echo "== x86_64-unknown-none +bmi2"
        RUSTFLAGS="-C target-feature=+bmi2,+popcnt --emit=asm" \
          cargo build --lib --release --target x86_64-unknown-none --target-dir target/bmi2
        # AT&T syntax: the width rides on the mnemonic, `pextq` and `pextl`.
        asm=$(find target/bmi2 -name 'hakmem-*.s' | head -n 1)
        grep -qE '\bpext[lq]\b' "$asm" || { echo "no pext in the +bmi2 kernel build" >&2; exit 1; }
      '';
      installPhaseCommand = "mkdir -p $out";
      doCheck = false;
    };
  in {
    # Cross builds: the same answer from any host, so from one.
    checks = lib.optionalAttrs (system == "x86_64-linux") {hakmem-none = none;};
  };
}
