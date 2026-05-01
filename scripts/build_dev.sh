# Uses the dev profile, which is optimized for build speed (under 5s if cached)
# No stub gen since we want speed
maturin build --profile dev
