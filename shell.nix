{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = with pkgs; [
    rustc
    rust-analyzer
    cargo
    rustfmt
    clippy

    python313
    python313Packages.pip
    uv
    maturin
    clang # Needed by maturin
    mold # Linker
  ];
}
