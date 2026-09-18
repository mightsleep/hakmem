# Target systems. The hardware-specific dimensions of the CI matrix (BMI2,
# PCLMULQDQ) are enabled on x86_64 only, see nix/matrix.nix.
{
  systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
}
