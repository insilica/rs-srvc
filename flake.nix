{
  description = "SysRev Version Control";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        self-rev = if (self ? rev) then self.rev else "HEAD";
        docs-html = pkgs.callPackage ./docs/html.nix { inherit self-rev; };
        srvc = pkgs.callPackage ./srvc.nix {
          Security = with pkgs;
            lib.optionals stdenv.isDarwin
            [ darwin.apple_sdk.frameworks.Security ];
          self-rev = self-rev;
        };
      in with pkgs; {
        packages.default = srvc;
        packages = { inherit docs-html srvc; };
        devShells.default = mkShell {
          buildInputs = [
            cargo
            entr
            git
            libiconv
            nixfmt
            rustc
            rustfmt
            sphinx
            sphinx-autobuild
          ] ++ (with python3Packages; [ sphinx-rtd-theme ])
            ++ (if stdenv.isDarwin then
              [ darwin.apple_sdk.frameworks.Security ]
            else
              [ ]);

          SELF_REV = self-rev;
        };
      });

}
