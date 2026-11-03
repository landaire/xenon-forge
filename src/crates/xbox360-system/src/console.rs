//! Kernel versions, hardware flags, boot flags, the kit type, the console
//! certificate type and firmware requests.

use core::sync::atomic::Ordering;

use xbox360_status::KernelError;
use xbox360_sys::xboxkrnl;

pub use xbox360_sys::xboxkrnl::CertificateConsoleType;
pub use xbox360_sys::xboxkrnl::FirmwareRequest;
pub use xbox360_sys::xboxkrnl::HardwareBootFlags;
pub use xbox360_sys::xboxkrnl::HardwareFlags;
pub use xbox360_sys::xboxkrnl::HardwareGeneration;
pub use xbox360_sys::xboxkrnl::HardwarePlatform;
pub use xbox360_sys::xboxkrnl::KernelVersion;
pub use xbox360_sys::xboxkrnl::KitType;

use crate::module;

/// Bit of the `XeKeysGetStatus` value (the hypervisor runtime flags) set
/// by a retail-mode hypervisor.
const KEYS_STATUS_RETAIL: u32 = 0x8000;

/// The xbdm.xex export that stores 1 for a test kit and 0 for a
/// development kit: `HRESULT (PULONG)`.
const XBDM_TEST_KIT_ORDINAL: u16 = 140;

/// The kind of console, `None` when no source tells. In order: the answer
/// of XBDM when it is loaded (its export 140), a retail-mode hypervisor
/// (`XeKeysGetStatus` bit 0x8000), then [`KitType::from_hardware_flags`].
pub fn kit_type() -> Option<KitType> {
    if let Some(kit) = xbdm_kit_type() {
        return Some(kit);
    }

    let mut status = 0u32;

    // SAFETY: `status` is live local storage of the u32 the export writes.
    unsafe { xboxkrnl::XeKeysGetStatus(&mut status) };
    if status & KEYS_STATUS_RETAIL != 0 {
        return Some(KitType::Retail);
    }

    KitType::from_hardware_flags(hardware_flags())
}

/// The answer of XBDM export 140, `None` when XBDM is not loaded, lacks
/// the export or fails.
fn xbdm_kit_type() -> Option<KitType> {
    // SAFETY: XBDM is a system module that stays loaded until the console
    // restarts.
    let xbdm = unsafe { module::module_handle(c"xbdm.xex") }.ok()?;
    let ordinal = module::Ordinal::new(XBDM_TEST_KIT_ORDINAL)?;
    let export = module::resolve_export(xbdm, ordinal).ok()?;
    let mut test_kit = 0u32;

    // SAFETY: export 140 of XBDM takes one pointer to a u32 it writes.
    let result =
        unsafe { call_console_1((&raw mut test_kit).expose_provenance() as u32, export.raw()) };
    if (result as i32) < 0 {
        return None;
    }

    Some(if test_kit != 0 {
        KitType::TestKit
    } else {
        KitType::DevelopmentKit
    })
}

/// Calls the console function at `function` with r3. Console callees store
/// their arguments in the caller's frame from sp+0x10 to sp+0x50, so the
/// call runs in a 0x50-byte frame of its own.
///
/// # Safety
///
/// `function` must be the code address of a function that takes one 32-bit
/// argument, and `argument` must meet its requirements.
#[unsafe(naked)]
unsafe extern "C" fn call_console_1(argument: u32, function: u32) -> u32 {
    core::arch::naked_asm!(
        "stwu 1, -0x50(1)",
        "mflr 0",
        "stw 0, 0x54(1)",
        "mtctr 4",
        "bctrl",
        "lwz 0, 0x54(1)",
        "mtlr 0",
        "addi 1, 1, 0x50",
        "blr",
    )
}

/// Version of the running kernel (`XboxKrnlVersion`).
///
/// # Panics
///
/// If the loader did not resolve the import.
pub fn kernel_version() -> KernelVersion {
    match xboxkrnl::XboxKrnlVersion.get() {
        Some(version) => *version,
        None => xbox360_status::unresolved_import("XboxKrnlVersion"),
    }
}

/// Version of the base kernel (`XboxKrnlBaseVersion`). `None` when the
/// loader left the import unresolved, as Xenia does.
pub fn base_kernel_version() -> Option<KernelVersion> {
    xboxkrnl::XboxKrnlBaseVersion.get().copied()
}

/// The hardware flags of the console.
///
/// # Panics
///
/// If the loader did not resolve `XboxHardwareInfo`.
pub fn hardware_flags() -> HardwareFlags {
    match xboxkrnl::XboxHardwareInfo.get() {
        Some(info) => HardwareFlags::from_bits_retain(info.flags.load(Ordering::Relaxed)),
        None => xbox360_status::unresolved_import("XboxHardwareInfo"),
    }
}

/// The boot flags of the console.
///
/// # Panics
///
/// If the loader did not resolve `XboxHardwareInfo`.
pub fn boot_flags() -> HardwareBootFlags {
    match xboxkrnl::XboxHardwareInfo.get() {
        Some(info) => info.boot_flags,
        None => xbox360_status::unresolved_import("XboxHardwareInfo"),
    }
}

/// The console type field of the console certificate
/// (`XeKeysGetConsoleType`). Which values mean which kind of console is
/// not established; [`kit_type`] tells the kind of console.
pub fn certificate_console_type() -> Result<CertificateConsoleType, KernelError> {
    let mut console_type = CertificateConsoleType::default();

    // SAFETY: `console_type` is live local storage of the u32 the export
    // writes.
    unsafe { xboxkrnl::XeKeysGetConsoleType(&mut console_type) }.result()?;

    Ok(console_type)
}

/// Hand the console to the firmware. Does not return.
pub fn return_to_firmware(request: FirmwareRequest) -> ! {
    // SAFETY: FirmwareRequest admits only the routines the kernel accepts;
    // the call takes no pointers.
    unsafe { xboxkrnl::HalReturnToFirmware(request) };

    // HalReturnToFirmware does not return.
    xbox360_thread::park()
}
