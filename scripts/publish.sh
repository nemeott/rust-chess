# Publish: maturin publish
# - Username: __token__
# - Password: pypi-API_TOKEN_HERE

set -e

PYPI_TOKEN="$(cat ../.pypi_token)"

cargo run --bin stub_gen -r # Automatically generate the stub file first

# TODO: manylinux_2_28 (nix shell?)

unset _PYTHON_HOST_PLATFORM # Unset to generate PyPI compatiable version (manylinux default)
maturin publish \
    --compatibility pypi \
    --username __token__ \
    --password pypi-$PYPI_TOKEN

# --zig \
