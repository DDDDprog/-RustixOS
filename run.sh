#!/bin/bash
set -e

# Source cargo env
. "$HOME/.cargo/env" 2>/dev/null || true

# Build kernel
echo "Building kernel..."
cargo build --target x86_64-unknown-none

KERNEL_ELF="target/x86_64-unknown-none/debug/kernel"
ISO_FILE="build/rustixos.iso"

# Try grub-mkrescue first
if command -v grub-mkrescue &> /dev/null; then
    echo "Creating ISO with grub-mkrescue..."
    mkdir -p build/iso/boot/grub
    cp "$KERNEL_ELF" build/iso/boot/kernel.elf
    cat > build/iso/boot/grub/grub.cfg << 'GRUB'
set default=0
set timeout=0
menuentry "RustixOS" {
    multiboot /boot/kernel.elf
}
GRUB
    grub-mkrescue -o "$ISO_FILE" build/iso/ 2>/dev/null || grub-mkrescue -o "$ISO_FILE" build/iso/
    echo "Running QEMU..."
    qemu-system-x86_64 -m 256 -display none -serial stdio -cdrom "$ISO_FILE"
    exit 0
fi

# Try xorriso
if command -v xorriso &> /dev/null; then
    echo "Creating ISO with xorriso..."
    mkdir -p build/iso/boot/grub
    cp "$KERNEL_ELF" build/iso/boot/kernel.elf
    cat > build/iso/boot/grub/grub.cfg << 'GRUB'
set default=0
set timeout=0
menuentry "RustixOS" {
    multiboot /boot/kernel.elf
}
GRUB
    xorriso -as mkisofs -iso-level 3 -full-iso9660-filenames -volid RUSTIXOS -output "$ISO_FILE" build/iso/
    echo "Running QEMU..."
    qemu-system-x86_64 -m 256 -display none -serial stdio -cdrom "$ISO_FILE"
    exit 0
fi

# Fallback: try direct boot
echo "No ISO tools found, trying direct boot..."
qemu-system-x86_64 -m 256 -display none -serial stdio -kernel "$KERNEL_ELF" 2>/dev/null || \
qemu-system-x86_64 -m 256 -display none -serial stdio -cdrom "$KERNEL_ELF"
