{ lib, rustPlatform, stdenv, darwin, self-rev ? null }:

rustPlatform.buildRustPackage rec {
  pname = "srvc";
  version = "0.16.0";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  buildInputs = lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.CoreServices
    darwin.apple_sdk.frameworks.Security
  ];

  doCheck = false;

  SELF_REV = self-rev;

  meta = { mainProgram = "sr"; };
}
