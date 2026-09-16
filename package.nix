{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "project";
  version = "0.1.0";

  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "A simple Rust command-line application";
    license = lib.licenses.mit;
    mainProgram = "project";
    platforms = lib.platforms.unix;
  };
}
