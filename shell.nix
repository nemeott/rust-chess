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
    uv
    maturin
    clang # Needed by maturin
    mold # Linker
  ];
}
