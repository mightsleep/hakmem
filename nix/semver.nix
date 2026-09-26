# cargo-semver-checks against the last release: `nix run .#semver`. An app,
# not a check: the baseline comes from crates.io. The binary is upstream's
# static musl build, fetched by hash, since nixpkgs has it neither in the
# binary cache nor building with its tests passing; that is what kept it in
# a GitHub action until now. Its rustdoc JSON comes from the current
# stable toolchain, as the action's did: a cargo-semver-checks release
# reads the formats of the stables of its day, so a new stable may want a
# bump of `version` and the hashes. Its work goes to target/semver-checks,
# which git ignores, so a tree it ran in stays clean for `nix run .#publish`.
{
  inputs,
  lib,
  ...
}: let
  version = "0.50.0";
  builds = {
    x86_64-linux = {
      target = "x86_64-unknown-linux-musl";
      hash = "sha256-V2P29iMQ92QouhnULKwGDFbgehuOO+A5TIxoY5iXj0g=";
    };
    aarch64-linux = {
      target = "aarch64-unknown-linux-musl";
      hash = "sha256-PEQWDD/dk/ctL1yEd0w3CmPgnQ0Xrgzcw93jRvMKE0g=";
    };
  };
in {
  perSystem = {
    pkgs,
    system,
    ...
  }: let
    build = builds.${system};
    cargo-semver-checks = pkgs.stdenvNoCC.mkDerivation {
      pname = "cargo-semver-checks";
      inherit version;
      src = pkgs.fetchurl {
        url = "https://github.com/obi1kenobi/cargo-semver-checks/releases/download/v${version}/cargo-semver-checks-${build.target}.tar.gz";
        inherit (build) hash;
      };
      sourceRoot = ".";
      installPhase = "install -Dm755 cargo-semver-checks $out/bin/cargo-semver-checks";
      doInstallCheck = true;
      installCheckPhase = "$out/bin/cargo-semver-checks semver-checks --version | grep -qx 'cargo-semver-checks ${version}'";
    };
    stable = inputs.fenix.packages.${system}.stable.minimalToolchain;
  in
    lib.optionalAttrs (builds ? ${system}) {
      packages.cargo-semver-checks = cargo-semver-checks;
      apps.semver = {
        type = "app";
        program = lib.getExe (pkgs.writeShellApplication {
          name = "hakmem-semver";
          runtimeInputs = [stable cargo-semver-checks];
          text = ''
            grep -qs '^name = "hakmem"' Cargo.toml || { echo "run it from the repository root" >&2; exit 1; }
            cargo semver-checks "$@"
          '';
        });
        meta.description = "cargo-semver-checks of the tree against the last hakmem on crates.io";
      };
    };
}
