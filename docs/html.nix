{ pkgs ? import <nixpkgs> { }, lib, rustPlatform, stdenv, self-rev ? null }:
stdenv.mkDerivation {
  name = "srvc-docs-html";

  src = ./.;

  nativeBuildInputs = with pkgs; [
    git
    python3Packages.sphinx-rtd-theme
    sphinx
  ];

  buildPhase = "make html";

  installPhase = ''
    mkdir -p $out
    cp -r _build/html $out
  '';

  SELF_REV = self-rev;
}
