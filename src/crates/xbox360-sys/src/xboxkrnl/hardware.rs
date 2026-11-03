//! System management controller, processor registers and console hardware information.

use core::sync::atomic::AtomicU8;

use super::field::KernelU32;

value_enum! {
    /// `HalReturnToFirmware` routine.
    #[non_exhaustive]
    pub enum FirmwareRequest: u32 {
        /// `HalRebootRoutine`, the routine Xenia asserts on.
        Reboot = 1,
        /// Reset through the system management controller.
        SmcReset = 4,
        /// Restart the system software. Stock XBDM uses it for a reboot.
        RebootQuiesce = 6,
    }
}

value_enum! {
    /// Byte 0 of an [`SmcMessage`]: the command of a request to the system
    /// management controller, or the kind of a message it sends. Names
    /// follow SMC firmware 510301; [`SmcRequest`](super::SmcRequest)
    /// encodes the typed requests.
    #[non_exhaustive]
    pub enum SmcCommand: u8 {
        /// Query of the power-up cause; reply
        /// [`PowerUpCause`](super::PowerUpCause).
        QueryPowerUpCause = 0x01,
        /// Query of the temperatures; reply
        /// [`Temperatures`](super::Temperatures).
        QueryTemperatures = 0x07,
        /// An I2C transfer.
        I2c = 0x11,
        /// Query of the firmware version; reply
        /// [`SmcVersion`](super::SmcVersion).
        QueryVersion = 0x12,
        /// Query of the IR receiver address. The low nibble of reply byte 1
        /// is the address.
        QueryIrAddress = 0x16,
        /// The optical drive tray state. Byte 1 is 0x60 plus the state.
        TrayState = 0x83,
        /// Sets the [`FanAlgorithm`](super::FanAlgorithm).
        SetFanAlgorithm = 0x88,
        /// Sets the [`FanSpeed`](super::FanSpeed) of the fan.
        SetFanSpeed = 0x89,
        /// Sets the [`PowerLed`](super::PowerLed) mode.
        SetPowerLed = 0x8c,
        /// Sets the [`FanSpeed`](super::FanSpeed) of the second fan output.
        /// Only some SMC builds have it (21010d does; 510301 ignores it).
        SetFan2Speed = 0x94,
        /// Sets the IR receiver address to the low nibble of byte 1.
        SetIrAddress = 0x95,
        /// Sets the [`RingOfLight`](super::RingOfLight).
        SetRingOfLight = 0x99,
    }
}

impl SmcCommand {
    /// Whether the SMC replies: commands below 0x80 are queries. The SMC
    /// does not answer a query it does not know.
    pub const fn is_query(self) -> bool {
        self.raw() < 0x80
    }
}

/// A 16-byte message to or from the system management controller. Byte 0
/// is the [`SmcCommand`].
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SmcMessage(pub [u8; 16]);

impl SmcMessage {
    /// A message of `command` with the other bytes zero.
    pub const fn new(command: SmcCommand) -> Self {
        let mut bytes = [0; 16];
        bytes[0] = command.raw();
        Self(bytes)
    }

    /// `None` for a command without a variant.
    pub const fn command(&self) -> Option<SmcCommand> {
        SmcCommand::from_raw(self.0[0])
    }
}

/// Bus address of the device of an [`SmcCommand::I2c`] transfer.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SmcI2cDevice(pub u8);

impl SmcI2cDevice {
    /// The EEPROM that holds the debug monitor settings block.
    pub const SETTINGS_EEPROM: Self = Self(0xd4);
}

/// A special purpose register of `KeGetSpecialPurposeRegister` and
/// `KeSetSpecialPurposeRegister`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpecialPurposeRegister(pub u32);

impl SpecialPurposeRegister {
    /// Data address breakpoint register.
    pub const DABR: Self = Self(0x3f5);
    /// Performance monitor control: bit 1 freezes and bit 2 enables the
    /// counters; bits 10 to 15 select the event bus width.
    pub const MONITOR_CONTROL: Self = Self(0x3f6);
    /// Performance monitor event selection.
    pub const EVENT_SELECT: Self = Self(0x3fa);
}

/// A performance monitor bus register of `KeGetPMWRegister` and
/// `KeSetPMWRegister`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PmwRegister(pub u32);

impl PmwRegister {
    /// Lane sources of the bus units.
    pub const NCU0: Self = Self(0x480);
    pub const NCU1: Self = Self(0x580);
    pub const NCU2: Self = Self(0x680);
    pub const CIU: Self = Self(0x708);
    pub const BIU: Self = Self(0xb50);
    /// L2 event group selection.
    pub const L2_GROUP: Self = Self(0x310);
    pub const L2: Self = Self(0x318);
    /// L2 ECC error counts.
    pub const L2_ECC: Self = Self(0x860);
}

/// A performance monitor register of `KeSetPRVRegister`. Values are in
/// the high word.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrvRegister(pub u32);

impl PrvRegister {
    /// Counter run control.
    pub const RUN: Self = Self(0x1c0);
    /// Interval timer control.
    pub const INTERVAL_CONTROL: Self = Self(0x1c8);
    /// Processor whose events trigger the counters.
    pub const TRIGGER: Self = Self(0x1d8);
    pub const INTERVAL: Self = Self(0x1e0);
    pub const INTERVAL_RELOAD: Self = Self(0x1e8);

    /// The value register of counter `counter`. Panics unless `counter` is
    /// 0 to 15.
    pub const fn counter_value(counter: u8) -> Self {
        assert!(counter < 16, "the performance monitor has 16 counters");
        Self(0x200 + counter as u32 * 8)
    }

    /// The control register of counter `counter`. Panics unless `counter`
    /// is 0 to 15.
    pub const fn counter_control(counter: u8) -> Self {
        assert!(counter < 16, "the performance monitor has 16 counters");
        Self(0x280 + counter as u32 * 8)
    }
}

flags! {
    /// The flags word of `XboxHardwareInfo`. The top four bits hold the
    /// [`HardwareGeneration`].
    pub struct HardwareFlags(u32) {
        /// The video driver detected the [`HardwareGeneration`]; without
        /// it the generation reads as Xenon.
        GENERATION_DETECTED = 0x4,
        /// A development kit kernel: set by the static initializer of
        /// development kit kernels and clear in retail kernels.
        DEVKIT_KERNEL = 0x10,
        HARD_DISK = 0x20,
        /// The kernel debugger is enabled at boot.
        KERNEL_DEBUGGER_ENABLED = 0x80,
        /// A kernel debugger is attached.
        KERNEL_DEBUGGER = 0x100,
        /// The console has 1 GB of memory, as some development kits do.
        ONE_GIGABYTE = 0x0001_0000,
        /// Test kit, as XBDM reads it. No kernel sets it, and a clear bit
        /// does not tell a development kit from a retail console.
        TEST_KIT = 0x0200_0000,
    }
}

impl HardwareFlags {
    /// The hardware generation in the top four bits.
    pub const fn generation(self) -> HardwareGeneration {
        HardwareGeneration((self.0 >> 28) as u8)
    }

    /// The platform in bits 0-1.
    pub const fn platform(self) -> HardwarePlatform {
        match HardwarePlatform::from_raw((self.0 & 0x3) as u8) {
            Some(platform) => platform,
            // Two bits have four values, and each has a variant.
            None => unreachable!(),
        }
    }
}

/// Kind of console.
///
/// Stock XBDM reports further kinds, such as a review kit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KitType {
    DevelopmentKit,
    TestKit,
    Retail,
}

impl KitType {
    /// A test kit with [`HardwareFlags::TEST_KIT`], as XBDM reads it, else
    /// a development kit with [`HardwareFlags::DEVKIT_KERNEL`]. `None`
    /// under a retail kernel: the flags do not tell a retail console from
    /// a kit that runs a retail kernel. `xbox360_system::console::kit_type`
    /// checks more sources.
    pub const fn from_hardware_flags(flags: HardwareFlags) -> Option<Self> {
        if flags.contains(HardwareFlags::TEST_KIT) {
            Some(Self::TestKit)
        } else if flags.contains(HardwareFlags::DEVKIT_KERNEL) {
            Some(Self::DevelopmentKit)
        } else {
            None
        }
    }
}

/// Hardware generation (motherboard) number from the top four bits of
/// [`HardwareFlags`]. [`HardwareGeneration::name`] names the known ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HardwareGeneration(u8);

impl HardwareGeneration {
    pub const XENON: Self = Self(0);
    pub const ZEPHYR: Self = Self(1);
    pub const FALCON: Self = Self(2);
    pub const JASPER: Self = Self(3);
    /// The Trinity motherboard, the first Xbox 360 S.
    pub const TRINITY: Self = Self(4);
    pub const CORONA: Self = Self(5);
    pub const WINCHESTER: Self = Self(6);
    pub const RIDGEWAY: Self = Self(8);

    /// A generation number. `None` above 15.
    pub const fn new(number: u8) -> Option<Self> {
        if number <= 0xf {
            Some(Self(number))
        } else {
            None
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    /// The motherboard name, as XBDM reports it. `None` for a number
    /// without a known name.
    pub const fn name(self) -> Option<&'static str> {
        match self.0 {
            0 => Some("Xenon"),
            1 => Some("Zephyr"),
            2 => Some("Falcon"),
            3 => Some("Jasper"),
            4 => Some("Trinity"),
            5 => Some("Corona"),
            6 => Some("Winchester"),
            8 => Some("Ridgeway"),
            _ => None,
        }
    }
}

value_enum! {
    /// Platform in bits 0-1 of [`HardwareFlags`], as XBDM names it.
    pub enum HardwarePlatform: u8 {
        Alpha = 0,
        Mongrel = 1,
        Shiva = 2,
        Waternoose = 3,
    }
}

flags! {
    /// Boot flags of [`HardwareInfo`], which the boot loaders and the
    /// hypervisor set. SB and SD are the second and fourth stage boot
    /// loaders. Bit 0x800 is tested by SB and has no known meaning.
    pub struct HardwareBootFlags(u16) {
        /// SB found its console binding all zero and skipped the SMC
        /// binding check: an unbound boot loader.
        UNBOUND_BOOT_LOADER = 0x1,
        /// Fuse row 0 bits 0-55 are not zero.
        LOCK_FUSES_BLOWN = 0x2,
        /// Fuse row 1 holds the console type that SB records.
        CONSOLE_TYPE_MATCH = 0x4,
        /// The fuse row 2 version count equals the one SB records.
        FUSE_VERSION_MATCH = 0x8,
        /// Fuse rows 3-6 hold a valid console key.
        CONSOLE_KEY_VALID = 0x10,
        /// Memory above 512 MB, as development kits have.
        DEVKIT_MEMORY = 0x80,
        /// Kernel debugger lite only; what sets it is not established.
        KD_LITE_ONLY = 0x100,
        /// The kernel was started by a shadow boot.
        SHADOW_BOOTED = 0x200,
        /// Shadow boot from CD-ROM: `\SystemRoot` is the CD-ROM.
        CDROM_SYSTEM_ROOT = 0x400,
        /// SD loaded the first flash patch slot.
        FLASH_PATCH1 = 0x1000,
        /// SD loaded the second flash patch slot.
        FLASH_PATCH2 = 0x2000,
        /// The loaded patch requests the next update sequence.
        UPDATE_SEQUENCE_PENDING = 0x4000,
        /// The update sequence fuses hold the patch's sequence.
        UPDATE_SEQUENCE_PROGRAMMED = 0x8000,
    }
}

/// `XboxHardwareInfo`: console hardware information.
#[repr(C)]
pub struct HardwareInfo {
    /// [`HardwareFlags`] word. The debug monitor changes it while the
    /// system runs, and the kernel acts on it.
    pub flags: KernelU32,
    pub number_of_processors: u8,
    pub revision: u8,
    _reserved_006: [AtomicU8; 6],
    /// Value the boot loader passes to the kernel with the boot flags. Its
    /// meaning is not established.
    pub boot_loader_data: u16,
    pub boot_flags: HardwareBootFlags,
}

const _: () = {
    assert!(core::mem::offset_of!(HardwareInfo, number_of_processors) == 0x4);
    assert!(core::mem::offset_of!(HardwareInfo, boot_loader_data) == 0xc);
    assert!(core::mem::offset_of!(HardwareInfo, boot_flags) == 0xe);
    assert!(core::mem::size_of::<HardwareInfo>() == 0x10);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_flags_give_generation_and_platform() {
        // A development kit with a Corona motherboard.
        let flags = HardwareFlags::from_bits_retain(0x5000_0813);

        assert_eq!(flags.generation(), HardwareGeneration::CORONA);
        assert_eq!(flags.generation().name(), Some("Corona"));
        assert_eq!(flags.platform(), HardwarePlatform::Waternoose);
        assert!(flags.contains(HardwareFlags::DEVKIT_KERNEL));
        assert_eq!(
            HardwareGeneration::new(7).and_then(HardwareGeneration::name),
            None
        );
        assert_eq!(HardwareGeneration::new(16), None);
    }

    #[test]
    fn smc_queries_are_below_0x80() {
        assert!(SmcCommand::QueryTemperatures.is_query());
        assert!(!SmcCommand::SetRingOfLight.is_query());
    }

    #[test]
    fn kit_type_from_hardware_flags() {
        let devkit = HardwareFlags::from_bits_retain(0x4000_0813);
        let retail = HardwareFlags::from_bits_retain(0x4000_0803);

        assert_eq!(
            KitType::from_hardware_flags(devkit),
            Some(KitType::DevelopmentKit)
        );
        assert_eq!(
            KitType::from_hardware_flags(devkit | HardwareFlags::TEST_KIT),
            Some(KitType::TestKit)
        );
        assert_eq!(KitType::from_hardware_flags(retail), None);
    }
}
