#!/bin/bash

if [[ -z "$1" ]]; then
    cargo run --release
elif [[ "$1" == "profile" ]]; then
    cargo run --release --features bevy/trace_chrome
elif [[ "$1" == "build-web" ]]; then
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/wave_sim.wasm
elif [[ "$1" == "run-web" ]]; then
    python3 -m http.server 8000
fi
