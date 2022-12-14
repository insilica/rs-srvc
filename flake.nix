# See https://dev.to/johnreillymurray/rust-environment-and-docker-build-with-nix-flakes-19c1
{
  description = "SysRev Version Control";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs =
          nixpkgs.legacyPackages.${system}; # import nixpkgs { inherit system; };
      in with pkgs;
      let
        srvc = pkgs.callPackage ./srvc.nix {
          Security = lib.optionals stdenv.isDarwin Security;
        };
      in {
        defaultPackage = srvc;
        packages = { inherit srvc; };
      });

}
