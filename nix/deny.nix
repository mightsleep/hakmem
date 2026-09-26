# Dependencies under watch: cargo-deny (licences, bans, sources) and
# cargo-audit (RustSec advisories from the pinned flake input). Both are
# checks, both run offline. The advisory DB moves with the weekly
# update-flake-lock PR.
{inputs, ...}: {
  perSystem = {config, ...}: let
    inherit (config.rust) craneLib;
    # Cargo.toml, Cargo.lock and deny.toml are what these read; nothing compiles
    # here, so no README (cleanCargoSource would drop deny.toml).
    src = config.hakmem.src [(config.hakmem.keep.named "deny.toml")];
  in {
    checks = {
      hakmem-deny = craneLib.cargoDeny {
        inherit src;
        cargoDenyChecks = "bans licenses sources";
        meta.description = "cargo-deny: licences, bans, sources";
      };
      hakmem-audit = craneLib.cargoAudit {
        inherit src;
        inherit (inputs) advisory-db;
        meta.description = "cargo-audit: RustSec advisories, offline";
      };
    };
  };
}
