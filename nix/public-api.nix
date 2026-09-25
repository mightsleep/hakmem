# The public API as text, one file per target under public-api/: the
# instruction-set tokens make it differ by architecture, so one file
# cannot be right on both CI runners. A change shows up in the diff of
# the change that made it, and this check fails until the files say the
# same. After a deliberate change:
#
#   nix build .#public-api && cp result/*.txt public-api/
#
# rustdoc needs no linker, so every host checks every target through the
# `cross` toolchain. cargo-public-api reads the rustdoc JSON of the pinned
# nightly; the two move together with the weekly flake.lock update, and
# when they part the check says so on that PR, not on a release day.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    craneLib = config.rust.craneLib.overrideToolchain (_: config.rust.cross);
    src = lib.cleanSourceWith {
      src = ./..;
      filter = path: type: craneLib.filterCargoSources path type || baseNameOf path == "README.md";
      name = "source";
    };
    # Default features: what `hakmem = "0.2"` gets.
    apiOf = target:
      craneLib.mkCargoDerivation {
        inherit src;
        pname = "hakmem-public-api-${target}";
        version = "0";
        cargoArtifacts = null;
        nativeBuildInputs = [pkgs.cargo-public-api];
        buildPhaseCargoCommand = "cargo public-api --simplified --target ${target} > ${target}.txt";
        installPhaseCommand = "mkdir -p $out && cp ${target}.txt $out/";
        doCheck = false;
        doInstallCargoArtifacts = false;
      };
    apis = map apiOf config.rust.apiTargets;
    api = pkgs.symlinkJoin {
      name = "hakmem-public-api";
      paths = apis;
    };
  in {
    packages.public-api = api;
    checks.hakmem-public-api = pkgs.runCommand "hakmem-public-api" {} ''
      status=0
      for target in ${lib.escapeShellArgs config.rust.apiTargets}; do
        if ! diff -u ${../public-api}/$target.txt ${api}/$target.txt; then
          status=1
        fi
      done
      if [ $status -ne 0 ]; then
        echo "The public API changed; if on purpose: nix build .#public-api && cp result/*.txt public-api/" >&2
        exit 1
      fi
      touch $out
    '';
  };
}
