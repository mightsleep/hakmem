# Formatting: `nix fmt` locally, the `treefmt` check in CI, the same
# derivations. rustfmt takes the nightly toolchain for the nightly options
# in rustfmt.toml.
{inputs, ...}: {
  imports = [inputs.treefmt-nix.flakeModule];
  perSystem = {config, ...}: {
    treefmt = {
      projectRootFile = "flake.nix";
      programs.rustfmt = {
        enable = true;
        package = config.rust.nightly;
      };
      programs.alejandra.enable = true;
    };
  };
}
