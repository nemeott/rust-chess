{
  pkgs ? import <nixpkgs> { },
}:

let
  # Rust nightly (for parallel building)
  fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") {
    inherit pkgs;
  };

  rustToolchain = fenix.complete.withComponents [
    "cargo"
    "clippy"
    "rust-src"
    "rustc"
    "rustfmt"
    "rust-analyzer"
  ];
in
pkgs.mkShell {
  packages = with pkgs; [
    rustToolchain

    python313
    python313Packages.pip
    uv
    maturin
    clang # Needed by maturin
    mold # Linker
  ];
}
