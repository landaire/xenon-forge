use xbox360::sys::xboxkrnl::NtStatus;

use crate::flash::Sector;

#[derive(Debug, thiserror::Error)]
pub enum DumpError {
    #[error("open {path}: status {:#010x}", .status.0)]
    Open { path: &'static str, status: NtStatus },

    #[error("flash geometry request: status {:#010x}", .status.0)]
    Geometry { status: NtStatus },

    #[error("flash geometry has {data_bytes} data and {spare_bytes} spare bytes per sector")]
    SectorLayout { data_bytes: u32, spare_bytes: u32 },

    #[error("flash geometry has {sectors_per_block} sectors per block")]
    BlockLayout { sectors_per_block: u32 },

    #[error("physical memory allocation of {bytes} bytes failed")]
    Allocation { bytes: u32 },

    #[error("READ_PAGE of sector {sector}: status {:#010x}", .status.0)]
    PolledRead { sector: Sector, status: NtStatus },

    #[error("{path}: start block {text:?} is not a block number below {block_count}")]
    StartBlock { path: &'static str, text: String, block_count: u32 },

    #[error("{path}: {source}")]
    Output {
        path: &'static str,
        #[source]
        source: std::io::Error,
    },
}
