/* Written by `xex-link link-script` for Cargo builds; Buck derives the same script from xex.toml. */
ENTRY(_start)

SECTIONS {
    . = 0x82010000;
    .text : {
        *(.text._start)
        *(.text .text.*)
    }
    . = ALIGN(4);
    .xthunks : { *(.xthunks .xthunks.*) }

    . = ALIGN(0x10000);
    .rodata : { *(.rodata .rodata.*) }

    . = ALIGN(0x10000);
    .init_array : { KEEP(*(.init_array .init_array.*)) }
    .data : { *(.data .data.*) *(.sdata .sdata.*) }
    .xvars : { *(.xvars .xvars.*) }

    . = ALIGN(0x10000);
    .bss : { *(.bss .bss.*) *(.sbss .sbss.*) *(COMMON) }

    /DISCARD/ : { *(.eh_frame) *(.eh_frame_hdr) *(.comment) *(.note .note.*) }
}
