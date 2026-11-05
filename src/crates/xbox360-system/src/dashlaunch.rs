//! DashLaunch runtime state.

use core::num::NonZeroU32;

use crate::console::kernel_version;
use crate::module::Ordinal;
use crate::module::module_handle;
use crate::module::resolve_export;

mod offset {
    pub const ID: u32 = 0x00;
    pub const TARGET_KERNEL: u32 = 0x214;
    pub const DEBUG_ROUTINE: u32 = 0x21c;
    pub const DEBUG_STEP_PATCH: u32 = 0x220;
}

const LAUNCH_DATA_ID: u32 = u32::from_be_bytes(*b"DL30");
const LAUNCH_DATA: Ordinal = match Ordinal::new(1) {
    Some(ordinal) => ordinal,
    None => panic!("ordinal 0"),
};

/// Validated `launch.xex` export 1 data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LaunchData(NonZeroU32);

impl LaunchData {
    /// Resolve the data for the running kernel.
    pub fn current() -> Option<Self> {
        // SAFETY: DashLaunch keeps launch.xex loaded while a plugin runs.
        let module = unsafe { module_handle(c"launch.xex") }.ok()?;
        let address = NonZeroU32::new(resolve_export(module, LAUNCH_DATA).ok()?.raw())?;
        let data = Self(address);

        (data.read_u32(offset::ID) == LAUNCH_DATA_ID
            && data.read_u16(offset::TARGET_KERNEL) == kernel_version().build)
            .then_some(data)
    }

    /// Address of the kernel `KiDebugRoutine` pointer resolved by DashLaunch.
    pub fn debug_routine_variable(self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.read_u32(offset::DEBUG_ROUTINE))
    }

    /// Address of the instruction DashLaunch identifies for enabling single-step exceptions.
    pub fn single_step_patch(self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.read_u32(offset::DEBUG_STEP_PATCH))
    }

    fn read_u32(self, offset: u32) -> u32 {
        let address = self.0.get() + offset;
        // SAFETY: fields of launch data stay mapped and have their natural alignment.
        unsafe { core::ptr::with_exposed_provenance::<u32>(address as usize).read_volatile() }
    }

    fn read_u16(self, offset: u32) -> u16 {
        let address = self.0.get() + offset;
        // SAFETY: fields of launch data stay mapped and have their natural alignment.
        unsafe { core::ptr::with_exposed_provenance::<u16>(address as usize).read_volatile() }
    }
}
