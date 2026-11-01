//! Read requests to `\Device\Flash` (openxbox kernel/src/sfcx/sfcxp.h:140-200,
//! sfcx.c:538-561 and 4600-4790).

use std::fmt;
use std::ptr;

use xbox360::sys::xboxkrnl;
use xbox360::sys::xboxkrnl::AccessMask;
use xbox360::sys::xboxkrnl::AnsiString;
use xbox360::sys::xboxkrnl::CreateOptions;
use xbox360::sys::xboxkrnl::Handle;
use xbox360::sys::xboxkrnl::IoControlCode;
use xbox360::sys::xboxkrnl::IoStatusBlock;
use xbox360::sys::xboxkrnl::NtStatus;
use xbox360::sys::xboxkrnl::ObjectAttributeFlags;
use xbox360::sys::xboxkrnl::ObjectAttributes;
use xbox360::sys::xboxkrnl::ShareAccess;

use crate::dma::DmaBuffers;
use crate::error::DumpError;

/// Data bytes of a sector (`SFCX_PAGE_SIZE`).
pub const SECTOR_BYTES: u32 = 512;
/// Spare bytes the controller returns with a sector (`SFCX_METADATA_SIZE`).
pub const SPARE_BYTES: u32 = 16;
/// Sectors of one READ_PAGES request. The driver limits a DMA transfer to
/// one 4 KB buffer page (sfcx.c:4457, 4703-4736).
pub const DMA_SECTORS: u32 = 8;

pub const DEVICE_NAME: &[u8] = b"\\Device\\Flash";

/// Physical sector index. Sector `n` holds flash data bytes `n * 512` to
/// `n * 512 + 511`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sector(pub u32);

impl Sector {
    /// `SFCX_PASS_THROUGH::FlashAddress`: the physical byte address,
    /// counting data bytes only.
    pub const fn flash_address(self) -> u32 {
        self.0 * SECTOR_BYTES
    }
}

impl fmt::Display for Sector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// `SFCX_PASS_THROUGH::ControllerStatus`, the status register after the
/// request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerStatus(pub u32);

impl ControllerStatus {
    /// Write/erase error, ECC state and bus abort bits (sfcxp.h:285-295).
    const ERROR_MASK: u32 = 0x0000_0002 | 0x0000_001C | 0x0000_2000 | 0x0000_4000;

    pub const fn is_error(self) -> bool {
        self.0 & Self::ERROR_MASK != 0
    }
}

/// The controller configuration register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerConfig(pub u32);

/// A device control code without the write access bit.
#[derive(Clone, Copy)]
pub struct ReadControl(IoControlCode);

impl ReadControl {
    pub const GET_FLASH_GEOMETRY: Self = Self::new(0x003C_0028);
    pub const READ_PAGE: Self = Self::new(0x003C_0004);
    pub const READ_PAGES: Self = Self::new(0x003C_002C);

    /// Rejects a code with `SFCX_WRITE_ACCESS` (sfcxp.h:200,
    /// `SFCX_IS_WRITE_IOCTL`). The constants above fail to compile if one
    /// has it.
    const fn new(code: u32) -> Self {
        assert!(!is_write_control(code), "flash write control code");
        Self(IoControlCode(code))
    }
}

/// How a sector is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadMethod {
    /// READ_PAGES: DMA of [`DMA_SECTORS`] sectors.
    Dma,
    /// READ_PAGE: one sector through the data register.
    Polled,
}

impl ReadMethod {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dma => "dma",
            Self::Polled => "polled",
        }
    }
}

/// `SFCX_FLASH_GEOMETRY` (sfcxp.h:1015). The kernel calls a 512-byte
/// sector a page.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Geometry {
    pub bytes_per_sector: u32,
    pub spare_bytes_per_sector: u32,
    pub sectors_per_block: u32,
    /// Logical and spare blocks.
    pub block_count: u32,
    pub spare_block_count: u32,
}

const _: () = assert!(size_of::<Geometry>() == 20);

/// `SFCX_PASS_THROUGH` (sfcxp.h:142). The request passes the same block as
/// its input and output buffer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PassThrough {
    pub flash_address: u32,
    pub buffer: u32,
    pub controller_config: u32,
    pub controller_status: u32,
    pub logical_address: u32,
    pub physical_address: u32,
    pub metadata_buffer: u32,
    /// Sectors to transfer.
    pub page_count: u32,
}

const _: () = assert!(size_of::<PassThrough>() == 32);

/// Result of a read request that the driver accepted.
#[derive(Debug, Clone, Copy)]
pub struct Completion {
    pub status: ControllerStatus,
    pub config: ControllerConfig,
}

/// `\Device\Flash`, opened for reading.
pub struct FlashDevice {
    handle: Handle,
}

impl FlashDevice {
    pub fn open() -> Result<Self, DumpError> {
        let Some(name) = AnsiString::new(DEVICE_NAME) else {
            unreachable!("device name is shorter than u16::MAX");
        };
        let attributes = ObjectAttributes {
            root_directory: Handle::NULL,
            object_name: &name,
            attributes: ObjectAttributeFlags::CASE_INSENSITIVE,
        };
        let mut handle = Handle::NULL;
        let mut io_status = IoStatusBlock::default();

        // SAFETY: the name, attributes, handle and status block are live
        // locals for the call.
        let status = unsafe {
            xboxkrnl::NtOpenFile(
                &mut handle,
                AccessMask::GENERIC_READ | AccessMask::SYNCHRONIZE,
                &attributes,
                &mut io_status,
                ShareAccess::READ | ShareAccess::WRITE,
                CreateOptions::SYNCHRONOUS_IO_NONALERT,
            )
        };
        if !status.is_success() {
            return Err(DumpError::Open { path: "\\Device\\Flash", status });
        }

        Ok(Self { handle })
    }

    pub fn geometry(&self) -> Result<Geometry, DumpError> {
        let mut geometry = Geometry::default();

        // The driver completes this request in its dispatch routine, in the
        // context of this thread.
        let status = self.control(
            ReadControl::GET_FLASH_GEOMETRY,
            ptr::null(),
            0,
            (&raw mut geometry).cast(),
            size_of::<Geometry>() as u32,
        );
        if !status.is_success() {
            return Err(DumpError::Geometry { status });
        }

        Ok(geometry)
    }

    /// READ_PAGES of [`DMA_SECTORS`] sectors from `first` into the data and
    /// spare pages of `buffers`. `first` must be aligned to the native page
    /// and the request must not cross a block.
    pub fn read_dma(&self, first: Sector, buffers: &mut DmaBuffers) -> Result<Completion, NtStatus> {
        self.pass_through(ReadControl::READ_PAGES, first, DMA_SECTORS, buffers)
    }

    /// READ_PAGE of `sector` into the start of the data and spare pages of
    /// `buffers`.
    pub fn read_polled(&self, sector: Sector, buffers: &mut DmaBuffers) -> Result<Completion, NtStatus> {
        self.pass_through(ReadControl::READ_PAGE, sector, 1, buffers)
    }

    fn pass_through(
        &self,
        control: ReadControl,
        first: Sector,
        sectors: u32,
        buffers: &mut DmaBuffers,
    ) -> Result<Completion, NtStatus> {
        const PASS_THROUGH_BYTES: u32 = size_of::<PassThrough>() as u32;

        let block = buffers.pass_through();
        let request = PassThrough {
            flash_address: first.flash_address(),
            buffer: buffers.data_address(),
            controller_config: 0,
            controller_status: 0,
            logical_address: 0,
            physical_address: 0,
            metadata_buffer: buffers.spare_address(),
            page_count: sectors,
        };
        // SAFETY: `block` points into the live physical allocation of
        // `buffers`, which the mutable borrow keeps exclusive.
        unsafe { block.write_volatile(request) };

        // The completion DPC writes the block, data and spare buffers, which
        // are all in the physical allocation.
        let status = self.control(control, block.cast(), PASS_THROUGH_BYTES, block.cast(), PASS_THROUGH_BYTES);
        if !status.is_success() {
            return Err(status);
        }

        // SAFETY: as above; the synchronous request has completed.
        let done = unsafe { block.read_volatile() };

        Ok(Completion {
            status: ControllerStatus(done.controller_status),
            config: ControllerConfig(done.controller_config),
        })
    }

    fn control(&self, control: ReadControl, input: *const u8, input_length: u32, output: *mut u8, output_length: u32) -> NtStatus {
        let ReadControl(code) = control;
        assert!(!is_write_control(code.0), "flash write control code {:#x}", code.0);

        let mut io_status = IoStatusBlock::default();

        // SAFETY: the handle is open for synchronous I/O, and the caller
        // passes buffers that stay valid until the request completes.
        unsafe {
            xboxkrnl::NtDeviceIoControlFile(
                self.handle,
                Handle::NULL,
                0,
                0,
                &mut io_status,
                code,
                input,
                input_length,
                output,
                output_length,
            )
        }
    }
}

impl Drop for FlashDevice {
    fn drop(&mut self) {
        // SAFETY: `self` owns the open handle.
        unsafe { xboxkrnl::NtClose(self.handle) };
    }
}

/// Whether `code` has the write access bit, which the driver treats as a
/// request that modifies the flash.
const fn is_write_control(code: u32) -> bool {
    const SFCX_WRITE_ACCESS: u32 = 2 << 14;

    code & SFCX_WRITE_ACCESS != 0
}
