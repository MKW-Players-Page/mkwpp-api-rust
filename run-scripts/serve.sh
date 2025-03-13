#!/usr/bin/env bash

cargo build --release
mv target/release/mkwpp-api-rust ./mkwpp-api-rust
cargo clean # to not take space in the server
./mkwpp-api-rust