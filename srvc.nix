{ lib, rustPlatform, stdenv, CoreServices, Security, self-rev ? null }:

rustPlatform.buildRustPackage rec {
  pname = "srvc";
  version = "0.14.1";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  buildInputs = lib.optionals stdenv.isDarwin [ CoreServices Security ];

  doCheck = false;

  SELF_REV = self-rev;

  meta = { mainProgram = "sr"; };
}
