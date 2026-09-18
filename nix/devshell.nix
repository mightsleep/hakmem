# Dev shell: the nightly toolchain (rustfmt.toml uses nightly options,
# clippy) plus cargo tooling. The MSRV is checked by the `hakmem-msrv`
# derivation, not here; the hook only prints it.
{
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    devShells.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        config.rust.nightly
        cargo-nextest
        cargo-semver-checks
        cargo-fuzz
      ];
      shellHook = ''
        echo "hakmem dev shell, $(rustc --version); MSRV ${config.rust.msrvVersion}"
      '';
    };
  };
}
