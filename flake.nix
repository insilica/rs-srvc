{
  description = "SysRev Version Control";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in with pkgs;
      let
        srvc = callPackage ./srvc.nix {
          Security = lib.optionals stdenv.isDarwin Security;
        };
      in with pkgs; {
        packages.default = srvc;
        packages = { inherit srvc; };
        devShells.default = mkShell {
          buildInputs = [ cargo git niv nixfmt rustc rustfmt webfs ];
        };
      });

}
