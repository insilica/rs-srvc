{
  description = "SysRev Version Control";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
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
        srvc = pkgs.callPackage ./srvc.nix { self-rev = self-rev; };
      in with pkgs; {
        packages.default = srvc;
        packages = { inherit docs-html srvc; };
        devShells.default = mkShell {
          buildInputs = [
            cargo
            entr
            git
            libiconv
            nixfmt-classic
            rustc
            rustfmt
            sphinx
            sphinx-autobuild
          ] ++ (with python3Packages; [ sphinx-rtd-theme ])
            ++ (if stdenv.isDarwin then [
              darwin.apple_sdk.frameworks.CoreServices
              darwin.apple_sdk.frameworks.Security
            ] else
              [ ]);

          SELF_REV = self-rev;
        };
      });

}
