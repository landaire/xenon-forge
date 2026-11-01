//! Physical memory for read requests. The driver completes a request in a
//! DPC on processor 2, which writes the request block, data and spare
//! buffers, so none of them is on a thread stack.

use std::ptr::NonNull;

use xbox360::sys::xboxkrnl;
use xbox360::sys::xboxkrnl::MemoryRegion;
use xbox360::sys::xboxkrnl::PageProtection;

use crate::error::DumpError;
use crate::flash::DMA_SECTORS;
use crate::flash::PassThrough;
use crate::flash::SECTOR_BYTES;
use crate::flash::SPARE_BYTES;

const PAGE_BYTES: usize = 4096;

/// The data page is one aligned 4 KB page, which READ_PAGES requires
/// (sfcx.c:4724-4734).
#[repr(C, align(4096))]
struct Pages {
    data: [u8; PAGE_BYTES],
    pass_through: PassThrough,
    spare: [u8; (DMA_SECTORS * SPARE_BYTES) as usize],
}

const _: () = assert!((DMA_SECTORS * SECTOR_BYTES) as usize == PAGE_BYTES);

pub struct DmaBuffers {
    pages: NonNull<Pages>,
}

impl DmaBuffers {
    pub fn new() -> Result<Self, DumpError> {
        const BYTES: u32 = size_of::<Pages>() as u32;

        // SAFETY: plain allocation call.
        let address = unsafe {
            xboxkrnl::MmAllocatePhysicalMemoryEx(
                MemoryRegion::System,
                BYTES,
                PageProtection::READWRITE,
                0,
                u32::MAX,
                PAGE_BYTES as u32,
            )
        };
        let Some(pages) = NonNull::new(std::ptr::with_exposed_provenance_mut::<Pages>(address as usize)) else {
            return Err(DumpError::Allocation { bytes: BYTES });
        };

        Ok(Self { pages })
    }

    pub fn pass_through(&mut self) -> *mut PassThrough {
        // SAFETY: `pages` is the live allocation.
        unsafe { &raw mut (*self.pages.as_ptr()).pass_through }
    }

    pub fn data_address(&self) -> u32 {
        // SAFETY: `pages` is the live allocation.
        unsafe { (&raw const (*self.pages.as_ptr()).data).expose_provenance() as u32 }
    }

    pub fn spare_address(&self) -> u32 {
        // SAFETY: `pages` is the live allocation.
        unsafe { (&raw const (*self.pages.as_ptr()).spare).expose_provenance() as u32 }
    }

    /// Data bytes of sector `index` of the last request.
    pub fn data(&self, index: u32) -> &[u8] {
        let start = (index * SECTOR_BYTES) as usize;

        // SAFETY: `pages` is the live allocation, and no request is in
        // progress while `self` is borrowed shared.
        let data = unsafe { &(*self.pages.as_ptr()).data };

        &data[start..start + SECTOR_BYTES as usize]
    }

    /// Spare bytes of sector `index` of the last request.
    pub fn spare(&self, index: u32) -> &[u8] {
        let start = (index * SPARE_BYTES) as usize;

        // SAFETY: as in `data`.
        let spare = unsafe { &(*self.pages.as_ptr()).spare };

        &spare[start..start + SPARE_BYTES as usize]
    }
}

impl Drop for DmaBuffers {
    fn drop(&mut self) {
        // SAFETY: `pages` came from MmAllocatePhysicalMemoryEx in the same
        // region.
        unsafe { xboxkrnl::MmFreePhysicalMemory(MemoryRegion::System, self.pages.as_ptr().addr() as u32) };
    }
}
