#!/bin/bash
set -e
. "$HOME/.cargo/env" 2>/dev/null || true
cargo build --target x86_64-unknown-none
KERNEL_ELF="target/x86_64-unknown-none/debug/kernel"
ISO_FILE="build/rustixos.iso"
if command -v grub-mkrescue &> /dev/null; then
    mkdir -p build/iso/boot/grub
    cp "$KERNEL_ELF" build/iso/boot/kernel.elf
    cat > build/iso/boot/grub/grub.cfg << 'GRUB'
set default=0
set timeout=0
menuentry "RustixOS" {
    multiboot /boot/kernel.elf
}
GRUB
    grub-mkrescue -o "$ISO_FILE" build/iso/ 2>/dev/null || true
fi
echo "Running..."
qemu-system-x86_64 -m 256 -serial stdio -cdrom "$ISO_FILE"
