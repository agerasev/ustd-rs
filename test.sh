#!/bin/sh

git submodule init && \
cd freertos-rust && git submodule update --init freertos-rust-examples/FreeRTOS-Kernel && cd .. && \
cd tests && \
cargo test --lib --no-default-features --features=std && \
cargo run --no-default-features --features=freertos && \
echo "" && \
echo "Success!"
