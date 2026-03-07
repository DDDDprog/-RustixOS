#!/bin/bash
set -e

echo "Building kernel..."
cargo build --target x86_64-unknown-none

echo "Running in QEMU..."
# Try different boot methods
qemu-system-x86_64 \
    -m 256 \
    -display none \
    -serial stdio \
    -cdrom target/x86_64-unknown-none/debug/kernel || \
qemu-system-x86_64 \
    -m 256 \
    -display none \
    -serial stdio \
    -kernel target/x86_64-unknown-none/debug/kernel
