python-setup-env:
  uv venv && source .venv/bin/activate && uv pip install maturin

python-setup-env-fish:
  uv venv && source .venv/bin/activate.fish && uv pip install maturin

python-build:
  maturin develop && cargo run -p sdk-python-stubs && cp python/sdk/init_manual_override.pyi python/sdk/__init__.pyi

node-build:
  cd ./npm && npm install && npm run build && npm run test && cd ..
