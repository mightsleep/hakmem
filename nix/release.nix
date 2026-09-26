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
# A version already on crates.io with the same checksum is a success, so
# a rerun of the release goes through. The release workflow runs it on
# pull requests too, as a rehearsal (HAKMEM_PUBLISH_REHEARSAL=1).
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
      runtimeInputs = [config.rust.nightly pkgs.git pkgs.gnutar pkgs.gzip pkgs.diffutils pkgs.curl pkgs.jq];
      text = ''
        die() { echo "publish: $*" >&2; exit 1; }
        # HAKMEM_PUBLISH_REHEARSAL=1 (the release workflow outside a tag):
        # a dry run in which what only a tagged commit can satisfy is
        # reported instead of fatal. Everything else stops it as it would
        # stop a release.
        rehearsal=''${HAKMEM_PUBLISH_REHEARSAL:-}
        [ -z "$rehearsal" ] || HAKMEM_PUBLISH_DRY_RUN=1
        refuse() {
          [ -n "$rehearsal" ] || die "$@"
          echo "publish: rehearsal, a release would stop here: $*" >&2
        }
        version=${version}
        grep -qs '^name = "hakmem"' Cargo.toml || die "run it from the repository root"
        [ -n "''${CARGO_REGISTRY_TOKEN:-}''${HAKMEM_PUBLISH_DRY_RUN:-}" ] || die "CARGO_REGISTRY_TOKEN is not set"
        dirty=$(git status --porcelain)
        [ -z "$dirty" ] || die "the tree has uncommitted changes:
        $dirty"
        [ "$(git describe --exact-match --tags HEAD 2>/dev/null)" = "v$version" ] || refuse "HEAD is not tagged v$version"
        grep -q "^## $version, [0-9]" CHANGELOG.md || refuse "CHANGELOG.md does not date $version"

        system=$(nix eval --impure --raw --expr builtins.currentSystem)
        nix build --no-link ".#checks.$system.hakmem-package" ".#checks.$system.hakmem-msrv"
        checked=$(nix build --no-link --print-out-paths .#hakmem-crate)

        work=$(mktemp -d)
        trap 'git worktree remove --force "$work/again" 2>/dev/null; rm -rf "$work"' EXIT
        cargo package --no-verify --locked --target-dir "$work/target"
        mkdir "$work/checked" "$work/upload"
        tar xzf "$checked"/hakmem-*.crate -C "$work/checked"
        tar xzf "$work/target/package/hakmem-$version.crate" -C "$work/upload"
        diff -r --exclude=.cargo_vcs_info.json "$work/checked" "$work/upload" \
          || die "the tarball to upload is not the one the checks passed"

        # A rerun after an upload that went through (a failed step later, or
        # cargo timing out on the index) finds the version on crates.io, and
        # passes if the checksum is the same. That needs cargo to pack a
        # commit to the same bytes wherever it is checked out (it fixes the
        # mtimes, and .cargo_vcs_info.json holds only the commit), so every
        # run checks it: a second checkout, at another path and with fresh
        # mtimes, must pack to the same file.
        git worktree add --quiet --detach "$work/again" HEAD
        cargo package --no-verify --locked --manifest-path "$work/again/Cargo.toml" --target-dir "$work/target-again"
        cmp "$work/target/package/hakmem-$version.crate" "$work/target-again/package/hakmem-$version.crate" \
          || die "cargo packs this commit to other bytes from a second checkout; a rerun could not recognise its own upload"
        sum=$(sha256sum "$work/target/package/hakmem-$version.crate" | cut -d' ' -f1)
        code=$(curl -s -A hakmem-publish -o "$work/version.json" -w '%{http_code}' \
          "https://crates.io/api/v1/crates/hakmem/$version")
        case $code in
          404)
            # HAKMEM_PUBLISH_DRY_RUN=1: every check, then cargo's dry run.
            cargo publish --locked ''${HAKMEM_PUBLISH_DRY_RUN:+--dry-run}
            ;;
          200)
            # In a rehearsal after a release, before the bump: another
            # commit, so another checksum, as it should be.
            if jq -e --arg s "$sum" '.version.checksum == $s' "$work/version.json" >/dev/null; then
              echo "publish: $version is on crates.io already, the same tarball"
            else
              refuse "$version is on crates.io with a different tarball"
            fi
            ;;
          *) die "crates.io answered $code for $version" ;;
        esac
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
