# Miri over the single unsafe module (BMI2 / PCLMULQDQ intrinsics in
# word.rs). Not a `check`: `cargo miri setup` builds a sysroot from
# rust-src and fetches crates, which the Nix sandbox without network cannot
# do. Runs as an app outside the sandbox.
#
# Miri interprets MIR, about 100× slower than a debug build. The laws
# against the reference loops over u128 / Wide<N> would take tens of
# minutes and find no UB: the crate has no pointers, the only unsafe is
# the intrinsics. Hence only tests/miri.rs: every hardware primitive on
# every carrier, a few samples, against the reference, in seconds.
{
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    apps.miri-hakmem = {
      type = "app";
      meta.description = "Miri over hakmem's unsafe module (both hardware paths), outside the sandbox";
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
          run() {
            echo "== miri: RUSTFLAGS='$1'"
            RUSTFLAGS="$1" cargo miri test --test miri
          }
          # Portable paths first, then the intrinsics; Miri has shims for
          # BMI2 and PCLMULQDQ.
          run ""
          run "-C target-feature=+bmi2,+pclmulqdq"
        '';
      });
    };
  };
}
