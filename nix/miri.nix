# Miri over the unsafe modules (BMI2 / PCLMULQDQ intrinsics in word.rs,
# SSSE3 and GFNI in lanes.rs, the AVX2 batch kernels in dilated.rs and
# hilbert3.rs; not yet the AVX-512 ones, see below). Not a `check`: `cargo
# miri setup` builds a sysroot from rust-src and fetches crates, which the
# Nix sandbox without network cannot do. Runs as an app outside the sandbox.
#
# Miri interprets MIR, about 100× slower than a debug build. The laws
# against the reference loops over u128 / Wide<N> would take tens of
# minutes and find no UB: the crate has no pointers, the only unsafe is
# the intrinsics. Hence only tests/miri.rs, and only in builds that have
# intrinsics to look at: the portable build has none, and a cell without
# it was four minutes of Miri checking safe code against itself.
{
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    apps.miri-hakmem = {
      type = "app";
      meta.description = "Miri over hakmem's unsafe modules (every hardware path Miri can run), outside the sandbox";
      program = pkgs.lib.getExe (pkgs.writeShellApplication {
        name = "miri-hakmem";
        # stdenv.cc: the Miri sysroot links the std/libc build scripts natively.
        runtimeInputs = [config.rust.miri pkgs.stdenv.cc];
        text = ''
          # disable-isolation: proptest calls getcwd at start-up (regression
          # persistence), which isolation forbids. UB and provenance checks
          # are unaffected.
          export MIRIFLAGS="''${MIRIFLAGS:--Zmiri-strict-provenance -Zmiri-disable-isolation}"
          cargo miri setup
          # Miri is one thread and the runner has four: the cells run side by
          # side, each with its own target dir (the flags differ, so nothing
          # would be shared anyway), and print their logs when all are done.
          logs=$(mktemp -d)
          cells=()
          start() {
            cells+=("$1")
            RUSTFLAGS="$2" cargo miri test --test miri --target-dir "target/miri-$1" \
              >"$logs/$1" 2>&1 &
          }
          # The intrinsics Miri has shims for: BMI2, PCLMULQDQ, SSSE3, GFNI,
          # AVX2. The GFNI cell is Miri-only: the GitHub runners are a mix of
          # Zen 3 (no GFNI, no AVX-512) and Ice Lake, so a native cell would
          # SIGILL at random.
          start bmi2 "-C target-feature=+bmi2,+pclmulqdq,+ssse3"
          start gfni "-C target-feature=+bmi2,+pclmulqdq,+ssse3,+gfni"
          # The 3D and Morton column kernels for AVX2: PSHUFB and SRLV.
          start avx2 "-C target-feature=+bmi2,+pclmulqdq,+ssse3,+avx2"
          # The AVX-512 VBMI kernels are not here. Miri knows a handful of
          # AVX-512 intrinsics and stdarch moves others between LLVM
          # intrinsics and generic SIMD from one nightly to the next, so a
          # cell here breaks on updates nobody on this side made. Shims go
          # upstream (rust-lang/miri#5345: vpternlogq, vpmultishiftqb; the
          # zmm shifts are still missing), and the cells come back when a
          # nightly runs them:
          #   start vbmi "-C target-feature=+bmi2,+pclmulqdq,+ssse3,+avx512vbmi"
          #   start vbmi-gfni "-C target-feature=+bmi2,+pclmulqdq,+ssse3,+avx512vbmi,+gfni"
          failed=0
          for _ in "''${cells[@]}"; do
            wait -n || failed=1
          done
          for cell in "''${cells[@]}"; do
            echo "== miri: $cell"
            cat "$logs/$cell"
          done
          exit "$failed"
        '';
      });
    };
  };
}
