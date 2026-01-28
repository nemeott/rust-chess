# Uses the dev profile, which is optimized for build speed (under 30s)
cargo run --bin stub_gen # Automatically generate the stub file first
maturin build --profile dev
