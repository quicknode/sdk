# Quicknode SDK

Quicknode SDK making it easy to use Quicknode products

Quicknode SDK is a Rust SDK with Python and Node.js bindings.

## Project Structure

```
my-sdk/
├── crates/
│   ├── core/          # Pure Rust business logic
│   ├── python/        # PyO3 bindings
│   └── node/          # napi-rs bindings
├── python/my_sdk/     # Python package with type hints
├── npm/               # Node.js package with TypeScript types
└── pyproject.toml     # maturin build config
```

## Installation

**Python:** `uv add my-sdk`

**Node.js:** `npm install my-sdk`

## Usage

**Python:**
```python
import my_sdk
my_sdk.add(2, 3)        # 5
my_sdk.divide(10, 2)    # 5.0
```

**TypeScript:**
```typescript
import { add, divide } from 'my-sdk';
add(2, 3);        // 5
divide(10, 2);    // 5.0
```

## Development

### Prerequisites

- Rust (stable)
- Python 3.8+ with [uv](https://docs.astral.sh/uv/)
- Node.js 18+

### Build Commands

```bash
# Core library
cargo test -p my-sdk-core

# Python (from project root)
uv venv && source .venv/bin/activate
uv pip install maturin
maturin develop

# Node.js (from npm/)
npm install && npm run build && npm run test
```

## License

MIT
