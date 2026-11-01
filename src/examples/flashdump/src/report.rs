//! The text summary written beside the dump.

use std::fmt::Write;

use xbox360::sys::xboxkrnl::KernelVersion;
use xbox360::sys::xboxkrnl::NtStatus;

use crate::error::DumpError;
use crate::extension;
use crate::extension::DeviceInfo;
use crate::extension::Extension;
use crate::flash::ControllerConfig;
use crate::flash::ControllerStatus;
use crate::flash::Geometry;
use crate::flash::ReadMethod;
use crate::flash::SECTOR_BYTES;
use crate::flash::SPARE_BYTES;
use crate::flash::Sector;

/// Where the dumped physical block count came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCountSource {
    /// `SFCX_DEVICE_EXTENSION::PhysicalBlockCount`, which passed every
    /// consistency check.
    Extension,
    /// `SFCX_FLASH_GEOMETRY::BlockCount`.
    Geometry,
}

impl BlockCountSource {
    const fn name(self) -> &'static str {
        match self {
            Self::Extension => "extension",
            Self::Geometry => "geometry",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicalBlocks {
    pub count: u32,
    pub source: BlockCountSource,
}

/// The READ_PAGES request the driver rejected, after which every sector is
/// read with READ_PAGE.
#[derive(Debug, Clone, Copy)]
pub struct DmaRejection {
    pub sector: Sector,
    pub status: NtStatus,
}

/// Consecutive sectors that one read method returned with the same error
/// status.
#[derive(Debug, Clone, Copy)]
struct ErrorRun {
    first: Sector,
    count: u32,
    status: ControllerStatus,
    method: ReadMethod,
}

/// Sectors whose controller status has an error bit. A READ_PAGES request
/// reports one status for its sectors, which are then read again with
/// READ_PAGE; both are listed.
#[derive(Debug, Default)]
pub struct ErrorLog {
    runs: Vec<ErrorRun>,
}

impl ErrorLog {
    pub fn record(&mut self, first: Sector, count: u32, status: ControllerStatus, method: ReadMethod) {
        if let Some(last) = self.runs.last_mut()
            && last.method == method
            && last.status == status
            && last.first.0 + last.count == first.0
        {
            last.count += count;
            return;
        }

        self.runs.push(ErrorRun { first, count, status, method });
    }

    /// Sectors that READ_PAGE returned with an error status.
    pub fn polled_sectors(&self) -> u32 {
        self.runs.iter().filter(|run| run.method == ReadMethod::Polled).map(|run| run.count).sum()
    }

    fn dma_sectors(&self) -> u32 {
        self.runs.iter().filter(|run| run.method == ReadMethod::Dma).map(|run| run.count).sum()
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub kernel: Option<KernelVersion>,
    pub geometry: Option<Geometry>,
    /// `ControllerConfig` that the first read request returned.
    pub controller_config: Option<ControllerConfig>,
    pub device: Option<DeviceInfo>,
    pub extension: Option<Extension>,
    pub extension_mismatches: Vec<&'static str>,
    pub physical_blocks: Option<PhysicalBlocks>,
    pub sector_count: Option<u32>,
    /// First sector of the dump; earlier sectors are not in the output.
    pub first_sector: u32,
    pub sectors_written: u32,
    pub dma_supported: bool,
    pub dma_rejection: Option<DmaRejection>,
    pub errors: ErrorLog,
}

impl Report {
    pub fn render(&self, outcome: &Result<(), DumpError>) -> String {
        let mut text = String::new();

        // Writing to a String does not fail.
        let _ = self.render_into(&mut text, outcome);

        text
    }

    fn render_into(&self, text: &mut String, outcome: &Result<(), DumpError>) -> std::fmt::Result {
        match outcome {
            Ok(()) => writeln!(text, "result ok")?,
            Err(error) => writeln!(text, "result error {error}")?,
        }
        if let Some(kernel) = self.kernel {
            writeln!(text, "kernel {kernel}")?;
        }
        writeln!(text, "record_bytes {}", SECTOR_BYTES + SPARE_BYTES)?;

        if let Some(geometry) = self.geometry {
            writeln!(text, "geometry_bytes_per_sector {}", geometry.bytes_per_sector)?;
            writeln!(text, "geometry_spare_bytes_per_sector {}", geometry.spare_bytes_per_sector)?;
            writeln!(text, "geometry_sectors_per_block {}", geometry.sectors_per_block)?;
            writeln!(text, "geometry_block_count {}", geometry.block_count)?;
            writeln!(text, "geometry_spare_block_count {}", geometry.spare_block_count)?;
        }
        if let Some(config) = self.controller_config {
            writeln!(text, "controller_config {:#010x}", config.0)?;
        }

        if let Some(device) = self.device {
            writeln!(text, "device_extension {:#010x}", device.extension)?;
            writeln!(text, "device_alignment_requirement {:#x}", device.alignment_requirement)?;
        }
        writeln!(text, "build_16547_extension {:#010x}", extension::BUILD_16547_BASE)?;
        if let Some(extension) = self.extension {
            writeln!(text, "extension_base {:#010x}", extension.base)?;
            writeln!(text, "extension_source {}", extension.source.name())?;
            writeln!(text, "extension_controller_config {:#010x}", extension.controller_config.0)?;
            writeln!(text, "extension_block_size {:#x}", extension.block_size)?;
            writeln!(text, "extension_page_size {:#x}", extension.page_size)?;
            writeln!(text, "extension_pages_per_block {}", extension.pages_per_block)?;
            writeln!(text, "extension_physical_block_count {}", extension.physical_block_count)?;
            writeln!(text, "extension_block_count {}", extension.block_count)?;
            writeln!(text, "extension_spare_block_count {}", extension.spare_block_count)?;
            writeln!(text, "extension_logical_block_count {}", extension.logical_block_count)?;
            writeln!(text, "extension_metadata_type {}", extension.metadata_type)?;
            writeln!(text, "extension_page_shift {}", extension.page_shift)?;
            writeln!(text, "extension_block_shift {}", extension.block_shift)?;
            writeln!(text, "extension_sectors_per_page {}", extension.sectors_per_page)?;
            if let Some(bytes) = extension.config_flash_bytes() {
                writeln!(text, "config_flash_bytes {bytes:#x}")?;
            }
            writeln!(text, "extension_mismatches {}", self.extension_mismatches.join(","))?;
        }

        if let Some(blocks) = self.physical_blocks {
            writeln!(text, "physical_block_count {}", blocks.count)?;
            writeln!(text, "physical_block_count_source {}", blocks.source.name())?;
        }
        if let Some(count) = self.sector_count {
            writeln!(text, "sector_count {count}")?;
        }
        writeln!(text, "first_sector {}", self.first_sector)?;
        writeln!(text, "sectors_written {}", self.sectors_written)?;
        writeln!(text, "dma_supported {}", u8::from(self.dma_supported))?;
        if let Some(rejection) = self.dma_rejection {
            writeln!(text, "dma_rejected_sector {}", rejection.sector)?;
            writeln!(text, "dma_rejected_status {:#010x}", rejection.status.0)?;
        }

        writeln!(text, "dma_error_sectors {}", self.errors.dma_sectors())?;
        writeln!(text, "polled_error_sectors {}", self.errors.polled_sectors())?;
        writeln!(text, "error_runs {}", self.errors.runs.len())?;
        writeln!(text, "# first_sector sector_count controller_status method")?;
        for run in &self.errors.runs {
            writeln!(
                text,
                "error {} {} {:#010x} {}",
                run.first,
                run.count,
                run.status.0,
                run.method.name()
            )?;
        }

        Ok(())
    }
}
