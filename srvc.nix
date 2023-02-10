{ lib, rustPlatform, stdenv, Security, self-rev ? null }:

rustPlatform.buildRustPackage rec {
  pname = "srvc";
  version = "0.14.0";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  buildInputs =
    lib.optionals stdenv.isDarwin [ Security ] ;

  doCheck = false;

  SELF_REV = self-rev;

  meta = { mainProgram = "sr"; };
}
