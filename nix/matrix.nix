# The CI matrix as data. Dimension × dimension = a list of cells
# (`lib.cartesianProduct`), one crane derivation in `checks` per cell and
# kind. `nix flake check` is the reduce; cachix turns the whole fold into a
# memo, so an unchanged cell is a cache hit. The GitHub workflow reads the
# check names at run time (`nix eval .#checks.<system>`) and expands them
# into a dynamic matrix: the YAML never knows the matrix and cannot drift
# from it.
{lib, ...}: {
  perSystem = {
    config,
    system,
    ...
  }: let
    inherit (config.rust) craneLib;
    inherit (config.hakmem) keep version;
    inherit (lib) concatMapStringsSep concatStringsSep nameValuePair listToAttrs filter;

    # README.md enters the crate through `include_str!`, and proptest's
    # saved failures must pass too, or CI never replays them and they fail
    # again for the first time. Nothing else: an edit in docs/ or
    # CHANGELOG.md must not change a single derivation.
    src = config.hakmem.src [keep.readme (keep.suffix ".proptest-regressions")];

    # A hardware cell from the CPU features it builds for. A builder
    # without one of them would die with SIGILL; the guard fails first,
    # with a sentence.
    hw = name: features: let
      plus = concatMapStringsSep "," (f: "+" + f) features;
      has = features |> map (f: "grep -qw ${f} /proc/cpuinfo") |> concatStringsSep " && ";
    in {
      inherit name;
      label =
        if features == []
        then name
        else plus;
      rustflags = lib.optionalString (features != []) "-C target-feature=${plus}";
      guard = lib.optionalString (features != []) ''
        ${has} \
          || { echo "builder CPU lacks ${concatStringsSep "/" features}; cannot run this cell" >&2; exit 1; }
      '';
    };

    dims = {
      hw =
        [(hw "portable" [])]
        # AVX2 rides along: the 3D Hilbert batch kernel is its only user,
        # and every runner has it.
        ++ lib.optional (lib.hasPrefix "x86_64" system) (hw "bmi2" ["bmi2" "pclmulqdq" "ssse3" "avx2"]);
      features = [
        {
          name = "default";
          args = "";
          label = "";
        }
        {
          name = "portable-feature";
          args = "--features portable";
          label = ", feature portable";
        }
        {
          # `alloc` is on by default; this is the crate an embedded
          # user gets, with nothing that allocates.
          name = "no-default";
          args = "--no-default-features";
          label = ", no default features (no alloc)";
        }
      ];
    };

    # hw `portable` × feature `portable` is the same path twice; skip it.
    combos =
      lib.cartesianProduct dims
      |> filter (c: !(c.hw.name == "portable" && c.features.name == "portable-feature"));
    cellName = c: "${c.hw.name}-${c.features.name}";

    common = {
      inherit src version;
      pname = "hakmem";
      strictDeps = true;
      cargoExtraArgs = "";
      # Release: the exhaustive sweeps are `ignore`d in debug builds.
      CARGO_PROFILE = "release";
    };
    depsFor = c:
      craneLib.buildDepsOnly (common
        // {
          pname = "hakmem-deps-${cellName c}";
          cargoExtraArgs = c.features.args;
          RUSTFLAGS = c.hw.rustflags;
        });
    # What the site calls a kind, where the check name says it shorter.
    noun.test = "tests";
    # What every kind of cell passes crane, named after the kind.
    cell = kind: c: extra:
      common
      // {
        pname = "hakmem-${kind}-${cellName c}";
        cargoArtifacts = depsFor c;
        cargoExtraArgs = c.features.args;
        RUSTFLAGS = c.hw.rustflags;
        meta.description = "${noun.${kind} or kind}, ${c.hw.label}${c.features.label}";
      }
      // extra;

    # nextest: one process per test (the exhaustive sweeps stay isolated),
    # better output. It cannot run doctests; `doctest` below does.
    kinds = {
      test = c:
        craneLib.cargoNextest (cell "test" c {
          preCheck = c.hw.guard;
          partitions = 1;
          partitionType = "count";
        });
      clippy = c: craneLib.cargoClippy (cell "clippy" c {cargoClippyExtraArgs = "--all-targets -- -D warnings";});
    };
    # The doctests (the README) along the hw dimension only; the `portable`
    # feature does not change them.
    doctest = c:
      craneLib.cargoTest (cell "doctest" c {
        cargoTestExtraArgs = "--doc";
        preCheck = c.hw.guard;
        meta.description = "doctests (the README), ${c.hw.label}";
      });

    # One cell in debug: the `debug_assert!`s and the overflow checks run
    # nowhere else, and the README tells users to run their tests this
    # way. The exhaustive sweeps skip themselves here, which is the one
    # thing the release cells already do better.
    debugTest = let
      dev = common // {CARGO_PROFILE = "dev";};
    in
      craneLib.cargoNextest (dev
        // {
          pname = "hakmem-test-debug";
          cargoArtifacts = craneLib.buildDepsOnly (dev // {pname = "hakmem-deps-debug";});
          partitions = 1;
          partitionType = "count";
          meta.description = "tests, debug build (debug asserts, overflow checks)";
        });
  in {
    checks =
      (kinds
        |> lib.mapAttrsToList (kind: f: combos |> map (c: nameValuePair "hakmem-${kind}-${cellName c}" (f c)))
        |> lib.concatLists
        |> listToAttrs)
      // (combos
        |> filter (c: c.features.name == "default")
        |> map (c: nameValuePair "hakmem-doctest-${c.hw.name}" (doctest c))
        |> listToAttrs)
      // {
        hakmem-test-debug = debugTest;
        # rustdoc as docs.rs will see it: no dependencies, warnings are errors.
        hakmem-doc = craneLib.cargoDoc (common
          // {
            pname = "hakmem-doc";
            cargoArtifacts = depsFor (lib.head combos);
            cargoDocExtraArgs = "--no-deps";
            RUSTDOCFLAGS = "-D warnings --cfg docsrs";
            meta.description = "rustdoc, warnings as errors, as docs.rs builds it";
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
