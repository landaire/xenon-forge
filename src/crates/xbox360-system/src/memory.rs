//! Memory operations over kernel mappings.

use xbox360_sys::xboxkrnl;

/// Write back data and invalidate instructions for an executable range.
///
/// # Safety
///
/// `address..address + length` must remain mapped for the calls.
pub unsafe fn synchronize_executable(address: *mut u8, length: u32) {
    // SAFETY: the caller keeps the range mapped.
    unsafe {
        xboxkrnl::KeFlushCacheRange(address.expose_provenance() as u32, length);
        xboxkrnl::KeSweepIcacheRange(address, length);
    }
}
