python-setup-env:
  UV_VENV_CLEAR=1 uv venv --python 3.14 .venv && uv pip install maturin

python-build:
  maturin develop && cargo run -p sdk-python-stubs && cp python/sdk/init_manual_override.pyi python/sdk/__init__.pyi

node-build:
  npm install && npm run build && npm run test
