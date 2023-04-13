============
Installation
============

Homebrew (macOS and Linux)
========================================

Install homebrew_, and run:

.. _homebrew: https://brew.sh/

.. code-block:: shell

    brew install insilica/srvc/srvc

Nix (macOS and Linux)
===================================

Install Nix_, and run:

.. _nix: https://nixos.org/

.. code-block:: shell

    nix --extra-experimental-features "nix-command" profile install nixpkgs#srvc

Binary (macOS, Linux, Windows)
==================================

Download the binary for your platform from the releases_ page. Extract it, make
sure it is executable, and place it on your PATH.

.. _releases: https://github.com/insilica/rs-srvc/releases

From source (Cargo)
===================

Install the Rust language tools_, and run:

.. _tools: https://doc.rust-lang.org/cargo/getting-started/installation.html

.. code-block:: shell

    cargo install --git https://github.com/insilica/rs-srvc.git

Make sure that the directory that cargo installs to is on your PATH.

From source (Nix)
=================

.. code-block:: shell

    nix --extra-experimental-features "nix-command flakes" profile install github:insilica/rs-srvc
