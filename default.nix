{ sources ? import ./nix/sources.nix, pkgs ? import sources.nixpkgs { } }:
with pkgs;
mkShell { buildInputs = [ cargo niv nixfmt rustc rustfmt ]; }
