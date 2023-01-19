{ pkgs ? import <nixpkgs> { }, self-rev ? null }:
let target = pkgs.stdenv.targetPlatform;
in with pkgs;
mkShell {
  buildInputs =
    [ cargo entr git libiconv nixfmt rustc rustfmt sphinx sphinx-autobuild ]
    ++ (with python3Packages; [ sphinx-rtd-theme ]) ++ (if target.isDarwin then
      [ darwin.apple_sdk.frameworks.Security ]
    else
      [ ]);

  SELF_REV = self-rev;
}
