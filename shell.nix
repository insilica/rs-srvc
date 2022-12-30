{ pkgs ? import <nixpkgs> { } }:
let target = pkgs.stdenv.targetPlatform;
in with pkgs;
mkShell {
  buildInputs = [ cargo git niv nixfmt rustc rustfmt webfs ]
    ++ (if target.isDarwin then
      [ darwin.apple_sdk.frameworks.Security ]
    else
      [ ]);
}
