#!/bin/bash

export PATH="$HOME/.cargo/bin:$PATH"

echo "=== Building RustixOS Kernel ==="
cargo build --target x86_64-unknown-none 2>&1 | tail -3

KERNEL_ELF="target/x86_64-unknown-none/debug/kernel"

# Create a 1.44MB floppy image for BIOS boot
dd if=/dev/zero of=build/floppy.img bs=512 count=2880 2>/dev/null

# Copy kernel to ISO directory first
mkdir -p build/iso/boot/grub
mkdir -p build/iso/EFI/boot

cp "$KERNEL_ELF" build/iso/boot/kernel.elf

# Create GRUB config
cat > build/iso/boot/grub/grub.cfg << 'GRUB'
set default=0
set timeout=0
menuentry "RustixOS" {
    multiboot /boot/kernel.elf
    boot
}
GRUB

# Create EFI boot image with embedded config
grub-mkimage -O x86_64-efi -o build/iso/EFI/boot/bootx64.efi -p /boot/grub normal boot linux multiboot 2>/dev/null

# Create ISO with both BIOS and EFI boot
xorriso -as mkisofs \
    -iso-level 2 \
    -b boot/grub/i386-pc/boot.img \
    -no-emul-boot \
    -append_partition 2 0xef build/iso/EFI/boot/bootx64.efi \
    -appended_part_as_gpt \
    -e EFI/boot/bootx64.efi \
    -no-emul-boot \
    -o build/rustixos.iso \
    build/iso/ 2>&1 | tail -5

echo ""
echo "=== Running QEMU with OVMF ==="
timeout 15 qemu-system-x86_64 -m 256 -cdrom build/rustixos.iso -bios /usr/share/qemu/OVMF.fd -nographic 2>&1 | head -40 || echo "Boot completed"
