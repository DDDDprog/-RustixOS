#!/bin/bash
set -e

echo "Building kernel..."
cargo build --target x86_64-unknown-none

echo "Running in QEMU with Multiboot..."
qemu-system-x86_64 \
    -m 256 \
    -display none \
    -serial stdio \
    -cdrom target/x86_64-unknown-none/debug/kernel
