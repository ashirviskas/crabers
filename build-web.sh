#!/usr/bin/env bash
set -euo pipefail

# Build wasm binary
cargo build --profile web --target wasm32-unknown-unknown

# Run wasm-bindgen to generate JS glue
wasm-bindgen \
    --out-dir web \
    --target web \
    target/wasm32-unknown-unknown/web/crabers.wasm

# Optimize wasm binary with wasm-opt (saves 30-50%)
if command -v wasm-opt &> /dev/null; then
    BEFORE=$(stat --format=%s web/crabers_bg.wasm)
    wasm-opt -O3 --all-features -o web/crabers_bg.wasm web/crabers_bg.wasm
    AFTER=$(stat --format=%s web/crabers_bg.wasm)
    echo "wasm-opt: ${BEFORE} -> ${AFTER} bytes ($(( (BEFORE - AFTER) * 100 / BEFORE ))% reduction)"
else
    echo "WARNING: wasm-opt not found. Install binaryen for smaller builds."
fi

# Symlink assets into web/ so the server can serve from web/ directory
ln -sfn ../assets web/assets

echo ""
echo "Build complete! Serve with:"
echo "  python3 -m http.server 8080 --directory web/"
