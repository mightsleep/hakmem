# Target systems. The hardware-specific dimensions of the CI matrix (BMI2,
# PCLMULQDQ) are enabled on x86_64 only, see nix/matrix.nix. Linux only:
# CI has no darwin runner, and a system nothing checks is a promise.
{
  systems = ["x86_64-linux" "aarch64-linux"];
}
