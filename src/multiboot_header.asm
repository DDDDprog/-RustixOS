; Multiboot2 header for RustixOS
section .multiboot_header
align 8

multiboot_header:
    ; Magic number
    dd 0xe85250d6
    ; Architecture (0 = i386)
    dd 0
    ; Header length
    dd multiboot_header_end - multiboot_header
    ; Checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (multiboot_header_end - multiboot_header))

    ; End tag
    dw 0
    dw 0
    dd 8

multiboot_header_end:
