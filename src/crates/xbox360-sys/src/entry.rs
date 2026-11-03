/// Call the function in r12 with r3-r10 unchanged and the full 64-bit
/// r14-r31 preserved.
///
/// The frame is laid out like the C runtime entry (`src/sdk/crt/entry.inc`):
/// SP+0x10 to SP+0x4f is the home area where a console-ABI callee (the
/// powerpc64 ILP32 code generation) stores its arguments, so r14-r31 are
/// saved above it, from SP+0x50.
///
/// # Safety
///
/// Enter it only with `b` from a stub that loads r12 with the target, as
/// [`bridged!`](crate::bridged!) does. It is not callable from Rust.
#[cfg(any(target_os = "xbox360", xbox360_docs))]
#[unsafe(naked)]
pub unsafe extern "C" fn entry_bridge() {
    core::arch::naked_asm!(
        "stwu 1, -{frame}(1)",
        "mflr 0",
        "stw 0, 0x8(1)",
        ".irp reg, 14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31",
        "std \\reg, {saves} + 8 * (\\reg - 14)(1)",
        ".endr",
        "mtctr 12",
        "bctrl",
        ".irp reg, 14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31",
        "ld \\reg, {saves} + 8 * (\\reg - 14)(1)",
        ".endr",
        "lwz 0, 0x8(1)",
        "mtlr 0",
        "addi 1, 1, {frame}",
        "blr",
        frame = const BRIDGE_FRAME_SIZE,
        saves = const BRIDGE_SAVE_OFFSET,
    )
}

/// End of the home area of a console callee: it stores r3-r10 at
/// SP+0x10 to SP+0x4f of its caller's frame.
#[cfg(any(target_os = "xbox360", xbox360_docs))]
const HOME_AREA_END: usize = 0x50;
/// Where [`entry_bridge`] saves r14-r31: above the home area of the
/// function it calls.
#[cfg(any(target_os = "xbox360", xbox360_docs))]
const BRIDGE_SAVE_OFFSET: usize = HOME_AREA_END;
/// Frame of [`entry_bridge`]: the home area and 18 saved doublewords,
/// 16-byte aligned. The same as the C runtime entry.
#[cfg(any(target_os = "xbox360", xbox360_docs))]
const BRIDGE_FRAME_SIZE: usize = (BRIDGE_SAVE_OFFSET + 18 * 8 + 15) & !15;

#[cfg(any(target_os = "xbox360", xbox360_docs))]
const _: () = {
    assert!(BRIDGE_FRAME_SIZE == 0xe0);
    assert!(BRIDGE_FRAME_SIZE.is_multiple_of(16));
};

/// Define `$entry`, an entry for console-ABI callers that runs `$target`
/// (an `extern "C" fn` with at most eight 32-bit arguments and a 32-bit or
/// no result) behind [`entry_bridge`], or behind `bridge`, which must end by
/// branching to [`entry_bridge`] with r12 unchanged.
///
/// ```ignore
/// xbox360_sys::bridged!(#[unsafe(no_mangle)] pub _start, title_main);
/// ```
#[macro_export]
macro_rules! bridged {
    ($(#[$meta:meta])* $vis:vis $entry:ident, $target:path, bridge = $bridge:path) => {
        $(#[$meta])*
        #[unsafe(naked)]
        $vis unsafe extern "C" fn $entry() {
            ::core::arch::naked_asm!(
                "lis 12, {target}@ha",
                "addi 12, 12, {target}@l",
                "b {bridge}",
                target = sym $target,
                bridge = sym $bridge,
            )
        }
    };
    ($(#[$meta:meta])* $vis:vis $entry:ident, $target:path) => {
        $crate::bridged!($(#[$meta])* $vis $entry, $target, bridge = $crate::entry_bridge);
    };
}
