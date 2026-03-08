; Multiboot header
; This header must be within the first 8192 bytes of the kernel

section .multiboot
align 4
multiboot_header:
    dd 0x1BADB002
    dd 0x00010003
    dd -(0x1BADB002 + 0x00010003)
    dd 0x100000
    dd 0x100000
