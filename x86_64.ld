/* Linker script for RustixOS kernel with multiboot2 header */
OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("i386:x86-64")
ENTRY("_start")

SECTIONS
{
    . = 1M;

    /* Multiboot2 header must be in first 32KiB and aligned to 8 bytes */
    .multiboot_header : {
        *(.multiboot_header)
    }

    .text : {
        *(.text)
    }

    .rodata : {
        *(.rodata)
    }

    .data : {
        *(.data)
    }

    .bss : {
        *(.bss)
    }

    /DISCARD/ : {
        *(.note.GNU-stack)
        *(.eh_frame)
    }
}
