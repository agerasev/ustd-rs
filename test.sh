#!/bin/sh

git submodule update --init && \
cd freertos-rust && git submodule update --init freertos-rust-examples/FreeRTOS-Kernel && cd .. && \
cargo test --no-default-features --features=backend-std && \
cargo test --no-default-features --features=test-freertos && \
echo "Done!"
