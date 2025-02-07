#!/usr/bin/env bash

cargo build --release
mv target/release/mkwpp-api-rust ./mkwpp-api-rust
rm -rf target # to not take space in the server