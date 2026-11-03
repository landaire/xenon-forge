//! Kernel expansions: signed code that the hypervisor verifies and
//! installs (`ExExpansionInstall`), and calls into installed expansions
//! (`ExExpansionCall`).

use core::fmt;

use xbox360_status::KernelError;
use xbox360_status::NtStatus;
use xbox360_sys::xboxkrnl;
use xbox360_sys::xboxkrnl::MemoryRegion;
use xbox360_sys::xboxkrnl::PageProtection;

pub use xbox360_sys::xboxkrnl::ExpansionId;

/// A failed [`install`] or [`call`]. [`ExpansionError::status`] gives the
/// kernel status.
// No thiserror: this crate builds for `core`-only console code, where the
// vendored thiserror does not build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpansionError {
    /// The image is empty, above [`ExpansionId::MAX_IMAGE_SIZE`] or not a
    /// multiple of [`ExpansionId::IMAGE_SIZE_ALIGNMENT`].
    InvalidImageSize { size: usize },
    /// The hypervisor refused the image, for example because its signature
    /// does not verify.
    Refused,
    /// `STATUS_NOT_IMPLEMENTED`, which the kernel returns when no
    /// expansion of the id is installed (an expansion can return it too).
    NotInstalled(ExpansionId),
    /// Another failure status.
    Kernel(KernelError),
}

impl fmt::Display for ExpansionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidImageSize { size } => write!(
                f,
                "expansion image of {size} bytes is empty, above 0x10000 bytes or not a multiple of 128"
            ),
            Self::Refused => f.write_str("the hypervisor refused the expansion image"),
            Self::NotInstalled(id) => write!(f, "no expansion {:#x} is installed", id.get()),
            Self::Kernel(error) => write!(f, "expansion request failed: {error}"),
        }
    }
}

impl core::error::Error for ExpansionError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error),
            _ => None,
        }
    }
}

impl ExpansionError {
    /// The kernel status of the failure.
    pub const fn status(self) -> NtStatus {
        match self {
            Self::InvalidImageSize { .. } => KernelError::InvalidParameter.status(),
            Self::Refused => KernelError::Unsuccessful.status(),
            Self::NotInstalled(_) => KernelError::NotImplemented.status(),
            Self::Kernel(error) => error.status(),
        }
    }
}

/// Lowest physical address the hypervisor accepts for an expansion image.
const LOWEST_IMAGE_ADDRESS: u32 = 0x4_0000;

/// Bytes handed to the hypervisor for every image: one whole 64 KiB
/// physical block, 64 KiB aligned.
const BLOCK_SIZE: usize = ExpansionId::MAX_IMAGE_SIZE;

/// Hand the signed expansion `image` to the hypervisor, which verifies and
/// installs it.
///
/// `ExExpansionInstall` passes the physical address of its buffer to the
/// hypervisor without copying. The hypervisor moves the buffer into
/// integrity-protected memory, which needs physically contiguous memory at
/// or above 0x40000 (below that it halts the console) within one 64 KiB
/// block, and zeroes the buffer afterwards. A buffer that covers only part
/// of its pages also halts the console in the stock 16547 hypervisor, so
/// `install` copies `image` to the start of one whole 64 KiB block it
/// allocates, passes the whole block and frees it after the call. The
/// kernel's own buffers for hypervisor calls of this kind are whole 64 KiB
/// blocks too. The hypervisor reads the image size from its header and
/// ignores the rest of the block.
///
/// Fails with [`ExpansionError::InvalidImageSize`] before any of that when
/// the image is empty, above [`ExpansionId::MAX_IMAGE_SIZE`] or not a
/// multiple of [`ExpansionId::IMAGE_SIZE_ALIGNMENT`], and with
/// `KernelError::NoMemory` when the allocation fails.
pub fn install(image: &[u8]) -> Result<(), ExpansionError> {
    let size = image.len();
    if size == 0
        || size > ExpansionId::MAX_IMAGE_SIZE
        || !size.is_multiple_of(ExpansionId::IMAGE_SIZE_ALIGNMENT)
    {
        return Err(ExpansionError::InvalidImageSize { size });
    }

    // One 64 KiB page: physically contiguous and 64 KiB aligned.
    // SAFETY: the call takes no addresses.
    let buffer = unsafe {
        xboxkrnl::MmAllocatePhysicalMemoryEx(
            MemoryRegion::CurrentProcess,
            BLOCK_SIZE as u32,
            PageProtection::READWRITE | PageProtection::LARGE_PAGES,
            LOWEST_IMAGE_ADDRESS,
            u32::MAX,
            BLOCK_SIZE as u32,
        )
    };
    if buffer == 0 {
        return Err(ExpansionError::Kernel(KernelError::NoMemory));
    }
    let buffer_ptr = core::ptr::with_exposed_provenance_mut::<u8>(buffer as usize);

    // SAFETY: the allocation is `BLOCK_SIZE` writable bytes that nothing
    // else uses, and `size` is at most `BLOCK_SIZE`; `image` is a separate
    // buffer of `size` bytes.
    unsafe { core::ptr::copy_nonoverlapping(image.as_ptr(), buffer_ptr, size) };

    // SAFETY: the buffer meets the hypervisor's placement rules above and
    // stays allocated until the call returns.
    let status = unsafe { xboxkrnl::ExExpansionInstall(buffer_ptr, BLOCK_SIZE as u32) };

    // SAFETY: the buffer came from MmAllocatePhysicalMemoryEx in the same
    // region and is not used after this.
    unsafe { xboxkrnl::MmFreePhysicalMemory(MemoryRegion::CurrentProcess, buffer) };

    match KernelError::from_status(status) {
        None => Ok(()),
        Some(KernelError::Unsuccessful) => Err(ExpansionError::Refused),
        Some(error) => Err(ExpansionError::Kernel(error)),
    }
}

/// Call the installed expansion `id` with four arguments and return its
/// success status.
///
/// # Safety
///
/// The expansion defines what the arguments mean; each address among them
/// must be valid for the accesses the expansion makes.
pub unsafe fn call(id: ExpansionId, arguments: [u32; 4]) -> Result<NtStatus, ExpansionError> {
    let [argument1, argument2, argument3, argument4] = arguments;

    // SAFETY: the caller guarantees the arguments meet the expansion's
    // requirements.
    let status =
        unsafe { xboxkrnl::ExExpansionCall(id, argument1, argument2, argument3, argument4) };
    match KernelError::from_status(status) {
        None => Ok(status),
        Some(KernelError::NotImplemented) => Err(ExpansionError::NotInstalled(id)),
        Some(error) => Err(ExpansionError::Kernel(error)),
    }
}
