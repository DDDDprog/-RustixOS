#!/bin/bash
set -e

echo "Building kernel..."
cargo build --target x86_64-unknown-none

echo "Running in QEMU..."
qemu-system-x86_64 \
    -m 256 \
    -display none \
    -serial stdio \
    -kernel target/x86_64-unknown-none/debug/kernel
