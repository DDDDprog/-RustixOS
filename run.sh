#!/bin/bash
set -e

echo "Building kernel with cargo..."
cargo build --target x86_64-unknown-none

echo "Creating ISO with GRUB..."
make iso

echo "Running in QEMU..."
make run-iso
