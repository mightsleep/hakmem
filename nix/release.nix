# What goes to crates.io, checked before it goes: the file list of the
# `cargo package` tarball (the one hakmem-msrv builds and tests) against
# release/package.txt, and the version in Cargo.toml against the newest
# heading of CHANGELOG.md. After a deliberate change to the list:
#
#   nix build .#package-contents && cp result release/package.txt
#
# The upload is `nix run .#publish`, the one step that needs the network
# and a token, so no derivation. It refuses unless HEAD is the version's
# tag, the changelog dates the version, these checks pass for the tree,
# and the tarball it is about to upload holds what the checked one holds.
# `.cargo_vcs_info.json` is the one difference allowed: cargo writes the
# commit into it, and a sandbox has no git to read it from.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    crate = config.packages.hakmem-crate;
    inherit ((lib.importTOML ../Cargo.toml).package) version;

    contents = pkgs.runCommand "hakmem-package-contents" {} ''
      tar tzf ${crate}/hakmem-*.crate | sed 's|^[^/]*/||' | grep -v '^$' | LC_ALL=C sort > $out
    '';

    publish = pkgs.writeShellApplication {
      name = "hakmem-publish";
      runtimeInputs = [config.rust.nightly pkgs.git pkgs.gnutar pkgs.gzip pkgs.diffutils];
      text = ''
        die() { echo "publish: $*" >&2; exit 1; }
        version=${version}
        grep -qs '^name = "hakmem"' Cargo.toml || die "run it from the repository root"
        [ -n "''${CARGO_REGISTRY_TOKEN:-}''${HAKMEM_PUBLISH_DRY_RUN:-}" ] || die "CARGO_REGISTRY_TOKEN is not set"
        [ -z "$(git status --porcelain)" ] || die "the tree has uncommitted changes"
        [ "$(git describe --exact-match --tags HEAD 2>/dev/null)" = "v$version" ] || die "HEAD is not tagged v$version"
        grep -q "^## $version, [0-9]" CHANGELOG.md || die "CHANGELOG.md does not date $version"

        system=$(nix eval --impure --raw --expr builtins.currentSystem)
        nix build --no-link ".#checks.$system.hakmem-package" ".#checks.$system.hakmem-msrv"
        checked=$(nix build --no-link --print-out-paths .#hakmem-crate)

        work=$(mktemp -d)
        trap 'rm -rf "$work"' EXIT
        cargo package --no-verify --locked --target-dir "$work/target"
        mkdir "$work/checked" "$work/upload"
        tar xzf "$checked"/hakmem-*.crate -C "$work/checked"
        tar xzf "$work/target/package/hakmem-$version.crate" -C "$work/upload"
        diff -r --exclude=.cargo_vcs_info.json "$work/checked" "$work/upload" \
          || die "the tarball to upload is not the one the checks passed"

        # HAKMEM_PUBLISH_DRY_RUN=1: every check, then cargo's dry run.
        cargo publish --locked ''${HAKMEM_PUBLISH_DRY_RUN:+--dry-run}
      '';
    };
  in {
    packages.package-contents = contents;
    checks.hakmem-package = pkgs.runCommand "hakmem-package-check" {} ''
      if ! diff -u ${../release/package.txt} ${contents}; then
        echo "The files cargo publish would upload changed; if on purpose: nix build .#package-contents && cp result release/package.txt" >&2
        exit 1
      fi
      head=$(grep -m1 '^## ' ${../CHANGELOG.md})
      case "$head" in
        "## ${version}, "*) ;;
        *)
          echo "CHANGELOG.md's newest heading is '$head', Cargo.toml says ${version}" >&2
          exit 1
          ;;
      esac
      touch $out
    '';
    apps.publish = {
      type = "app";
      program = lib.getExe publish;
      meta.description = "Upload hakmem to crates.io from a tagged, checked tree";
    };
  };
}
