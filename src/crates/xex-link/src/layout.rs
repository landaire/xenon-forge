use xex2::opt::ImageFlags;

use crate::pe;

// Sections and the file are aligned to the loader page size: 4 KB with
// SMALL_PAGES, else 64 KB.
pub const LARGE_PAGE: u32 = 0x1_0000;
pub const SMALL_PAGE: u32 = 0x1000;

/// Loader page size, which aligns the load address, sections, and file.
pub fn page_size(image_flags: ImageFlags) -> u32 {
    if image_flags.contains(ImageFlags::SMALL_PAGES) {
        SMALL_PAGE
    } else {
        LARGE_PAGE
    }
}

/// Page size of an image loaded at `address`: 64 KB in 0x80000000-0x8FFFFFFF
/// and 4 KB (SMALL_PAGES) in 0x90000000-0x9FFFFFFF. `None` elsewhere: no
/// image loads there. A development kit refused a module with "XEX:
/// XexpLoadfile: Load address violation" (0xC000007B) when its page size
/// did not match (at 0x82000000 and 0x91D00000) and at 0x10000000,
/// 0x40000000 and 0xA0000000 with either page size.
pub fn image_page_size(address: u32) -> Option<u32> {
    match address >> 28 {
        0x8 => Some(LARGE_PAGE),
        0x9 => Some(SMALL_PAGE),
        _ => None,
    }
}

pub fn image_end(sections: &[pe::Section]) -> u32 {
    sections
        .iter()
        .map(|section| section.rva + section.virtual_size)
        .max()
        .unwrap_or(0)
}
