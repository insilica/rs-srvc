<!-- Based on https://github.com/BurntSushi/ripgrep/blob/87b33c96c02b5d728324632956d301ef3d234f80/RELEASE-CHECKLIST.md -->

Release Checklist
-----------------
* Ensure local `master` is up to date with respect to `origin/master`.
* Run `cargo update` and review dependency updates.
* Run `nix flake update`.
* Run `cargo test`.
* Commit updated `Cargo.lock` and `flake.lock`.
* Edit `Cargo.toml` and `srvc.nix` to set the new version.
* Update the CHANGELOG as appropriate.
* Push changes to GitHub, NOT including the tag.
* Once CI for `master` finishes successfully, push the version tag.
* Wait for CI to finish creating the release. If the release build fails, then
  delete the tag from GitHub, make fixes, re-tag, delete the release and push.
* Copy the relevant section of the CHANGELOG to the tagged release notes.
* Run `ci/sha256-releases {VERSION}`. Then edit
  `srvc.rb` in the homebrew-srvc repo to update the version number and sha256 hashes.
  Commit changes and push to homebrew-srvc.
* Wait for CI to complete successfully.
