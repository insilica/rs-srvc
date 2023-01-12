{
  description = "SysRev Version Control";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        srvc = pkgs.callPackage ./srvc.nix {
          Security = with pkgs;
            lib.optionals stdenv.isDarwin
            [ darwin.apple_sdk.frameworks.Security ];
            self-rev = if (self ? rev) then self.rev else null;
        };
      in with pkgs; {
        packages.default = srvc;
        packages = { inherit srvc; };
        devShells.default = import ./shell.nix {
          inherit pkgs;
          self-rev = if (self ? rev) then self.rev else null;
        };
      });

}
