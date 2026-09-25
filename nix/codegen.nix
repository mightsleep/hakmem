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
  in
    lib.optionalAttrs (system == "x86_64-linux") {
      packages.codegen-bmi2 = bmi2;
      checks.hakmem-codegen-bmi2 = check "bmi2" "CHECK,BMI2" bmi2;
      packages.codegen-portable = portable;
      checks.hakmem-codegen-portable = check "portable" "CHECK,PORTABLE" portable;
    };
}
