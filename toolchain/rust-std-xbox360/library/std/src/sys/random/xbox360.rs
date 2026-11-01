//! Random bytes from the kernel's cryptographic generator.

use xbox360_sys::xboxkrnl;

pub fn fill_bytes(bytes: &mut [u8]) {
    for chunk in bytes.chunks_mut(u32::MAX as usize) {
        // SAFETY: `chunk` is writable for its length.
        unsafe { xboxkrnl::XeCryptRandom(chunk.as_mut_ptr(), chunk.len() as u32) };
    }
}
