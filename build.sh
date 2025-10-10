#!/bin/bash
set -e

echo "Building backend..."
cargo build --release --bin clock-it --no-default-features --features ssr

echo "Building frontend WASM..."
cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate --profile wasm-release --target-dir ./target/front

echo "Running wasm-bindgen..."
# Don't reinstall, assume it's already installed in Docker
wasm-bindgen --target web \
    --no-typescript \
    --out-dir target/site/pkg \
    --out-name clock-it \
    target/front/wasm32-unknown-unknown/wasm-release/clock_it.wasm

echo "Optimizing WASM..."
wasm-opt --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals -Oz \
    target/site/pkg/clock-it_bg.wasm \
    -o target/site/pkg/clock-it_bg.wasm

echo "Compiling SCSS..."
sass style/main.scss target/site/pkg/clock-it.css

echo "Copying binary and assets..."
cp target/release/clock-it ./clock-it
[ -d "public" ] && cp -r public/* target/site/ 2>/dev/null || true

echo "✅ Build complete!"