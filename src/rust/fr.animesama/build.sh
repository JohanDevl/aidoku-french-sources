#!/bin/bash
cargo +nightly build --target wasm32-unknown-unknown --release
mkdir -p target/wasm32-unknown-unknown/release/Payload
cp res/* target/wasm32-unknown-unknown/release/Payload
cp target/wasm32-unknown-unknown/release/*.wasm target/wasm32-unknown-unknown/release/Payload/main.wasm
cd target/wasm32-unknown-unknown/release && zip -r package.aix Payload
mv package.aix ../../../package.aix
cd ../../..