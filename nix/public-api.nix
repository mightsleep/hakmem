# The public API as text, checked in as `public-api.txt`: a change to it
# shows up in the diff of the change that made it, and this check fails
# until the file says the same. After a deliberate change:
#
#   nix build .#public-api && cp result/public-api.txt .
#
# cargo-public-api reads the rustdoc JSON of the pinned nightly; the two
# move together with the weekly flake.lock update, and when they part the
# check says so on that PR, not on a release day.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config.rust) craneLib;
    src = lib.cleanSourceWith {
      src = ./..;
      filter = path: type: craneLib.filterCargoSources path type || baseNameOf path == "README.md";
      name = "source";
    };
    # Default features: what `hakmem = "0.2"` gets.
    api = craneLib.mkCargoDerivation {
      inherit src;
      pname = "hakmem-public-api";
      version = "0";
      cargoArtifacts = null;
      nativeBuildInputs = [pkgs.cargo-public-api];
      buildPhaseCargoCommand = "cargo public-api --simplified > public-api.txt";
      installPhaseCommand = "mkdir -p $out && cp public-api.txt $out/";
      doCheck = false;
    };
  in {
    packages.public-api = api;
    checks.hakmem-public-api = pkgs.runCommand "hakmem-public-api" {} ''
      if ! diff -u ${../public-api.txt} ${api}/public-api.txt; then
        echo "The public API changed; if on purpose: nix build .#public-api && cp result/public-api.txt ." >&2
        exit 1
      fi
      touch $out
    '';
  };
}
