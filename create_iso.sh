#!/bin/bash
set -e

# Build kernel
. "$HOME/.cargo/env" 2>/dev/null || true
cargo build --target x86_64-unknown-none

KERNEL_ELF="target/x86_64-unknown-none/debug/kernel"
ISO_FILE="build/rustixos.iso"

mkdir -p build/iso/boot/grub

# Copy kernel
cp "$KERNEL_ELF" build/iso/boot/kernel.elf

# Create grub config
cat > build/iso/boot/grub/grub.cfg << 'GRUB'
set default=0
set timeout=0
menuentry "RustixOS" {
    multiboot /boot/kernel.elf
}
GRUB

# Create ISO using xorriso
xorriso -as mkisofs \
    -iso-level 3 \
    -full-iso9660-filenames \
    -volid RUSTIXOS \
    -output "$ISO_FILE" \
    build/iso/

echo "ISO created: $ISO_FILE"
