# Uses the dev profile, which is optimized for build speed (under 5s if cached)
cargo run --bin stub_gen # Automatically generate the stub file first
maturin develop --profile dev --uv
