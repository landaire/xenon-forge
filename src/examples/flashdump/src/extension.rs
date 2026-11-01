//! Geometry fields of `SFCX_DEVICE_EXTENSION` (openxbox
//! kernel/src/sfcx/sfcxp.h:552-615), read from kernel memory to cross-check
//! the geometry request and to find the physical block count, which no
//! request returns.

use xbox360::sys::xboxkrnl;
use xbox360::sys::xboxkrnl::AnsiString;
use xbox360::sys::xboxkrnl::DeviceObject;
use xbox360::sys::xboxkrnl::ObjectAttributeFlags;

use crate::flash::ControllerConfig;
use crate::flash::DEVICE_NAME;
use crate::flash::Geometry;
use crate::flash::SECTOR_BYTES;

/// Address of `SfcxDeviceExtension` in the build 16547 kernel, where
/// `ControllerConfig` is at 0x801CE09C and `BlockSize` at 0x801CE5A0.
pub const BUILD_16547_BASE: u32 = 0x801C_DFE8;
const BUILD_16547: u16 = 16547;

/// `SFCX_MAXIMUM_BLOCKS`.
const MAXIMUM_BLOCKS: u16 = 4096;

/// Where the extension address came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionSource {
    /// `DEVICE_OBJECT::DeviceExtension` of `\Device\Flash`.
    DeviceObject,
    /// [`BUILD_16547_BASE`].
    Build16547,
}

impl ExtensionSource {
    pub const fn name(self) -> &'static str {
        match self {
            Self::DeviceObject => "device_object",
            Self::Build16547 => "build_16547_constant",
        }
    }
}

/// The device object fields in use.
#[derive(Debug, Clone, Copy)]
pub struct DeviceInfo {
    pub extension: u32,
    pub alignment_requirement: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Extension {
    pub base: u32,
    pub source: ExtensionSource,

    pub controller_config: ControllerConfig,
    pub block_size: u32,
    pub page_size: u16,
    /// 512-byte sectors per block.
    pub pages_per_block: u16,
    pub physical_block_count: u16,
    pub block_count: u16,
    pub spare_block_count: u16,
    pub metadata_type: u8,
    pub page_shift: u8,
    pub block_shift: u8,
    pub sectors_per_page: u8,
    pub logical_block_count: u16,
}

impl Extension {
    /// Reads the extension at `base`.
    ///
    /// # Safety
    ///
    /// `base` must be the address of a mapped `SFCX_DEVICE_EXTENSION`.
    pub unsafe fn read(base: u32, source: ExtensionSource) -> Self {
        const CONTROLLER_CONFIG: u32 = 0xB4;
        const BLOCK_SIZE: u32 = 0x5B8;
        const PAGE_SIZE: u32 = 0x5BC;
        const PAGES_PER_BLOCK: u32 = 0x5BE;
        const PHYSICAL_BLOCK_COUNT: u32 = 0x5C0;
        const BLOCK_COUNT: u32 = 0x5C2;
        const SPARE_BLOCK_COUNT: u32 = 0x5C4;
        const METADATA_TYPE: u32 = 0x5C8;
        const PAGE_SHIFT: u32 = 0x5C9;
        const BLOCK_SHIFT: u32 = 0x5CA;
        const SECTORS_PER_PAGE: u32 = 0x5CB;
        const LOGICAL_BLOCK_COUNT: u32 = 0x5CE;

        // SAFETY: the caller guarantees the extension is mapped.
        unsafe {
            Self {
                base,
                source,
                controller_config: ControllerConfig(read_u32(base + CONTROLLER_CONFIG)),
                block_size: read_u32(base + BLOCK_SIZE),
                page_size: read_u16(base + PAGE_SIZE),
                pages_per_block: read_u16(base + PAGES_PER_BLOCK),
                physical_block_count: read_u16(base + PHYSICAL_BLOCK_COUNT),
                block_count: read_u16(base + BLOCK_COUNT),
                spare_block_count: read_u16(base + SPARE_BLOCK_COUNT),
                metadata_type: read_u8(base + METADATA_TYPE),
                page_shift: read_u8(base + PAGE_SHIFT),
                block_shift: read_u8(base + BLOCK_SHIFT),
                sectors_per_page: read_u8(base + SECTORS_PER_PAGE),
                logical_block_count: read_u16(base + LOGICAL_BLOCK_COUNT),
            }
        }
    }

    /// The checks that fail, comparing the fields with each other, with the
    /// geometry request, with the configuration the last read request
    /// returned and with the device object. Empty when the layout
    /// assumption holds.
    pub fn mismatches(&self, geometry: &Geometry, config: ControllerConfig, device: Option<DeviceInfo>) -> Vec<&'static str> {
        let shift_ok = |shift: u8| shift < 32;
        let checks = [
            ("device_extension", device.is_none_or(|device| device.extension == self.base)),
            ("controller_config", self.controller_config == config),
            (
                "block_size",
                shift_ok(self.block_shift) && self.block_size == 1 << self.block_shift,
            ),
            (
                "page_size",
                shift_ok(self.page_shift) && u32::from(self.page_size) == 1 << self.page_shift,
            ),
            (
                "pages_per_block",
                u32::from(self.pages_per_block) == self.block_size / SECTOR_BYTES
                    && u32::from(self.pages_per_block) == geometry.sectors_per_block,
            ),
            (
                "sectors_per_page",
                u32::from(self.sectors_per_page) == u32::from(self.page_size) / SECTOR_BYTES,
            ),
            ("block_count", u32::from(self.block_count) == geometry.block_count),
            ("spare_block_count", u32::from(self.spare_block_count) == geometry.spare_block_count),
            (
                "logical_block_count",
                Some(self.logical_block_count) == self.block_count.checked_sub(self.spare_block_count),
            ),
            (
                "physical_block_count",
                (self.block_count..=MAXIMUM_BLOCKS).contains(&self.physical_block_count),
            ),
        ];

        checks.into_iter().filter(|&(_, ok)| !ok).map(|(name, _)| name).collect()
    }

    /// Flash bytes that the configuration register describes, which the
    /// driver uses for the physical block count when the board reports no
    /// smaller flash (sfcx.c:6284-6289). `None` for a 16 KB block flash,
    /// whose physical block count is the block count.
    pub fn config_flash_bytes(&self) -> Option<u64> {
        const SMALL_BLOCK_SHIFT: u8 = 14;

        if self.block_shift == SMALL_BLOCK_SHIFT {
            return None;
        }

        let config = self.controller_config.0;
        let shift = ((config >> 21) & 0xF) + ((config >> 19) & 0x3) + 23;

        Some(1u64 << shift)
    }
}

/// References `\Device\Flash` and reads its device object.
pub fn device_info() -> Option<DeviceInfo> {
    let name = AnsiString::new(DEVICE_NAME)?;
    let object_type = xboxkrnl::IoDeviceObjectType.get()?;
    let mut object = 0u32;

    // SAFETY: the name and `object` are live locals for the call.
    let status = unsafe {
        xboxkrnl::ObReferenceObjectByName(
            &name,
            ObjectAttributeFlags::CASE_INSENSITIVE,
            std::ptr::from_ref(object_type).addr() as u32,
            0,
            &mut object,
        )
    };
    if !status.is_success() || object == 0 {
        return None;
    }

    // SAFETY: the reference keeps the device object alive until the
    // dereference below.
    let info = unsafe {
        let device = std::ptr::with_exposed_provenance::<DeviceObject>(object as usize);
        DeviceInfo {
            extension: (*device).device_extension.addr() as u32,
            alignment_requirement: (*device).alignment_requirement,
        }
    };

    // SAFETY: releases the reference taken above.
    unsafe { xboxkrnl::ObDereferenceObject(object) };

    Some(info)
}

/// The extension address and its source. On build 16547 the static
/// extension address, which is mapped kernel data whatever the device
/// object layout; on another build the device object pointer.
pub fn locate(device: Option<DeviceInfo>, kernel_build: Option<u16>) -> Option<(u32, ExtensionSource)> {
    if kernel_build == Some(BUILD_16547) {
        return Some((BUILD_16547_BASE, ExtensionSource::Build16547));
    }

    device
        .filter(|device| device.extension != 0)
        .map(|device| (device.extension, ExtensionSource::DeviceObject))
}

unsafe fn read_u32(address: u32) -> u32 {
    // SAFETY: the caller guarantees `address` is mapped.
    unsafe { std::ptr::with_exposed_provenance::<u32>(address as usize).read_volatile() }
}

unsafe fn read_u16(address: u32) -> u16 {
    // SAFETY: the caller guarantees `address` is mapped.
    unsafe { std::ptr::with_exposed_provenance::<u16>(address as usize).read_volatile() }
}

unsafe fn read_u8(address: u32) -> u8 {
    // SAFETY: the caller guarantees `address` is mapped.
    unsafe { std::ptr::with_exposed_provenance::<u8>(address as usize).read_volatile() }
}
