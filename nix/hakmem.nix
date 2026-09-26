# What the other modules share, as per-system options the way nix/rust.nix
# shares the toolchains: the crate's version, its sources cut to what a
# derivation reads, and snapshots.
#
# A snapshot is a file in the repository that must equal one a derivation
# builds: the public API, the instruction counts, the package's file list,
# the workflows. Each module declares its own under `hakmem.snapshots`, and
# from all of them come one check each and one way to take a change on
# purpose:
#
#   nix run .#snapshots                 write every snapshot of this system
#   nix run .#snapshots -- NAME...      those only
{
  lib,
  flake-parts-lib,
  ...
}: let
  inherit (flake-parts-lib) mkPerSystemOption;
  inherit (lib) mkOption types;
in {
  options.perSystem = mkPerSystemOption ({
    config,
    pkgs,
    ...
  }: let
    cfg = config.hakmem;

    # Predicates over a source path, for `src`.
    cargo = config.rust.craneLib.filterCargoSources;
    named = name: path: _: baseNameOf path == name;
    suffix = s: path: _: lib.hasSuffix s (baseNameOf path);

    # One `diff` per file; `|>` cannot sit inside the interpolation below.
    compare = s:
      s.files
      |> lib.mapAttrsToList (path: built: "diff -u ${./.. + "/${path}"} ${built} || same=false")
      |> lib.concatLines;

    snapshot = types.submodule ({name, ...}: {
      options = {
        files = mkOption {
          type = types.attrsOf types.str;
          description = "Repository path to the built file it must equal.";
        };
        why = mkOption {
          type = types.str;
          description = "What a difference means, said when the check fails.";
        };
        description = mkOption {
          type = types.str;
          default = name;
          description = "One line for the check's meta, which the site shows.";
        };
        inputs = mkOption {
          type = types.listOf types.package;
          default = [];
          description = "Tools `before` and `after` run.";
        };
        before = mkOption {
          type = types.lines;
          default = "";
          description = "Shell run before the comparison, for what the files alone do not hold.";
        };
        after = mkOption {
          type = types.lines;
          default = "";
          description = "Shell run once the files compare equal.";
        };
      };
    });
  in {
    options.hakmem = {
      version = mkOption {
        type = types.str;
        readOnly = true;
        default = (lib.importTOML ../Cargo.toml).package.version;
        description = "The crate's version, from Cargo.toml.";
      };
      src = mkOption {
        type = types.functionTo types.path;
        readOnly = true;
        # Named `source` whatever the filter: the same files give the same
        # store path, so a module does not rebuild for how it asked.
        default = extra:
          lib.cleanSourceWith {
            src = ./..;
            name = "source";
            filter = path: type: lib.any (keep: keep path type) ([cargo] ++ extra);
          };
        description = "The Cargo sources and whatever else the predicates in the argument keep.";
      };
      keep = mkOption {
        type = types.raw;
        readOnly = true;
        default = {
          inherit named suffix;
          readme = named "README.md";
        };
        description = "Predicates for `src`.";
      };
      snapshots = mkOption {
        type = types.attrsOf snapshot;
        default = {};
        description = "Files that must equal what a derivation builds; the name is the check's.";
      };
    };

    config = {
      checks =
        cfg.snapshots
        |> lib.mapAttrs (name: s:
          pkgs.runCommand name {
            nativeBuildInputs = s.inputs;
            meta.description = s.description;
          } ''
            ${s.before}
            same=true
            ${compare s}
            if ! $same; then
              echo ${lib.escapeShellArg s.why} >&2
              echo "If on purpose: nix run .#snapshots -- ${name}" >&2
              exit 1
            fi
            ${s.after}
            touch $out
          '');

      # The files of each snapshot at their repository paths, built only
      # when asked for: the app below names the ones it writes.
      legacyPackages.snapshot =
        cfg.snapshots
        |> lib.mapAttrs (name: s:
          s.files
          |> lib.mapAttrsToList (path: built: {
            name = path;
            path = built;
          })
          |> pkgs.linkFarm "snapshot-${name}");

      apps.snapshots = {
        type = "app";
        meta.description = "Write the snapshots (all of this system's, or the ones named) into the repository";
        program = lib.getExe (pkgs.writeShellApplication {
          name = "hakmem-snapshots";
          runtimeInputs = [pkgs.coreutils pkgs.gnugrep];
          text = ''
            grep -qs '^name = "hakmem"' Cargo.toml || { echo "run it from the repository root" >&2; exit 1; }
            system=$(nix eval --impure --raw --expr builtins.currentSystem)
            names=("$@")
            [ ''${#names[@]} -gt 0 ] || names=(${lib.escapeShellArgs (builtins.attrNames cfg.snapshots)})
            for name in "''${names[@]}"; do
              out=$(nix build --no-link --print-out-paths ".#legacyPackages.$system.snapshot.$name")
              cp -rL --no-preserve=mode,ownership "$out/." .
              echo "snapshot $name written"
            done
          '';
        });
      };
    };
  });
}
