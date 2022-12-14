{ lib, rustPlatform, stdenv, Security }:

rustPlatform.buildRustPackage rec {
  pname = "srvc";
  version = "0.9.0";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  buildInputs = lib.optionals stdenv.isDarwin [ Security ];

  doCheck = false;

  meta = { mainProgram = "sr"; };
}
