/* A link script that the title sets (`link_script` in BUCK, and -Tlink.x in
 * .cargo/config.toml for Cargo builds) instead of the one xex-link derives
 * from xex.toml. This is the derived script, written with
 * `xex-link link-script link.x --manifest xex.toml`, to edit from. It does
 * not follow later changes to xex.toml. */
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
