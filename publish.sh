# Publish: maturin publish
# - Username: __token__
# - Password: pypi-API_TOKEN_HERE

set -e

PYPI_TOKEN="$(cat ../.pypi_token)"

cargo run --bin stub_gen # Automatically generate the stub file first

unset _PYTHON_HOST_PLATFORM # Unset to generate PyPI compatiable version (manylinux default)
maturin publish \
    --compatibility manylinux_2_28 \
    --username __token__ \
    --password pypi-$PYPI_TOKEN

# --zig \
