/*
 * Macros of the import thunk library. src/tools/xbox360-bindgen writes one
 * invocation of them per import into ppc32.S and ppc64.S.
 *
 * A thunk is 16 bytes: the import record (type << 24 | library << 16 |
 * ordinal), then nop; nop; blr, which only runs if the loader never patches
 * it. The loader rewrites the thunk to call the export.
 *
 * Each function, thunk and slot is in its own section, so that
 * --gc-sections drops the imports nobody uses. Each symbol is weak and
 * defined once per assembly unit, so separate copies of the library link
 * together and can share one unit, such as a fat LTO module.
 */

.ifndef .Lxbox360_thunk_macros
.set .Lxbox360_thunk_macros, 1

.macro xbox360_thunk_words record
	.long \record
	.long 0x60000000
	.long 0x60000000
	.long 0x4e800020
.endm

/* A function import that is its thunk (powerpc64 ILP32 callers). */
.macro xbox360_thunk name, record
.ifndef \name
	.pushsection .xthunks.\name, "ax", @progbits
	.balign 4
	.weak \name
	.type \name, @function
\name:
	xbox360_thunk_words \record
	.size \name, 16
	.popsection
.endif
.endm

/* A data import slot, which the loader sets to the address of the export. */
.macro xbox360_slot name, record
.ifndef \name
	.pushsection .xvars.\name, "aw", @progbits
	.balign 4
	.weak \name
	.type \name, @object
	.size \name, 4
\name:
	.long \record
	.popsection
.endif
.endm

/*
 * The start of a function \name that calls the thunk of the import. Close
 * it with xbox360_end.
 */
.macro xbox360_function name
	.pushsection .text.xbox360_import.\name, "ax", @progbits
	.balign 4
	.weak \name
	.type \name, @function
\name:
.endm

/*
 * A frame of \frame bytes with the home area of r3-r10 at 0x10-0x50, which
 * console callees store their arguments in.
 */
.macro xbox360_frame frame
	stwu 1, -\frame(1)
	mflr 0
	stw 0, \frame+4(1)
.endm

.macro xbox360_call name
	bl .Lxbox360_thunk_\name
.endm

/*
 * Convert the result of the export for a SysV caller: a 64-bit integer goes
 * to r3 (high) and r4 (low), and the bits above a narrow integer, which the
 * export leaves undefined, are extended.
 */
.macro xbox360_result kind
.ifc \kind, u64
	mr 4, 3
	rldicl 3, 3, 32, 32
.endif
.ifc \kind, u8
	clrlwi 3, 3, 24
.endif
.ifc \kind, u16
	clrlwi 3, 3, 16
.endif
.ifc \kind, i8
	extsb 3, 3
.endif
.ifc \kind, i16
	extsh 3, 3
.endif
.endm

.macro xbox360_return frame
	lwz 0, \frame+4(1)
	mtlr 0
	addi 1, 1, \frame
	blr
.endm

/* The end of the function \name, and its thunk. */
.macro xbox360_end name, record
	.size \name, .-\name
	.section .xthunks.\name, "ax", @progbits
	.balign 4
.Lxbox360_thunk_\name:
	xbox360_thunk_words \record
	.popsection
.endm

/*
 * A function import for SysV (powerpc) callers whose arguments need no
 * moves: it adds the home area and converts the result `kind`.
 */
.macro xbox360_stub name, record, kind=word
.ifndef \name
	xbox360_function \name
	xbox360_frame 0x50
	xbox360_call \name
	xbox360_result \kind
	xbox360_return 0x50
	xbox360_end \name, \record
.endif
.endm

/*
 * A function import that does not return. It traps if the export returns
 * anyway, as Xenia's KeBugCheck does. `frame` is 0 when the caller already
 * reserves the home area (powerpc64 ILP32).
 */
.macro xbox360_noreturn name, record, frame=0x50
.ifndef \name
	xbox360_function \name
.if \frame
	xbox360_frame \frame
.endif
	xbox360_call \name
	trap
	xbox360_end \name, \record
.endif
.endm

.endif
