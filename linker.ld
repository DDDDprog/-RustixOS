OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("i386:x86-64")
ENTRY(_start)

SECTIONS
{
    /* Multiboot header MUST be in first 8192 bytes - start at 0x1000 */
    . = 0x1000;

    .multiboot : {
        KEEP(*(.multiboot))
    }
    
    . = 0x100000;

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
        *(. COMMON)
    }
    
    /DISCARD/ : {
        *(.note.GNU-stack)
        *(.eh_frame)
    }
}
