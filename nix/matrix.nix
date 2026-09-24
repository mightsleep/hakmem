# The CI matrix as data. Dimension × dimension = a list of cells
# (`lib.cartesianProduct`), one crane derivation in `checks` per cell.
# `nix flake check` is the reduce; cachix turns the whole fold into a memo,
# so an unchanged cell is a cache hit. The GitHub workflow reads the check
# names at run time (`nix eval .#checks.<system>`) and expands them into a
# dynamic matrix: the YAML never knows the matrix and cannot drift from it.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    ...
  }: let
    inherit (config.rust) craneLib;
    isX86 = lib.hasPrefix "x86_64" system;

    # cleanCargoSource keeps only .rs and Cargo.*; README.md enters the crate
    # through `include_str!`, so it must pass too. Nothing else: an edit in
    # docs/ or CHANGELOG.md must not change a single derivation.
    src = lib.cleanSourceWith {
      src = ./..;
      filter = path: type: craneLib.filterCargoSources path type || baseNameOf path == "README.md";
      name = "source";
    };
    version = (craneLib.crateNameFromCargoToml {cargoToml = ../Cargo.toml;}).version;

    # --- dimensions ---------------------------------------------------------
    dims = {
      hw =
        [
          {
            name = "portable";
            rustflags = "";
            # The portable path must run everywhere: no guard.
            guard = "";
          }
        ]
        ++ lib.optionals isX86 [
          {
            name = "bmi2";
            # AVX2 rides along: the 3D Hilbert batch kernel is its only user,
            # and every runner has it.
            rustflags = "-C target-feature=+bmi2,+pclmulqdq,+ssse3,+avx2";
            # A builder without BMI2 would die with SIGILL; fail with a
            # readable message instead.
            guard = ''
              grep -qw bmi2 /proc/cpuinfo && grep -qw pclmulqdq /proc/cpuinfo && grep -qw ssse3 /proc/cpuinfo && grep -qw avx2 /proc/cpuinfo \
                || { echo "builder CPU lacks bmi2/pclmulqdq/avx2; cannot run this cell" >&2; exit 1; }
            '';
          }
        ];
      features = [
        {
          name = "default";
          args = "";
        }
        {
          name = "portable-feature";
          args = "--features portable";
        }
        {
          name = "alloc";
          args = "--features alloc";
        }
      ];
    };

    # hw `portable` × feature `portable` is the same path twice; skip it.
    combos =
      lib.filter (c: !(c.hw.name == "portable" && c.features.name == "portable-feature"))
      (lib.cartesianProduct dims);

    # --- derivations --------------------------------------------------------
    common = {
      inherit src version;
      pname = "hakmem";
      strictDeps = true;
      cargoExtraArgs = "";
      # Release: the exhaustive sweeps are `ignore`d in debug builds.
      CARGO_PROFILE = "release";
    };

    cellName = c: "${c.hw.name}-${c.features.name}";

    depsFor = c:
      craneLib.buildDepsOnly (common
        // {
          pname = "hakmem-deps-${cellName c}";
          cargoExtraArgs = c.features.args;
          RUSTFLAGS = c.hw.rustflags;
        });

    # nextest: one process per test (the exhaustive sweeps stay isolated),
    # better output. It cannot run doctests; `hakmem-doctest` below does.
    testFor = c:
      craneLib.cargoNextest (common
        // {
          pname = "hakmem-test-${cellName c}";
          cargoArtifacts = depsFor c;
          cargoExtraArgs = c.features.args;
          RUSTFLAGS = c.hw.rustflags;
          preCheck = c.hw.guard;
          partitions = 1;
          partitionType = "count";
        });

    doctestFor = c:
      craneLib.cargoTest (common
        // {
          pname = "hakmem-doctest-${cellName c}";
          cargoArtifacts = depsFor c;
          cargoExtraArgs = c.features.args;
          cargoTestExtraArgs = "--doc";
          RUSTFLAGS = c.hw.rustflags;
          preCheck = c.hw.guard;
        });

    clippyFor = c:
      craneLib.cargoClippy (common
        // {
          pname = "hakmem-clippy-${cellName c}";
          cargoArtifacts = depsFor c;
          cargoExtraArgs = c.features.args;
          cargoClippyExtraArgs = "--all-targets -- -D warnings";
          RUSTFLAGS = c.hw.rustflags;
        });

    baseline = lib.head combos;

    tests = lib.listToAttrs (map (c: lib.nameValuePair "hakmem-test-${cellName c}" (testFor c)) combos);
    clippies = lib.listToAttrs (map (c: lib.nameValuePair "hakmem-clippy-${cellName c}" (clippyFor c)) combos);
    # Doctests (the README) only along the hw dimension; the `portable`
    # feature does not change them.
    doctests =
      lib.listToAttrs (map (c: lib.nameValuePair "hakmem-doctest-${c.hw.name}" (doctestFor c))
        (lib.filter (c: c.features.name == "default") combos));
  in {
    checks =
      tests
      // clippies
      // doctests
      // {
        # rustdoc as docs.rs will see it: no dependencies, warnings are errors.
        hakmem-doc = craneLib.cargoDoc (common
          // {
            pname = "hakmem-doc";
            cargoArtifacts = depsFor baseline;
            cargoDocExtraArgs = "--no-deps";
            RUSTDOCFLAGS = "-D warnings --cfg docsrs";
          });
      };

    # The matrix as plain data (cell names, no derivations) for tooling
    # that wants the list without evaluating the checks.
    legacyPackages.hakmemMatrix = {
      inherit system;
      cells = map cellName combos;
    };
  };
}
