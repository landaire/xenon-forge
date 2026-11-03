//! Kernel version and clock times.

use core::num::NonZeroU32;

use super::field::KernelU32;

/// A kernel version, as `XboxKrnlVersion` and `XboxKrnlBaseVersion` hold
/// it. `qfe` is the raw field: kernels set bit 15 in it, and the dashboard
/// prints [`KernelVersion::qfe_number`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KernelVersion {
    pub major: u16,
    pub minor: u16,
    pub build: u16,
    pub qfe: u16,
}

impl KernelVersion {
    const QFE_FLAG: u16 = 0x8000;

    /// The QFE number: `qfe` without bit 15, as dashboard 2.0.6717 prints
    /// it (`Qfe & 0x7FFF`).
    pub const fn qfe_number(self) -> u16 {
        self.qfe & !Self::QFE_FLAG
    }

    /// Bit 15 of `qfe`. What it means is not established.
    pub const fn qfe_flag(self) -> bool {
        self.qfe & Self::QFE_FLAG != 0
    }
}

impl core::fmt::Display for KernelVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.build, self.qfe
        )
    }
}

const _: () = assert!(core::mem::size_of::<KernelVersion>() == 8);

/// `KeTimeStampBundle`: times that the kernel updates on each clock tick.
#[repr(C)]
pub struct TimeStampBundle {
    _times: [KernelU32; 4],
    /// Milliseconds since boot.
    pub tick_count: KernelU32,
    _reserved_14: KernelU32,
}

const _: () = {
    assert!(core::mem::offset_of!(TimeStampBundle, tick_count) == 0x10);
    assert!(core::mem::size_of::<TimeStampBundle>() == 0x18);
};

/// Identifier of an installed kernel expansion, the first argument of
/// `ExExpansionCall`. Never zero: the kernel would branch to a null routine
/// for id 0.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExpansionId(NonZeroU32);

impl ExpansionId {
    /// Largest image `ExExpansionInstall` accepts.
    pub const MAX_IMAGE_SIZE: usize = 0x10000;
    /// The multiple an image size must be.
    pub const IMAGE_SIZE_ALIGNMENT: usize = 128;

    pub const fn new(id: NonZeroU32) -> Self {
        Self(id)
    }

    /// `None` for 0.
    pub const fn from_u32(id: u32) -> Option<Self> {
        match NonZeroU32::new(id) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use super::KernelVersion;

    #[test]
    fn qfe_number_leaves_out_bit_15() {
        let flagged = KernelVersion { major: 2, minor: 0, build: 16547, qfe: 0x8000 | 327 };
        assert_eq!((flagged.qfe_number(), flagged.qfe_flag()), (327, true));

        let plain = KernelVersion { qfe: 327, ..flagged };
        assert_eq!((plain.qfe_number(), plain.qfe_flag()), (327, false));
    }
}
