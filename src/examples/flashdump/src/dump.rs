//! Reads every physical sector and writes its data and spare bytes.

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

use xbox360::sys::xboxkrnl;

use crate::dma::DmaBuffers;
use crate::error::DumpError;
use crate::extension;
use crate::extension::Extension;
use crate::flash::DMA_SECTORS;
use crate::flash::FlashDevice;
use crate::flash::ReadMethod;
use crate::flash::SECTOR_BYTES;
use crate::flash::SPARE_BYTES;
use crate::flash::Sector;
use crate::report::BlockCountSource;
use crate::report::DmaRejection;
use crate::report::PhysicalBlocks;
use crate::report::Report;

pub const BIN_PATH: &str = "\\Device\\Harddisk0\\Partition1\\DEVKIT\\flashraw.bin";
const START_PATH: &str = "\\Device\\Harddisk0\\Partition1\\DEVKIT\\flashdump.start";

pub fn run(report: &mut Report) -> Result<(), DumpError> {
    const OUTPUT_CHUNK_BYTES: usize = 1 << 20;
    const PROGRESS_BLOCKS: u32 = 8;

    report.kernel = xboxkrnl::XboxKrnlVersion.get().copied();

    let device = FlashDevice::open()?;
    let geometry = device.geometry()?;
    report.geometry = Some(geometry);
    println!(
        "flashdump: geometry {}+{} bytes, {} sectors per block, {} blocks, {} spare",
        geometry.bytes_per_sector,
        geometry.spare_bytes_per_sector,
        geometry.sectors_per_block,
        geometry.block_count,
        geometry.spare_block_count
    );
    if geometry.bytes_per_sector != SECTOR_BYTES || geometry.spare_bytes_per_sector != SPARE_BYTES {
        return Err(DumpError::SectorLayout {
            data_bytes: geometry.bytes_per_sector,
            spare_bytes: geometry.spare_bytes_per_sector,
        });
    }
    if geometry.sectors_per_block == 0 {
        return Err(DumpError::BlockLayout { sectors_per_block: 0 });
    }

    let mut buffers = DmaBuffers::new()?;

    // The first read returns the controller configuration that the
    // extension check compares.
    let probe = device
        .read_polled(Sector(0), &mut buffers)
        .map_err(|status| DumpError::PolledRead { sector: Sector(0), status })?;
    report.controller_config = Some(probe.config);

    report.device = extension::device_info();
    let location = extension::locate(report.device, report.kernel.map(|version| version.build));
    // SAFETY: the address is the static extension of the build 16547
    // kernel image, or the extension of the flash device object.
    let extension = location.map(|(base, source)| unsafe { Extension::read(base, source) });
    report.extension = extension;
    if let Some(extension) = extension {
        report.extension_mismatches = extension.mismatches(&geometry, probe.config, report.device);
    }

    // Only the extension holds the physical block count, which can exceed
    // the geometry block count. Its value is used only when every check of
    // the extension layout passes.
    let blocks = match extension {
        Some(extension) if report.extension_mismatches.is_empty() => PhysicalBlocks {
            count: u32::from(extension.physical_block_count),
            source: BlockCountSource::Extension,
        },
        _ => PhysicalBlocks { count: geometry.block_count, source: BlockCountSource::Geometry },
    };
    report.physical_blocks = Some(blocks);
    println!(
        "flashdump: config {:#010x} extension {:?} mismatches [{}] physical blocks {} from {:?}",
        probe.config.0,
        extension.map(|extension| extension.base),
        report.extension_mismatches.join(","),
        blocks.count,
        blocks.source
    );

    let sector_count = blocks.count * geometry.sectors_per_block;
    report.sector_count = Some(sector_count);
    let first_sector = read_start_block(blocks.count)? * geometry.sectors_per_block;
    report.first_sector = first_sector;

    // A group of DMA_SECTORS sectors is one 4 KB native-page-aligned DMA
    // request inside one block.
    report.dma_supported = geometry.sectors_per_block % DMA_SECTORS == 0;
    let mut method = if report.dma_supported { ReadMethod::Dma } else { ReadMethod::Polled };

    let file = File::create(BIN_PATH).map_err(|source| DumpError::Output { path: BIN_PATH, source })?;
    let mut out = BufWriter::with_capacity(OUTPUT_CHUNK_BYTES, file);
    let progress_sectors = PROGRESS_BLOCKS * geometry.sectors_per_block;

    let mut next = first_sector;
    while next < sector_count {
        let first = Sector(next);
        let count = DMA_SECTORS.min(sector_count - next);

        let dma = match method {
            ReadMethod::Dma => Some(device.read_dma(first, &mut buffers)),
            ReadMethod::Polled => None,
        };
        match dma {
            Some(Ok(done)) if !done.status.is_error() => {
                write_records(&mut out, &buffers, count)?;
            }
            Some(Ok(done)) => {
                report.errors.record(first, count, done.status, ReadMethod::Dma);
                read_polled(&device, &mut buffers, &mut out, report, first, count)?;
            }
            Some(Err(status)) => {
                println!("flashdump: READ_PAGES at sector {first} status {:#010x}, using READ_PAGE", status.0);
                report.dma_rejection = Some(DmaRejection { sector: first, status });
                method = ReadMethod::Polled;
                read_polled(&device, &mut buffers, &mut out, report, first, count)?;
            }
            None => {
                read_polled(&device, &mut buffers, &mut out, report, first, count)?;
            }
        }

        next += count;
        report.sectors_written = next - first_sector;
        if next % progress_sectors == 0 || next == sector_count {
            println!(
                "flashdump: block {}/{} error sectors {}",
                next / geometry.sectors_per_block,
                blocks.count,
                report.errors.polled_sectors()
            );
        }
    }

    out.flush().map_err(|source| DumpError::Output { path: BIN_PATH, source })?;

    Ok(())
}

/// The block to start at, from `START_PATH`, which is then removed so that the next dump starts
/// at block 0. Without the file the dump starts at block 0. A dump that stopped can be continued
/// from a later block and joined on the host.
fn read_start_block(block_count: u32) -> Result<u32, DumpError> {
    let text = match std::fs::read_to_string(START_PATH) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(source) => return Err(DumpError::Output { path: START_PATH, source }),
    };
    std::fs::remove_file(START_PATH).map_err(|source| DumpError::Output { path: START_PATH, source })?;

    match text.trim().parse::<u32>() {
        Ok(block) if block < block_count => Ok(block),
        _ => Err(DumpError::StartBlock { path: START_PATH, text, block_count }),
    }
}

/// Reads `count` sectors from `first` one at a time and writes them.
fn read_polled(
    device: &FlashDevice,
    buffers: &mut DmaBuffers,
    out: &mut impl Write,
    report: &mut Report,
    first: Sector,
    count: u32,
) -> Result<(), DumpError> {
    for index in 0..count {
        let sector = Sector(first.0 + index);
        let done = device
            .read_polled(sector, buffers)
            .map_err(|status| DumpError::PolledRead { sector, status })?;
        if done.status.is_error() {
            report.errors.record(sector, 1, done.status, ReadMethod::Polled);
        }

        write_records(out, buffers, 1)?;
    }

    Ok(())
}

/// Writes the first `count` sectors of the last request as data then
/// spare bytes.
fn write_records(out: &mut impl Write, buffers: &DmaBuffers, count: u32) -> Result<(), DumpError> {
    let output = |source| DumpError::Output { path: BIN_PATH, source };

    for index in 0..count {
        out.write_all(buffers.data(index)).map_err(output)?;
        out.write_all(buffers.spare(index)).map_err(output)?;
    }

    Ok(())
}
