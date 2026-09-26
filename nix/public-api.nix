# The public API as text, one file per target under public-api/: the
# instruction-set tokens make it differ by architecture, so one file
# cannot be right on both CI runners. A snapshot (nix/hakmem.nix): a
# change shows up in the diff of the change that made it, and the check
# fails until the files say the same.
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
    inherit (config.hakmem) keep;
    inherit (config.rust) apiTargets;
    src = config.hakmem.src [keep.readme];
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
    api = pkgs.symlinkJoin {
      name = "hakmem-public-api";
      paths = map apiOf apiTargets;
    };
  in {
    packages.public-api = api;
    hakmem.snapshots.hakmem-public-api = {
      files =
        apiTargets
        |> map (t: lib.nameValuePair "public-api/${t}.txt" "${api}/${t}.txt")
        |> lib.listToAttrs;
      why = "The public API changed.";
      description = "the public API matches public-api/, every target";
    };
  };
}
