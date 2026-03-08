#!/bin/bash
set -e

export PATH="$HOME/.cargo/bin:$PATH"

echo "=== Building RustixOS Kernel ==="
cargo build --target x86_64-unknown-none

KERNEL_ELF="target/x86_64-unknown-none/debug/kernel"
ISO_FILE="build/rustixos.iso"

echo ""
echo "=== Creating Bootable ISO ==="

# Check for GRUB
if command -v grub-mkrescue &> /dev/null; then
    echo "Using GRUB to create ISO..."
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
else
    echo "ERROR: grub-mkrescue not found!"
    echo ""
    echo "Please install GRUB tools:"
    echo "  sudo apt-get update"
    echo "  sudo apt-get install grub-common xorriso"
    echo ""
    echo "Then run this script again."
    exit 1
fi

echo ""
echo "=== Running QEMU ==="
qemu-system-x86_64 -m 256 -cdrom "$ISO_FILE"
