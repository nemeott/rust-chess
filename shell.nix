{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = with pkgs; [
    # Rust
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
    mold # Linker

    # Python
    python313
    uv

    # Maturin
    maturin
    clang
  ];
}
