$env:RUSTFLAGS="-C target-feature=+simd128";
wasm-pack build wasm --release --target bundler