/*
 * Entries of C and C++ modules.
 *
 * The loader calls the entry of a module with the console calling
 * convention, which expects r14-r31 back in full 64 bits. Code of the
 * powerpc (SysV) code generation saves only their low words. The entry
 * saves them in full, reserves the home area that console callees store
 * their arguments in, and calls `target` with r3-r10 unchanged.
 */

.macro xbox360_entry name, target
	.section .text.\name, "ax", @progbits
	.globl \name
	.type \name, @function
	.balign 4
\name:
	stwu 1, -0xe0(1)
	mflr 0
	stw 0, 0x8(1)
	.irp reg, 14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
	std \reg, 0x50 + 8 * (\reg - 14)(1)
	.endr
	bl \target
	.irp reg, 14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
	ld \reg, 0x50 + 8 * (\reg - 14)(1)
	.endr
	lwz 0, 0x8(1)
	mtlr 0
	addi 1, 1, 0xe0
	blr
	.size \name, .-\name
.endm
