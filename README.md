# Sunaba

## Stack
- Linux
- Docker
- Rust

## Install
```bash
# Build
cargo build --release

# Install
install -m 755 target/release/sunaba "$HOME/.local/bin/sunaba"

# Verify
which sunaba
```

## Inspector
```bash
# Run:
npx @modelcontextprotocol/inspector

# UI:
http://localhost:6274
```
