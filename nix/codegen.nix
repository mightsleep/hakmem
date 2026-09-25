# What the compiler makes of the kernels: codegen/src/lib.rs says which
# instructions a build must (or must not) contain, FileCheck holds the asm
# to it, and the count of each mnemonic per function is checked in next to
# it, like public-api/. After a deliberate change:
#
#   nix build .#codegen-bmi2 && cp result/x86_64-linux-bmi2.txt codegen/
#
# Compile only, nothing runs, so a cell may ask for instructions no CI
# runner has. The asm is kept in the output for whoever wants to read it.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    ...
  }: let
    inherit (config.rust) craneLib;
    llvm = pkgs.llvmPackages.llvm;
    src = lib.cleanSourceWith {
      src = ./..;
      filter = path: type:
        craneLib.filterCargoSources path type
        || baseNameOf path == "README.md"
        || lib.hasSuffix ".awk" path;
      name = "source";
    };
    # One build of hakmem and the wrappers, asm of both, cut into the
    # functions lib.rs names.
    cell = name: rustflags:
      pkgs.runCommand "hakmem-codegen-${name}" {
        nativeBuildInputs = [config.rust.nightly llvm pkgs.stdenv.cc];
      } ''
        cp -r ${src} src && chmod -R u+w src && cd src/codegen
        export HOME=$TMPDIR CARGO_TARGET_DIR=$TMPDIR/target RUSTFLAGS=${lib.escapeShellArg rustflags}
        # hakmem's asm holds the kernels, which are not generic; the
        # wrappers' holds everything monomorphised at the call.
        cargo rustc --offline --release --lib -p hakmem -- --emit=asm -C codegen-units=1
        cargo rustc --offline --release --lib -- --emit=asm -C codegen-units=1
        mkdir -p $out
        find $CARGO_TARGET_DIR -name '*.s' -print0 | xargs -0 cat | llvm-cxxfilt > $out/all.s
        awk -f functions.awk src/lib.rs $out/all.s > $out/functions.s
        awk -f mnemonics.awk $out/functions.s > $out/${system}-${name}.txt
        cp src/lib.rs $out/claims.rs
      '';
    check = name: prefixes: out:
      pkgs.runCommand "hakmem-codegen-${name}-check" {nativeBuildInputs = [llvm];} ''
        FileCheck --check-prefixes=${prefixes} --input-file=${out}/functions.s ${out}/claims.rs
        if ! diff -u ${../codegen + "/${system}-${name}.txt"} ${out}/${system}-${name}.txt; then
          echo "The instruction counts changed; if on purpose: nix build .#codegen-${name} && cp result/${system}-${name}.txt codegen/" >&2
          exit 1
        fi
        touch $out
      '';
    bmi2 = cell "bmi2" "-C target-feature=+bmi2,+pclmulqdq,+ssse3,+avx2";
    # No flags: what a dependency gets by default, and where the tokens
    # have to earn their keep.
    portable = cell "portable" "";

    # The cells above see one codegen unit and one caller per wrapper, and
    # there everything inlines. The benches are a crate that calls hakmem
    # the way a user's does: a release build, sixteen codegen units, the
    # same kernel from several places. Their linked binaries are where a
    # lost `#[inline]` shows, as a hakmem function that should not exist;
    # `<u64 as Word>::pext` did, and the 2D Hilbert decode was eight times
    # slower. The snapshot is the list of those functions per bench
    # (codegen/outlined.awk). After a deliberate change:
    #
    #   nix build .#codegen-outlined-portable && cp result/x86_64-linux-outlined-portable.txt codegen/
    benchSrc = lib.cleanSourceWith {
      src = ./..;
      filter = path: type:
        craneLib.filterCargoSources path type
        || baseNameOf path == "README.md";
      name = "source";
    };
    benchArgs = rustflags: {
      src = benchSrc;
      pname = "hakmem";
      version = (craneLib.crateNameFromCargoToml {cargoToml = ../Cargo.toml;}).version;
      strictDeps = true;
      CARGO_PROFILE = "release";
      RUSTFLAGS = rustflags;
      # Source lines for annotated.s. Line tables do not move an inlining
      # decision: the list came out the same without them.
      CARGO_PROFILE_RELEASE_DEBUG = "line-tables-only";
      CARGO_PROFILE_BENCH_DEBUG = "line-tables-only";
    };
    outlined = name: rustflags:
      craneLib.mkCargoDerivation (benchArgs rustflags
        // {
          pname = "hakmem-codegen-outlined-${name}";
          cargoArtifacts = craneLib.buildDepsOnly (benchArgs rustflags // {pname = "hakmem-deps-outlined-${name}";});
          nativeBuildInputs = [llvm pkgs.jq];
          doInstallCargoArtifacts = false;
          buildPhaseCargoCommand = ''
            cargo build --release --benches --message-format json-render-diagnostics > build.json
          '';
          # `--benches` brings the lib's unit tests too; only the benches
          # are a user.
          installPhaseCommand = ''
            mkdir -p $out
            jq -r 'select(.target.kind == ["bench"] and .executable != null) | "\(.target.name) \(.executable)"' build.json \
              | while read -r bench bin; do
                  awk -f ${../codegen/got.awk} <(llvm-nm -C --defined-only "$bin") <(llvm-objdump -R "$bin") > got
                  llvm-objdump -d -l --no-show-raw-insn --no-leading-addr -C "$bin" \
                    | awk -v bench="$bench" -v dir=$out -v got=got -f ${../codegen/outlined.awk}
                done
            sort $out/outlined | uniq -c \
              | awk '{ c = $1; sub(/^ *[0-9]+ /, ""); print (c > 1 ? $0 " ×" c : $0) }' \
              > $out/${system}-outlined-${name}.txt
            sort -t$'\t' -k1,1 -k2,2nr $out/sizes -o $out/sizes
            rm $out/outlined
          '';
        });
    outlinedCheck = name: out:
      pkgs.runCommand "hakmem-codegen-outlined-${name}-check" {} ''
        if ! diff -u ${../codegen + "/${system}-outlined-${name}.txt"} ${out}/${system}-outlined-${name}.txt; then
          echo "A hakmem function appeared in or left the benches' binaries: + is a lost inlining unless meant." >&2
          echo "If on purpose: nix build .#codegen-outlined-${name} && cp result/${system}-outlined-${name}.txt codegen/" >&2
          exit 1
        fi
        touch $out
      '';
    outlinedPortable = outlined "portable" "";
    # A fixed level instead of the benches' `target-cpu=native`: the same
    # list on every builder.
    outlinedV3 = outlined "v3" "-C target-cpu=x86-64-v3";
  in
    lib.optionalAttrs (system == "x86_64-linux") {
      packages.codegen-bmi2 = bmi2;
      checks.hakmem-codegen-bmi2 = check "bmi2" "CHECK,BMI2" bmi2;
      packages.codegen-portable = portable;
      checks.hakmem-codegen-portable = check "portable" "CHECK,PORTABLE" portable;
      packages.codegen-outlined-portable = outlinedPortable;
      checks.hakmem-codegen-outlined-portable = outlinedCheck "portable" outlinedPortable;
      packages.codegen-outlined-v3 = outlinedV3;
      checks.hakmem-codegen-outlined-v3 = outlinedCheck "v3" outlinedV3;
    };
}
