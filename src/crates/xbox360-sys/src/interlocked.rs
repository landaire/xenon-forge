//! Interlocked operations in the form the kernel loader accepts.
//!
//! The loader reports every `lwarx`/`ldarx` that does not directly follow
//! `mtmsrd r13, 1` and every `stwcx.`/`stdcx.` that is not directly followed
//! by `mtmsrd rS, 1`. Each reservation here runs with external interrupts
//! disabled and restores the previous MSR after the store.
//!
//! The 64-bit operations are inline functions that pass each value to the
//! assembly in two 32-bit halves and join them into one doubleword, so they
//! work with the register pairs of the powerpc code generation and the
//! single registers of powerpc64 ILP32 alike.
//!
//! Each function documents its memory ordering. None orders later accesses
//! after the operation: a caller that needs acquire ordering issues
//! `isync` (after a dependent branch) or `lwsync` after the call, and a
//! caller of a relaxed operation that needs release ordering issues
//! `lwsync` before it.

/// Compare the word at `target` with `expected` and store `new` on a match.
/// Returns the observed word. A mismatch stores the observed word back to
/// end the reservation.
///
/// # Ordering
///
/// Release: a leading `lwsync` orders earlier loads and stores before the
/// operation. No barrier follows it.
///
/// # Safety
///
/// `target` must be valid for reads and writes and 4-byte aligned.
#[unsafe(naked)]
pub unsafe extern "C" fn compare_exchange32(target: *mut u32, expected: u32, new: u32) -> u32 {
    core::arch::naked_asm!(
        "lwsync",
        "1:",
        "mfmsr 9",
        "mtmsrd 13, 1",
        "lwarx 10, 0, 3",
        "cmpw 10, 4",
        "bne 2f",
        "stwcx. 5, 0, 3",
        "mtmsrd 9, 1",
        "bne 1b",
        "b 3f",
        "2:",
        "stwcx. 10, 0, 3",
        "mtmsrd 9, 1",
        "3:",
        "mr 3, 10",
        "blr",
    )
}

/// Compare the doubleword at `target` with `expected` and store `new` on a
/// match. Returns the observed doubleword. A mismatch stores the observed
/// value back to end the reservation.
///
/// # Ordering
///
/// Release: a leading `lwsync` orders earlier loads and stores before the
/// operation. No barrier follows it.
///
/// # Safety
///
/// `target` must be valid for reads and writes and 8-byte aligned.
#[inline]
pub unsafe fn compare_exchange64(target: *mut u64, expected: u64, new: u64) -> u64 {
    let (high, low): (u32, u32);
    // SAFETY: the caller guarantees that `target` is valid and aligned.
    unsafe {
        core::arch::asm!(
            "lwsync",
            "rldimi {expected_low}, {expected_high}, 32, 0",
            "rldimi {new_low}, {new_high}, 32, 0",
            "2:",
            "mfmsr {msr}",
            "mtmsrd 13, 1",
            "ldarx {seen}, 0, {target}",
            "cmpd {seen}, {expected_low}",
            "bne 3f",
            "stdcx. {new_low}, 0, {target}",
            "mtmsrd {msr}, 1",
            "bne 2b",
            "b 4f",
            "3:",
            "stdcx. {seen}, 0, {target}",
            "mtmsrd {msr}, 1",
            "4:",
            "srdi {high}, {seen}, 32",
            "clrldi {low}, {seen}, 32",
            target = in(reg) target,
            expected_high = in(reg) (expected >> 32) as u32,
            expected_low = inout(reg) expected as u32 => _,
            new_high = in(reg) (new >> 32) as u32,
            new_low = inout(reg) new as u32 => _,
            msr = out(reg) _,
            seen = out(reg) _,
            high = lateout(reg) high,
            low = lateout(reg) low,
            options(nostack),
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}

/// Add `delta` to the word at `target`. Returns the previous value.
///
/// # Ordering
///
/// Relaxed: no barrier.
///
/// # Safety
///
/// `target` must be valid for reads and writes and 4-byte aligned.
#[unsafe(naked)]
pub unsafe extern "C" fn fetch_add32(target: *mut u32, delta: u32) -> u32 {
    core::arch::naked_asm!(
        "1:",
        "mfmsr 9",
        "mtmsrd 13, 1",
        "lwarx 10, 0, 3",
        "add 11, 10, 4",
        "stwcx. 11, 0, 3",
        "mtmsrd 9, 1",
        "bne 1b",
        "mr 3, 10",
        "blr",
    )
}

/// Add `delta` to the doubleword at `target`, wrapping on overflow.
/// Returns the previous value.
///
/// # Ordering
///
/// Relaxed: no barrier.
///
/// # Safety
///
/// `target` must be valid for reads and writes and 8-byte aligned.
#[inline]
pub unsafe fn fetch_add64(target: *mut u64, delta: u64) -> u64 {
    let (high, low): (u32, u32);
    // SAFETY: the caller guarantees that `target` is valid and aligned.
    unsafe {
        core::arch::asm!(
            "rldimi {delta_low}, {delta_high}, 32, 0",
            "2:",
            "mfmsr {msr}",
            "mtmsrd 13, 1",
            "ldarx {seen}, 0, {target}",
            "add {sum}, {seen}, {delta_low}",
            "stdcx. {sum}, 0, {target}",
            "mtmsrd {msr}, 1",
            "bne 2b",
            "srdi {high}, {seen}, 32",
            "clrldi {low}, {seen}, 32",
            target = in(reg) target,
            delta_high = in(reg) (delta >> 32) as u32,
            delta_low = inout(reg) delta as u32 => _,
            msr = out(reg) _,
            seen = out(reg) _,
            sum = out(reg) _,
            high = lateout(reg) high,
            low = lateout(reg) low,
            options(nostack),
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}

/// Store `value` at `target`. Returns the previous value.
///
/// # Ordering
///
/// Relaxed: no barrier.
///
/// # Safety
///
/// `target` must be valid for reads and writes and 4-byte aligned.
#[unsafe(naked)]
pub unsafe extern "C" fn exchange32(target: *mut u32, value: u32) -> u32 {
    core::arch::naked_asm!(
        "1:",
        "mfmsr 9",
        "mtmsrd 13, 1",
        "lwarx 10, 0, 3",
        "stwcx. 4, 0, 3",
        "mtmsrd 9, 1",
        "bne 1b",
        "mr 3, 10",
        "blr",
    )
}

/// Load the doubleword at `address` in one access.
///
/// # Ordering
///
/// Relaxed: no barrier.
///
/// # Safety
///
/// `address` must be valid for reads and 8-byte aligned.
#[inline]
pub unsafe fn load64(address: *const u64) -> u64 {
    let (high, low): (u32, u32);
    // SAFETY: the caller guarantees that `address` is valid and aligned.
    unsafe {
        core::arch::asm!(
            "ld {value}, 0({address})",
            "srdi {high}, {value}, 32",
            "clrldi {low}, {value}, 32",
            address = in(reg_nonzero) address,
            value = out(reg) _,
            high = lateout(reg) high,
            low = lateout(reg) low,
            options(nostack, preserves_flags),
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}

/// Store `value` at `address` in one access.
///
/// # Ordering
///
/// Relaxed: no barrier.
///
/// # Safety
///
/// `address` must be valid for writes and 8-byte aligned.
#[inline]
pub unsafe fn store64(address: *mut u64, value: u64) {
    // SAFETY: the caller guarantees that `address` is valid and aligned.
    unsafe {
        core::arch::asm!(
            "rldimi {low}, {high}, 32, 0",
            "std {low}, 0({address})",
            address = in(reg_nonzero) address,
            high = in(reg) (value >> 32) as u32,
            low = inout(reg) value as u32 => _,
            options(nostack, preserves_flags),
        );
    }
}
