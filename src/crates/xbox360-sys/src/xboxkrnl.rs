//! xboxkrnl.exe imports, kernel structures, and the types and constants of
//! their arguments.
//!
//! Kernel structures that Rust code only reads or that the kernel shares,
//! such as [`Kthread`], are views of the fields in use. A field that the
//! kernel or another processor changes is an atomic, and the bytes between
//! the fields are private. A field that the kernel acts on is a
//! [`KernelU32`] or [`KernelU8`], which safe code can only read.

mod audio;
mod boolean;
mod crypt;
mod debug;
mod device;
mod exception;
mod field;
mod file;
mod handle;
mod hardware;
mod loader;
mod memory;
mod network;
mod object;
mod string;
mod system;
mod thread;
mod video;

#[cfg(any(target_os = "xbox360", xbox360_docs))]
use crate::image::ImageDirectory;
#[cfg(any(target_os = "xbox360", xbox360_docs))]
use crate::image::ImageNtHeaders;
#[cfg(any(target_os = "xbox360", xbox360_docs))]
use crate::image::XexHeaderKey;

pub use self::audio::XmaContextData;
pub use self::audio::XmaContextInit;
pub use self::audio::XmaLoopData;
pub use self::boolean::Boolean;
pub use self::boolean::Boolean8;
pub use self::crypt::DesState;
pub use self::debug::CdromChannelInformation;
pub use self::debug::DebugMemoryStatistics;
pub use self::debug::DebugMonitorData;
pub use self::debug::DebugMonitorFlags;
pub use self::debug::DebugMonitorService;
pub use self::device::DeviceControlParameters;
pub use self::device::DeviceObject;
pub use self::device::DeviceObjectFlags;
pub use self::device::DiskGeometry;
pub use self::device::DriverDispatch;
pub use self::device::DriverObject;
pub use self::device::DriverStartIo;
pub use self::device::FileDeviceType;
pub use self::device::IoControlCode;
pub use self::device::IoStackControl;
pub use self::device::IoStackFlags;
pub use self::device::IoStackLocation;
pub use self::device::IoStackParameters;
pub use self::device::Irp;
pub use self::device::IrpMajorFunction;
pub use self::device::PartitionInformation;
pub use self::device::ReadWriteParameters;
pub use self::device::ScsiInquiryData;
pub use self::device::ScsiPassThroughDirect;
pub use self::exception::BugCheckCode;
pub use self::exception::Context;
pub use self::exception::ContextFlags;
pub use self::exception::ExceptionFlags;
pub use self::exception::ExceptionRecord;
pub use self::exception::KswitchFrame;
pub use self::exception::KtrapFrame;
pub use self::field::KernelU8;
pub use self::field::KernelU32;
pub use self::field::ListEntry;
pub use self::file::CreateDisposition;
pub use self::file::CreateOptions;
pub use self::file::FileAttributes;
pub use self::file::FileBasicInformation;
pub use self::file::FileDirectoryInformation;
pub use self::file::FileDispositionInformation;
pub use self::file::FileEndOfFileInformation;
pub use self::file::FileInformationClass;
pub use self::file::FileNetworkOpenInformation;
pub use self::file::FilePositionInformation;
pub use self::file::FileRenameInformation;
pub use self::file::FileSystemAttributes;
pub use self::file::FsAttributeInformation;
pub use self::file::FsDeviceInformation;
pub use self::file::FsInformationClass;
pub use self::file::FsSizeInformation;
pub use self::file::IoStatusBlock;
pub use self::file::ShareAccess;
pub use self::handle::Handle;
pub use self::hardware::FirmwareRequest;
pub use self::hardware::HardwareBootFlags;
pub use self::hardware::HardwareFlags;
pub use self::hardware::HardwareGeneration;
pub use self::hardware::HardwareInfo;
pub use self::hardware::PmwRegister;
pub use self::hardware::PrvRegister;
pub use self::hardware::SmcCommand;
pub use self::hardware::SmcI2cDevice;
pub use self::hardware::SmcMessage;
pub use self::hardware::SpecialPurposeRegister;
pub use self::loader::DllEntryReason;
pub use self::loader::ImageFlags;
pub use self::loader::ImageVersion;
pub use self::loader::LdrDataTableEntry;
pub use self::loader::LoadedCommandLine;
pub use self::loader::LoadedImageName;
pub use self::memory::AllocationType;
pub use self::memory::MemoryBasicInformation;
pub use self::memory::MemoryRegion;
pub use self::memory::MmProcessStatistics;
pub use self::memory::MmStatistics;
pub use self::memory::PageProtection;
pub use self::memory::PoolTag;
pub use self::memory::PoolType;
pub use self::network::NicStatistics;
#[cfg(any(target_os = "xbox360", xbox360_docs))]
pub use self::network::nic_get_stats;
pub use self::object::AccessMask;
pub use self::object::DispatcherObject;
pub use self::object::EVENT_INCREMENT;
pub use self::object::EventType;
pub use self::object::Kdpc;
pub use self::object::Kevent;
pub use self::object::Ksemaphore;
pub use self::object::KspinLock;
pub use self::object::Ktimer;
pub use self::object::ObjectAttributeFlags;
pub use self::object::ObjectAttributes;
pub use self::object::ObjectType;
pub use self::object::ProcessorMode;
pub use self::object::RtlCriticalSection;
pub use self::object::RtlCriticalSectionFields;
pub use self::object::TimerType;
pub use self::object::WaitReason;
pub use self::object::WaitType;
pub use self::string::AnsiString;
pub use self::string::UnicodeString;
pub use self::system::KernelVersion;
pub use self::system::TimeStampBundle;
pub use self::thread::Irql;
pub use self::thread::Kprocess;
pub use self::thread::Kthread;
pub use self::thread::NotificationRegistration;
pub use self::thread::ProcessType;
pub use self::thread::ThreadCreationFlags;
pub use self::thread::TlsIndex;
pub use self::video::DisplayDiscoveryData;
pub use self::video::DisplayInformation;
pub use self::video::DisplayModeOverrideParameters;
pub use self::video::SystemCommandBuffer;
pub use self::video::VideoModeInfo;
pub use crate::status::NtStatus;
pub use crate::status::Win32ErrorCode;

import!(
    /// Raises a breakpoint exception to the attached debugger.
    pub safe DbgBreakPoint() -> ()
);

import!(
    /// Writes a formatted string to the debug output. The arguments after
    /// the format pass in r4-r10, so they are at most seven 32-bit values.
    pub DbgPrint(format: *const u8, ...) -> u32
);

import!(
    /// Writes a crash dump for an exception record and context.
    pub DumpWriteDump(record: u32, context: u32, flags: u32) -> u32
);

import!(
    /// Allocates `size` bytes of pool memory with a four-character tag. Returns the address or 0.
    pub ExAllocatePoolWithTag(size: u32, tag: PoolTag) -> u32
);

import!(
    /// Allocates pool memory of type `pool_type` with a four-character tag. Returns the address or 0.
    pub ExAllocatePoolTypeWithTag(size: u32, tag: PoolTag, pool_type: PoolType) -> u32
);

import_variable!(
    /// Console game region. The dashboard 2.0.6717 compares the first 16
    /// bits with 0x0101 to select Japanese video mode names.
    pub safe ExConsoleGameRegion: u32,
    slot = __imp_ExConsoleGameRegion
);

import!(
    /// Creates a thread. With `xapi_thread_startup` = 0 the thread enters `start_address` with r3 = `start_context` in the console ABI (use a [`bridged!`](crate::bridged) entry).
    /// Returning from the start routine ends the thread with the returned value as its exit code, as `ExTerminateThread` does.
    /// Empty `creation_flags` start it at once. `handle_out` receives the thread handle, or the referenced thread object with [`ThreadCreationFlags::RETURN_THREAD_OBJECT`].
    pub ExCreateThread(
        handle_out: *mut u32,
        stack_size: u32,
        thread_id_out: *mut u32,
        xapi_thread_startup: u32,
        start_address: u32,
        start_context: u32,
        creation_flags: ThreadCreationFlags,
    ) -> NtStatus
);

import!(
    /// Frees pool memory.
    pub ExFreePool(address: u32) -> ()
);

import!(
    /// Reads a configuration setting into `buffer`. The setting size is stored at `required_size` when not null.
    /// The 16-bit arguments pass zero-extended in full registers.
    pub ExGetXConfigSetting(
        category: u16,
        setting: u16,
        buffer: *mut u8,
        buffer_size: u16,
        required_size: *mut u16,
    ) -> NtStatus
);

import!(
    /// Registers a thread creation and deletion callback when `register`
    /// is true and removes it otherwise. `registration` must stay valid
    /// while registered.
    pub ExRegisterThreadNotification(registration: *mut NotificationRegistration, register: bool) -> ()
);

import!(
    /// Registers a title termination callback when `register` is true and
    /// removes it otherwise. `registration` must stay valid while
    /// registered.
    pub ExRegisterTitleTerminateNotification(
        registration: *mut NotificationRegistration,
        register: bool,
    ) -> ()
);

import!(
    /// Writes a configuration setting from `buffer`. The 16-bit arguments pass zero-extended in full registers.
    pub ExSetXConfigSetting(category: u16, setting: u16, buffer: *const u8, buffer_size: u16) -> NtStatus
);

import!(
    /// Ends the current thread with `exit_code` without unwinding its stack.
    pub ExTerminateThread(exit_code: u32) -> !
);

import_variable!(
    /// Object type of thread objects.
    pub safe ExThreadObjectType: ObjectType,
    slot = __imp_ExThreadObjectType
);

import!(
    /// Returns the element count of a file system cache.
    pub FscGetCacheElementCount(cache: u32) -> u32
);

import!(
    /// Sets the element count of a file system cache.
    pub FscSetCacheElementCount(cache: u32, count: u32) -> u32
);

import!(
    /// Returns the AV pack value the hardware reports.
    pub safe HalGetCurrentAVPack() -> u32
);

import!(
    /// Opens or closes the optical drive tray.
    pub HalOpenCloseODDTray(value: u32) -> ()
);

import!(
    /// Registers a system management controller event callback when
    /// `register` is true and removes it otherwise. `registration` must stay
    /// valid while registered.
    pub HalRegisterSMCNotification(registration: *mut NotificationRegistration, register: bool) -> ()
);

import!(
    /// Restarts or shuts down the console. Stock XBDM code continues after the call, so it is not declared to diverge.
    pub HalReturnToFirmware(routine: FirmwareRequest) -> ()
);

import!(
    /// Sends a 16-byte message to the system management controller and receives the 16-byte reply.
    pub HalSendSMCMessage(message: *const SmcMessage, reply: *mut SmcMessage) -> ()
);

import!(
    /// Removes all entries from a singly linked list. Returns the first entry.
    pub InterlockedFlushSList(header: *mut u64) -> u32
);

import!(
    /// Removes the first entry from a singly linked list. Returns the entry or 0.
    pub InterlockedPopEntrySList(header: *mut u64) -> u32
);

import!(
    /// Acquires the device object lock. Returns the IRQL that `IoReleaseDeviceObjectLock` restores.
    pub IoAcquireDeviceObjectLock() -> Irql
);

import!(
    /// Allocates an I/O request packet with `stack_size` stack locations. Returns it or null.
    pub IoAllocateIrp(stack_size: u32) -> *mut Irp
);

import!(
    /// Completes an I/O request packet.
    pub IoCompleteRequest(irp: *mut Irp, priority_boost: u32) -> ()
);

import!(
    /// Creates a device object for a driver. The device object address is stored at `device`.
    pub IoCreateDevice(
        driver: *mut DriverObject,
        extension_size: u32,
        name: *const AnsiString,
        device_type: FileDeviceType,
        options: u32,
        device: *mut *mut DeviceObject,
    ) -> NtStatus
);

import_variable!(
    /// Object type of device objects.
    pub safe IoDeviceObjectType: ObjectType,
    slot = __imp_IoDeviceObjectType
);

import!(
    /// Dismounts the file system volume on a device object.
    pub IoDismountVolume(device: *mut DeviceObject) -> NtStatus
);

import!(
    /// Dismounts the file system volume that an open file belongs to.
    pub IoDismountVolumeByFileHandle(handle: Handle) -> NtStatus
);

import!(
    /// Completes an I/O request packet with an invalid device request status.
    pub IoInvalidDeviceRequest(device: *mut DeviceObject, irp: *mut Irp) -> NtStatus
);

import!(
    /// Releases the device object lock and restores `irql`.
    pub IoReleaseDeviceObjectLock(irql: Irql) -> ()
);

import!(
    /// Starts the next queued I/O request packet of a device.
    pub IoStartNextPacket(device: *mut DeviceObject) -> ()
);

import!(
    /// Starts or queues an I/O request packet on a device.
    pub IoStartPacket(device: *mut DeviceObject, irp: *mut Irp, key: *const u32) -> ()
);

import!(
    /// Sends a device I/O control request to a device object and waits for it.
    pub IoSynchronousDeviceIoControlRequest(
        code: IoControlCode,
        device: *mut DeviceObject,
        input: *const u8,
        input_length: u32,
        output: *mut u8,
        output_length: u32,
        returned: *mut u32,
    ) -> NtStatus
);

import!(
    /// Sends a read or write request to a device object and waits for it.
    pub IoSynchronousFsdRequest(
        major: IrpMajorFunction,
        device: *mut DeviceObject,
        buffer: *mut u8,
        length: u32,
        offset: *const u64,
    ) -> NtStatus
);

import!(
    /// Acquires a spin lock. The caller is already at DISPATCH_LEVEL.
    pub KeAcquireSpinLockAtRaisedIrql(lock: *mut KspinLock) -> ()
);

import!(
    /// Halts the system with a bug check code.
    pub safe KeBugCheck(code: BugCheckCode) -> !
);

import!(
    /// Halts the system with a bug check code and four parameters.
    pub safe KeBugCheckEx(
        code: BugCheckCode,
        parameter1: u32,
        parameter2: u32,
        parameter3: u32,
        parameter4: u32,
    ) -> !
);

import!(
    /// Cancels a timer. True when the timer was set.
    pub KeCancelTimer(timer: *mut Ktimer) -> Boolean8
);

import!(
    /// Builds a thread context from kernel trap frames.
    pub KeContextFromKframes(frame: *mut KtrapFrame, context: *mut Context) -> ()
);

import_variable!(
    /// Address of the [`DebugMonitorData`] block that the kernel reads, or
    /// 0. The debug monitor stores it. While a block is published, the
    /// kernel calls the routines it names, so the block must stay valid
    /// and its routine fields must hold routines of the expected form.
    pub safe KeDebugMonitorData: KernelU32,
    slot = __imp_KeDebugMonitorData
);

import!(
    /// Suspends the current thread for `interval` (100 ns units; negative is relative).
    pub KeDelayExecutionThread(
        mode: ProcessorMode,
        alertable: bool,
        interval: *const i64,
    ) -> NtStatus
);

import!(
    /// Enables the processor performance monitor.
    pub KeEnablePPUPerformanceMonitor() -> u32
);

import!(
    /// Disables normal kernel APC delivery to the current thread.
    pub KeEnterCriticalRegion() -> ()
);

import!(
    /// Flushes the data cache for an address range.
    pub KeFlushCacheRange(address: u32, length: u32) -> ()
);

import!(
    /// Returns the process type of the current thread.
    pub safe KeGetCurrentProcessType() -> ProcessType
);

import!(
    /// Reads a performance monitor register.
    pub KeGetPMWRegister(register: PmwRegister) -> u64
);

import!(
    /// Reads a special purpose register. Only the low word of the result is returned.
    pub KeGetSpecialPurposeRegister(register: SpecialPurposeRegister) -> u32
);

import!(
    /// Initializes a deferred procedure call object.
    pub KeInitializeDpc(dpc: *mut Kdpc, routine: usize, context: usize) -> ()
);

import!(
    /// Initializes an event object.
    pub KeInitializeEvent(event: *mut Kevent, kind: EventType, initial_state: bool) -> ()
);

import!(
    /// Initializes a semaphore object with a count and limit.
    pub KeInitializeSemaphore(semaphore: *mut Ksemaphore, count: u32, limit: u32) -> ()
);

import!(
    /// Initializes a timer object for processes of `process_type`. Stock XBDM passes [`ProcessType::SYSTEM`] for its capture timer.
    pub KeInitializeTimerEx(timer: *mut Ktimer, kind: TimerType, process_type: ProcessType) -> ()
);

import!(
    /// Queues a deferred procedure call. True when it was queued.
    pub KeInsertQueueDpc(dpc: *mut Kdpc, argument1: u32, argument2: u32) -> Boolean8
);

import!(
    /// Runs `routine(context)` on every processor at IPI level.
    pub KeIpiGenericCall(routine: u32, context: u32) -> u32
);

import!(
    /// Enables normal kernel APC delivery to the current thread.
    pub KeLeaveCriticalRegion() -> ()
);

import!(
    /// Returns the timebase frequency.
    pub safe KeQueryPerformanceFrequency() -> u64
);

import!(
    /// Stores the system time as a FILETIME.
    pub KeQuerySystemTime(time: *mut u64) -> ()
);

import!(
    /// Raises the IRQL to DISPATCH_LEVEL. Returns the previous IRQL.
    pub KeRaiseIrqlToDpcLevel() -> Irql
);

import!(
    /// Releases a semaphore object. Returns the previous count.
    pub KeReleaseSemaphore(semaphore: *mut Ksemaphore, increment: u32, adjustment: u32, wait: bool) -> u32
);

import!(
    /// Releases a spin lock without changing the IRQL.
    pub KeReleaseSpinLockFromRaisedIrql(lock: *mut KspinLock) -> ()
);

import!(
    /// Removes a deferred procedure call from the queue. True when it was queued.
    pub KeRemoveQueueDpc(dpc: *mut Kdpc) -> Boolean8
);

import!(
    /// Sets an event object to not signaled. Returns the previous state.
    pub KeResetEvent(event: *mut Kevent) -> u32
);

import!(
    /// Decrements the suspend count of a thread object. Returns the previous count.
    pub KeResumeThread(thread: *mut Kthread) -> u32
);

import!(
    /// Sets the processor affinity of a thread. The previous affinity is stored at `previous`.
    pub KeSetAffinityThread(thread: *mut Kthread, affinity: u32, previous: *mut u32) -> NtStatus
);

import!(
    /// Sets the base priority increment of a thread object.
    pub KeSetBasePriorityThread(thread: *mut Kthread, increment: u32) -> u32
);

import!(
    /// Sets an event object to signaled. Returns the previous state. `increment` is usually [`EVENT_INCREMENT`].
    pub KeSetEvent(event: *mut Kevent, increment: u32, wait: bool) -> u32
);

import!(
    /// Writes a performance monitor register.
    pub KeSetPMWRegister(register: PmwRegister, value: u64) -> ()
);

import!(
    /// Writes a performance monitor register.
    pub KeSetPRVRegister(register: PrvRegister, value: u64) -> ()
);

import!(
    /// Sets the priority of a thread object. Returns the previous priority.
    pub KeSetPriorityThread(thread: *mut Kthread, priority: u32) -> u32
);

import!(
    /// Writes a special purpose register. The export takes the 64-bit `value` in one register.
    pub KeSetSpecialPurposeRegister(register: SpecialPurposeRegister, value: u64) -> u32
);

import!(
    /// Sets a timer to expire at `due_time` (100 ns units; negative is relative to now), then every `period` milliseconds when `period` is not 0.
    /// `dpc` is queued at expiry when not null. True when the timer was already set.
    pub KeSetTimerEx(timer: *mut Ktimer, due_time: i64, period: u32, dpc: *mut Kdpc) -> Boolean8
);

import!(
    /// Busy-waits for `microseconds`.
    pub safe KeStallExecutionProcessor(microseconds: u32) -> ()
);

import!(
    /// Increments the suspend count of a thread object. Returns the previous count.
    pub KeSuspendThread(thread: *mut Kthread) -> u32
);

import!(
    /// Invalidates the instruction cache for an address range.
    pub KeSweepIcacheRange(address: *mut u8, length: u32) -> ()
);

import_variable!(
    /// Kernel time stamp bundle.
    pub safe KeTimeStampBundle: TimeStampBundle,
    slot = __imp_KeTimeStampBundle
);

import!(
    /// Tries to acquire a spin lock at DISPATCH_LEVEL. True when acquired.
    pub KeTryToAcquireSpinLockAtRaisedIrql(lock: *mut KspinLock) -> Boolean8
);

import!(
    /// Waits on several dispatcher objects. `wait_blocks` may be null for up to four objects.
    pub KeWaitForMultipleObjects(
        count: u32,
        objects: *const *mut DispatcherObject,
        wait_type: WaitType,
        reason: WaitReason,
        mode: ProcessorMode,
        alertable: bool,
        timeout: *const i64,
        wait_blocks: *mut u32,
    ) -> NtStatus
);

import!(
    /// Waits on a dispatcher object. A null `timeout` waits without limit.
    pub KeWaitForSingleObject(
        object: *mut DispatcherObject,
        reason: WaitReason,
        mode: ProcessorMode,
        alertable: bool,
        timeout: *const i64,
    ) -> NtStatus
);

import!(
    /// Raises the IRQL to DISPATCH_LEVEL and acquires a spin lock. Returns the previous IRQL.
    pub KfAcquireSpinLock(lock: *mut KspinLock) -> Irql
);

import!(
    /// Raises the IRQL. Returns the previous IRQL.
    pub KfRaiseIrql(irql: Irql) -> Irql
);

import!(
    /// Lowers the IRQL.
    pub KfLowerIrql(irql: Irql) -> ()
);

import!(
    /// Releases a spin lock and restores `irql`.
    pub KfReleaseSpinLock(lock: *mut KspinLock, irql: Irql) -> ()
);

import!(
    /// Allocates physical memory. Returns the address or 0.
    pub MmAllocatePhysicalMemory(kind: u32, size: u32, protect: PageProtection) -> u32
);

import!(
    /// Allocates physical memory within an address range and alignment. Returns the address or 0.
    pub MmAllocatePhysicalMemoryEx(
        region: MemoryRegion,
        size: u32,
        protect: PageProtection,
        lowest: u32,
        highest: u32,
        alignment: u32,
    ) -> u32
);

import!(
    /// Frees physical memory that [`MmAllocatePhysicalMemoryEx`] allocated
    /// in `region`.
    pub MmFreePhysicalMemory(region: MemoryRegion, address: u32) -> ()
);

import!(
    /// Translates a mapped virtual address to its physical address.
    pub MmGetPhysicalAddress(address: u32) -> u32
);

import!(
    /// True when an address is mapped.
    pub safe MmIsAddressValid(address: u32) -> Boolean8
);

import!(
    /// Returns the page protection of an address.
    pub safe MmQueryAddressProtect(address: u32) -> PageProtection
);

import!(
    /// Returns the size of the allocation that contains an address.
    pub MmQueryAllocationSize(address: u32) -> u32
);

import!(
    /// Fills `statistics`, whose length field the caller sets.
    pub MmQueryStatistics(statistics: *mut MmStatistics) -> NtStatus
);

import!(
    /// Reserves or commits virtual memory. Base and size are updated in place.
    pub NtAllocateVirtualMemory(
        base: *mut u32,
        size: *mut u32,
        allocation_type: AllocationType,
        protect: PageProtection,
        region: MemoryRegion,
    ) -> NtStatus
);

import!(
    /// Sets an event to not signaled.
    pub NtClearEvent(handle: Handle) -> NtStatus
);

import!(
    /// Closes a handle.
    pub NtClose(handle: Handle) -> NtStatus
);

import!(
    /// Creates an object directory.
    pub NtCreateDirectoryObject(
        handle: *mut Handle,
        attributes: *const ObjectAttributes,
    ) -> NtStatus
);

import!(
    /// Creates an event.
    pub NtCreateEvent(
        handle: *mut Handle,
        attributes: *const ObjectAttributes,
        kind: EventType,
        initial_state: bool,
    ) -> NtStatus
);

import!(
    /// Creates or opens a file.
    pub NtCreateFile(
        handle: *mut Handle,
        access: AccessMask,
        attributes: *const ObjectAttributes,
        io_status: *mut IoStatusBlock,
        allocation_size: *const u64,
        file_attributes: FileAttributes,
        share: ShareAccess,
        disposition: CreateDisposition,
        options: CreateOptions,
    ) -> NtStatus
);

import!(
    /// Creates a semaphore with a count and limit.
    pub NtCreateSemaphore(
        handle: *mut Handle,
        attributes: *const ObjectAttributes,
        count: u32,
        limit: u32,
    ) -> NtStatus
);

import!(
    /// Deletes a file.
    pub NtDeleteFile(object_attributes: *const ObjectAttributes) -> NtStatus
);

import!(
    /// Sends a device I/O control request through a file handle.
    pub NtDeviceIoControlFile(
        handle: Handle,
        event: Handle,
        apc_routine: u32,
        apc_context: u32,
        io_status: *mut IoStatusBlock,
        control_code: IoControlCode,
        input: *const u8,
        input_length: u32,
        output: *mut u8,
        output_length: u32,
    ) -> NtStatus
);

import!(
    /// Writes buffered data of a file to the device.
    pub NtFlushBuffersFile(handle: Handle, io_status: *mut IoStatusBlock) -> NtStatus
);

import!(
    /// Decommits or releases virtual memory. Base and size are updated in place.
    pub NtFreeVirtualMemory(
        base: *mut u32,
        size: *mut u32,
        free_type: AllocationType,
        options: u32,
    ) -> NtStatus
);

import!(
    /// Opens an object directory.
    pub NtOpenDirectoryObject(handle: *mut Handle, attributes: *const ObjectAttributes) -> NtStatus
);

import!(
    /// Opens a file.
    pub NtOpenFile(
        handle: *mut Handle,
        access: AccessMask,
        attributes: *const ObjectAttributes,
        io_status: *mut IoStatusBlock,
        share: ShareAccess,
        options: CreateOptions,
    ) -> NtStatus
);

import!(
    /// Reads directory entries. `file_name` is an optional wildcard pattern.
    pub NtQueryDirectoryFile(
        handle: Handle,
        event: Handle,
        apc_routine: u32,
        apc_context: u32,
        io_status: *mut IoStatusBlock,
        information: *mut u8,
        length: u32,
        file_name: *const AnsiString,
        restart: bool,
    ) -> NtStatus
);

import!(
    /// Reads one entry of an object directory. `context` holds the enumeration position.
    pub NtQueryDirectoryObject(
        handle: Handle,
        buffer: *mut u8,
        length: u32,
        restart: bool,
        context: *mut u32,
        return_length: *mut u32,
    ) -> NtStatus
);

import!(
    /// Reads the network open information of a file by name.
    pub NtQueryFullAttributesFile(
        attributes: *const ObjectAttributes,
        information: *mut u8,
    ) -> NtStatus
);

import!(
    /// Reads information of class `class` about an open file.
    pub NtQueryInformationFile(
        handle: Handle,
        io_status: *mut IoStatusBlock,
        information: *mut u8,
        length: u32,
        class: FileInformationClass,
    ) -> NtStatus
);

import!(
    /// Fills a memory basic information structure for an address.
    pub NtQueryVirtualMemory(address: u32, information: *mut u32, process_type: u32) -> NtStatus
);

import!(
    /// Reads information of class `class` about a volume.
    pub NtQueryVolumeInformationFile(
        handle: Handle,
        io_status: *mut IoStatusBlock,
        information: *mut u8,
        length: u32,
        class: FsInformationClass,
    ) -> NtStatus
);

import!(
    /// Reads from a file. A null `byte_offset` reads at the current position. A null `event` signals the file handle.
    pub NtReadFile(
        handle: Handle,
        event: Handle,
        apc_routine: u32,
        apc_context: u32,
        io_status: *mut IoStatusBlock,
        buffer: *mut u8,
        length: u32,
        byte_offset: *const u64,
    ) -> NtStatus
);

import!(
    /// Reads from a file into a list of page-sized segments.
    pub NtReadFileScatter(
        handle: Handle,
        event: Handle,
        apc: u32,
        context: u32,
        io_status: *mut IoStatusBlock,
        segments: *const u32,
        length: u32,
        offset: *const u64,
    ) -> NtStatus
);

import!(
    /// Releases a semaphore. The previous count is stored at `previous` when not null.
    pub NtReleaseSemaphore(handle: Handle, count: u32, previous: *mut u32) -> NtStatus
);

import!(
    /// Sets an event to signaled. The previous state is stored at `previous_state` when not null.
    pub NtSetEvent(handle: Handle, previous_state: *mut u32) -> NtStatus
);

import!(
    /// Writes information of class `class` about an open file.
    pub NtSetInformationFile(
        handle: Handle,
        io_status: *mut IoStatusBlock,
        information: *const u8,
        length: u32,
        class: FileInformationClass,
    ) -> NtStatus
);

import!(
    /// Sets the system time as a FILETIME. The previous time is stored at `previous_time` when not null.
    pub NtSetSystemTime(new_time: *const u64, previous_time: *mut u64) -> NtStatus
);

import!(
    /// Waits on a handle. A null `timeout` waits without limit.
    pub NtWaitForSingleObjectEx(
        handle: Handle,
        mode: ProcessorMode,
        alertable: bool,
        timeout: *const i64,
    ) -> NtStatus
);

import!(
    /// Waits on several handles.
    pub NtWaitForMultipleObjectsEx(
        count: u32,
        handles: *const Handle,
        wait_type: WaitType,
        mode: ProcessorMode,
        alertable: bool,
        timeout: *const i64,
    ) -> NtStatus
);

import!(
    /// Writes to a file. A null `byte_offset` writes at the current position. A null `event` signals the file handle.
    pub NtWriteFile(
        handle: Handle,
        event: Handle,
        apc_routine: u32,
        apc_context: u32,
        io_status: *mut IoStatusBlock,
        buffer: *const u8,
        length: u32,
        byte_offset: *const u64,
    ) -> NtStatus
);

import!(
    /// Writes to a file from a list of page-sized segments.
    pub NtWriteFileGather(
        handle: Handle,
        event: Handle,
        apc: u32,
        context: u32,
        io_status: *mut IoStatusBlock,
        segments: *const u32,
        length: u32,
        offset: *const u64,
    ) -> NtStatus
);

import!(
    /// Yields the remainder of the time slice of the current thread.
    pub safe NtYieldExecution() -> NtStatus
);

import!(
    /// Allocates an object of type `object_type` with a body of `body_size` bytes.
    pub ObCreateObject(
        object_type: u32,
        attributes: *const ObjectAttributes,
        body_size: u32,
        object: *mut u32,
    ) -> NtStatus
);

import!(
    /// Creates a symbolic link object from `link` to `target`.
    pub ObCreateSymbolicLink(link: *const AnsiString, target: *const AnsiString) -> NtStatus
);

import!(
    /// Deletes the symbolic link object `link`.
    pub ObDeleteSymbolicLink(link: *const AnsiString) -> NtStatus
);

import!(
    /// Releases a reference to an object.
    pub ObDereferenceObject(object: u32) -> ()
);

import!(
    /// Inserts an object into the handle table and object directory. The handle is stored at `handle`.
    pub ObInsertObject(
        object: u32,
        attributes: *const ObjectAttributes,
        pointer_bias: u32,
        handle: *mut Handle,
    ) -> NtStatus
);

import!(
    /// Finds a thread object by thread ID and references it.
    pub ObLookupAnyThreadByThreadId(thread_id: u32, thread: *mut *mut Kthread) -> NtStatus
);

import!(
    /// Removes the permanent flag of a named object.
    pub ObMakeTemporaryObject(object: u32) -> ()
);

import!(
    /// Opens a named object and stores a handle.
    pub ObOpenObjectByName(
        attributes: *const ObjectAttributes,
        object_type: u32,
        parse_context: u32,
        handle: *mut Handle,
    ) -> NtStatus
);

import!(
    /// References the object of a handle. `object_type` 0 accepts any type.
    pub ObReferenceObjectByHandle(handle: Handle, object_type: u32, object: *mut u32) -> NtStatus
);

import!(
    /// References a named object.
    pub ObReferenceObjectByName(
        name: *const AnsiString,
        attributes: ObjectAttributeFlags,
        object_type: u32,
        parse_context: u32,
        object: *mut u32,
    ) -> NtStatus
);

import!(
    /// Reads the target of a symbolic link object into `target`.
    pub ObTranslateSymbolicLink(
        attributes: *const ObjectAttributes,
        target: *mut AnsiString,
    ) -> NtStatus
);

import!(
    /// Reports a failed assertion to the debugger.
    pub RtlAssert(message: *const u8, file: *const u8, line: u32, extra: *const u8) -> ()
);

import!(
    /// Stores the context of the current thread.
    pub RtlCaptureContext(context: *mut u8) -> ()
);

import!(
    /// Enters a critical section.
    pub RtlEnterCriticalSection(section: *mut RtlCriticalSection) -> ()
);

import!(
    /// Stores the low and high stack limits of the current thread.
    pub RtlGetStackLimits(low: *mut u32, high: *mut u32) -> ()
);

import!(
    /// Returns the value or address of an optional header field of a XEX header.
    pub RtlImageXexHeaderField(header: u32, key: XexHeaderKey) -> u32
);

import!(
    /// Initializes a critical section.
    pub RtlInitializeCriticalSection(section: *mut RtlCriticalSection) -> ()
);

import!(
    /// Initializes a critical section with a spin count.
    pub RtlInitializeCriticalSectionAndSpinCount(section: *mut RtlCriticalSection, spin_count: u32) -> NtStatus
);

import!(
    /// Leaves a critical section.
    pub RtlLeaveCriticalSection(section: *mut RtlCriticalSection) -> ()
);

import!(
    /// Converts an NTSTATUS value to a Win32 error code.
    pub safe RtlNtStatusToDosError(status: NtStatus) -> Win32ErrorCode
);

import!(
    /// Tries to enter a critical section. True when entered.
    pub RtlTryEnterCriticalSection(section: *mut RtlCriticalSection) -> Boolean8
);

import!(
    /// Formats into `buffer` from a va_list. `arguments` is a console va_list: 8-byte slots, 32-bit values in the low word.
    pub _vsnprintf(buffer: *mut u8, count: u32, format: u32, arguments: u32) -> i32
);

import!(
    /// Allocates a thread local storage index. Returns [`TlsIndex::OUT_OF_INDEXES`] when none is free.
    pub safe KeTlsAlloc() -> TlsIndex
);

import!(
    /// Frees a thread local storage index. True on success.
    pub KeTlsFree(index: TlsIndex) -> Boolean
);

import!(
    /// Returns the value of a thread local storage index for the current thread.
    pub safe KeTlsGetValue(index: TlsIndex) -> u32
);

import!(
    /// Sets the value of a thread local storage index for the current thread. True on success.
    pub KeTlsSetValue(index: TlsIndex, value: u32) -> Boolean
);

import_variable!(
    /// Console hardware information.
    pub safe XboxHardwareInfo: HardwareInfo,
    slot = __imp_XboxHardwareInfo
);

import_variable!(
    /// Version of the base kernel.
    pub safe XboxKrnlBaseVersion: KernelVersion,
    slot = __imp_XboxKrnlBaseVersion
);

import_variable!(
    /// Version of the running kernel.
    pub safe XboxKrnlVersion: KernelVersion,
    slot = __imp_XboxKrnlVersion
);

import!(
    /// Sets the parity bit of each byte of a DES key of `size` bytes.
    pub XeCryptDesParity(input: *const u8, size: u32, output: *mut u8) -> ()
);

import!(
    /// Expands an 8-byte DES key into `state`.
    pub XeCryptDesKey(state: *mut DesState, key: *const u8) -> ()
);

import!(
    /// Encrypts (`encrypt` nonzero) or decrypts one 8-byte block in ECB mode.
    pub XeCryptDesEcb(state: *const DesState, input: *const u8, output: *mut u8, encrypt: Boolean) -> ()
);

import!(
    /// HMAC-SHA1 of the concatenation of up to three buffers under `key`; the first `digest_size` bytes of
    /// the digest go to `digest`. The dashboard passes null buffers of size 0 for the unused ones.
    pub XeCryptHmacSha(
        key: *const u8,
        key_size: u32,
        data1: *const u8,
        size1: u32,
        data2: *const u8,
        size2: u32,
        data3: *const u8,
        size3: u32,
        digest: *mut u8,
        digest_size: u32,
    ) -> ()
);

import!(
    /// MD5 of the concatenation of up to three buffers; the first `digest_size` bytes of the
    /// digest go to `digest`. The dashboard passes null buffers of size 0 for the unused ones.
    pub XeCryptMd5(
        data1: *const u8,
        size1: u32,
        data2: *const u8,
        size2: u32,
        data3: *const u8,
        size3: u32,
        digest: *mut u8,
        digest_size: u32,
    ) -> ()
);

import!(
    /// Fills a buffer with random bytes.
    pub XeCryptRandom(buffer: *mut u8, length: u32) -> ()
);

import_variable!(
    /// Module handle of the running executable: the address of its
    /// [`LdrDataTableEntry`].
    pub safe XexExecutableModuleHandle: KernelU32,
    slot = __imp_XexExecutableModuleHandle
);

import!(
    /// Finds a loaded module by name and stores its handle.
    pub XexGetModuleHandle(name: *const u8, handle: *mut u32) -> NtStatus
);

import!(
    /// Finds the resource `name` of a module and stores its address and size. `name` must equal the stored name, which is at most 8 bytes, byte for byte, so case is significant. `0xFFFFFFFF` as `module` names the caller's module. Fails with `STATUS_NOT_FOUND` when the module has no such resource.
    pub XexGetModuleSection(
        module: u32,
        name: *const u8,
        address: *mut u32,
        size: *mut u32,
    ) -> NtStatus
);

import!(
    /// Finds an export of a module by ordinal and stores its address. `ordinal` may instead be the address of a NUL-terminated export name.
    pub XexGetProcedureAddress(module: u32, ordinal: u32, address: *mut u32) -> NtStatus
);

import!(
    /// Loads a XEX image and stores its module handle. `version` is the oldest version accepted.
    pub XexLoadImage(
        name: *const u8,
        flags: ImageFlags,
        version: ImageVersion,
        handle: *mut u32,
    ) -> NtStatus
);

import!(
    /// Finds the loader entry of the module that contains an address. Returns the image base.
    pub XexPcToFileHeader(address: u32, entry: *mut u32) -> u32
);

import!(
    /// Unloads a XEX image.
    pub XexUnloadImage(handle: u32) -> NtStatus
);

import!(
    /// Returns the address of a PE data directory of an image. Its size is stored at `size`.
    pub RtlImageDirectoryEntryToData(
        base: u32,
        mapped: bool,
        directory: ImageDirectory,
        size: *mut u32,
    ) -> u32
);

import!(
    /// Returns the PE NT headers of an image, or null.
    pub RtlImageNtHeader(base: u32) -> *const ImageNtHeaders
);

import!(
    /// Calls a debug monitor service of the kernel.
    pub ExDebugMonitorService(service: DebugMonitorService, a: u32, b: u32, c: u32) -> NtStatus
);

import!(
    /// Returns the address when the debugger may read it, else 0.
    pub MmDbgReadCheck(address: u32) -> u32
);

import!(
    /// Releases a mapping that `MmDbgWriteCheck` created.
    pub MmDbgReleaseAddress(address: u32, context: *mut u32) -> ()
);

import!(
    /// Maps `address` writable for the debugger. Returns the mapped address, else 0.
    pub MmDbgWriteCheck(address: u32, context: *mut u32) -> u32
);

import_variable!(
    /// Command line of the running executable.
    pub safe ExLoadedCommandLine: LoadedCommandLine,
    slot = __imp_ExLoadedCommandLine
);

import_variable!(
    /// Object path of the running executable.
    pub safe ExLoadedImageName: LoadedImageName,
    slot = __imp_ExLoadedImageName
);

import!(
    /// Makes the command processor write its ring read pointer, in dwords, to `physical` (dword
    /// aligned). `block_log2` is the read pointer block size field of the ring control register.
    pub VdEnableRingBufferRPtrWriteBack(physical: u32, block_log2: u32) -> ()
);

import!(
    /// Fills a display information structure.
    pub VdGetCurrentDisplayInformation(information: *mut DisplayInformation) -> ()
);

import!(
    /// Copies the next of the kernel's three system command buffer descriptors to `buffer` and stores in
    /// `identifier` the value the GPU writes to the identifier address once that buffer has run.
    pub VdGetSystemCommandBuffer(buffer: *mut SystemCommandBuffer, identifier: *mut u32) -> ()
);

import!(
    /// Gives the GPU command processor to the calling title: loads the CP microcode (the kernel's own
    /// when both pointers are null), stores `shutdown_callback`/`context` for [`VdShutdownEngines`] and
    /// resets the engines. `unused` is not read. Returns 1. Traps when the engines are already owned.
    pub VdInitializeEngines(
        unused: u32,
        shutdown_callback: u32,
        context: u32,
        prefetch_microcode: *const u32,
        engine_microcode: *const u32,
    ) -> u32
);

import!(
    /// Points the command processor ring at `physical` with 2^(`size_log2` + 1) dwords and resets its
    /// read and write pointers.
    pub VdInitializeRingBuffer(physical: u32, size_log2: u32) -> ()
);

import!(
    /// Reads a video encoder register.
    pub VdReadDVERegisterUlong(register: u32) -> u32
);

import!(
    /// Programs the configured video mode. `flags` selects display-off,
    /// forced-output, color-space, and pixel-format options.
    pub VdSetDisplayMode(flags: u32) -> ()
);

import!(
    /// Overrides the display mode; nonzero on success. `scan` is 1 for
    /// interlaced and 2 for progressive [INFERENCE]; `parameters` may be
    /// null. All zero arguments remove the override.
    pub VdSetDisplayModeOverride(
        width: u32,
        height: u32,
        refresh_rate: f64,
        scan: u32,
        parameters: *const DisplayModeOverrideParameters,
    ) -> u32
);

import!(
    /// Sets the routine the graphics interrupt handler calls with (0, `context`), or none when 0.
    pub VdSetGraphicsInterruptCallback(callback: u32, context: u32) -> ()
);

import!(
    /// Sets the virtual address of the dword where the GPU writes the identifier of each finished
    /// system command buffer, or none when 0.
    pub VdSetSystemCommandBufferGpuIdentifierAddress(address: u32) -> ()
);

import!(
    /// Waits for the GPU to idle, returns the command processor to the kernel's own ring and calls the
    /// shutdown callback of [`VdInitializeEngines`].
    pub VdShutdownEngines() -> ()
);

import!(
    /// Writes a video encoder register.
    pub VdWriteDVERegisterUlong(register: u32, value: u32) -> u32
);

import!(
    /// Turns the controller radio on or off.
    pub XInputdSetRFPowerMode(enable: u32) -> u32
);

import!(
    /// Registers an audio render client. The client handle is stored at `handle`.
    pub XAudioRegisterRenderDriverClient(client: *const u32, handle: *mut u32) -> u32
);

import!(
    /// Removes an audio render client.
    pub XAudioUnregisterRenderDriverClient(handle: u32) -> u32
);

import!(
    /// Submits a frame of samples for an audio render client.
    pub XAudioSubmitRenderDriverFrame(client: u32, samples: u32) -> u32
);

import!(
    /// Returns the audio render driver thread object.
    pub XAudioGetRenderDriverThread() -> *mut Kthread
);

import!(
    /// Sets the speaker configuration (the XConfig audio flags value).
    pub XAudioSetSpeakerConfig(config: u32, reserved: u32) -> u32
);

import!(
    /// Fills a network interface statistics structure.
    pub NicGetStats(device_flags: u32, stats: *mut u8) -> u32
);

import!(
    /// State of the USB camera driver. The dashboard reads 2 as ready.
    pub safe XUsbcamGetState() -> u32
);
import!(
    /// Creates the USB camera driver context in `workspace` and stores its
    /// camera handle. Returns zero on success.
    pub XUsbcamCreate(workspace: *mut u8, workspace_size: u32, camera: *mut u32) -> u32
);

import!(
    /// Destroys a USB camera driver context.
    pub XUsbcamDestroy(camera: u32) -> ()
);

import!(
    /// Allocates one hardware XMA decoder context.
    pub XMACreateContext(context: *mut *mut XmaContextData) -> u32
);

import!(
    /// Initializes an allocated XMA decoder context.
    pub XMAInitializeContext(context: *mut XmaContextData, init: *const XmaContextInit) -> ()
);

import!(
    /// Releases a hardware XMA decoder context.
    pub XMAReleaseContext(context: *mut XmaContextData) -> ()
);

import!(
    /// Enables a hardware XMA decoder context.
    pub XMAEnableContext(context: *mut XmaContextData) -> ()
);

import!(
    /// Disables a hardware XMA decoder context. `wait` waits for in-flight
    /// decoder work.
    pub XMADisableContext(context: *mut XmaContextData, wait: u32) -> u32
);

import!(
    /// Returns the decoder's output ring write offset in 256-byte blocks.
    pub XMAGetOutputBufferWriteOffset(context: *mut XmaContextData) -> u32
);

import!(
    /// Sets the output ring read offset in 256-byte blocks.
    pub XMASetOutputBufferReadOffset(context: *mut XmaContextData, offset: u32) -> ()
);

import!(
    /// Returns the output ring read offset in 256-byte blocks.
    pub XMAGetOutputBufferReadOffset(context: *mut XmaContextData) -> u32
);

import!(
    /// Marks the output ring of an XMA decoder context valid.
    pub XMASetOutputBufferValid(context: *mut XmaContextData) -> ()
);

import!(
    /// Returns whether the output ring is valid.
    pub XMAIsOutputBufferValid(context: *mut XmaContextData) -> u32
);

import!(
    /// Marks input buffer zero of an XMA decoder context valid.
    pub XMASetInputBuffer0Valid(context: *mut XmaContextData) -> ()
);

import!(
    /// Returns whether input buffer zero is valid.
    pub XMAIsInputBuffer0Valid(context: *mut XmaContextData) -> u32
);

import!(
    /// Waits until the decoder is not working on `context`.
    pub XMABlockWhileInUse(context: *mut XmaContextData) -> ()
);

import!(
    /// Writes one USB camera configuration value.
    pub XUsbcamSetConfig(
        camera: u32,
        request: u32,
        selector: u32,
        value: *const u8,
        length: u32,
        callback: usize,
        context: usize,
    ) -> i32
);
import!(
    /// Writes a boot loader image to flash. A nonzero `flags` value on return also indicates failure.
    pub XeKeysSaveBootLoader(flags: *mut u32, size: *mut u32, buffer: u32) -> NtStatus
);

import!(
    /// Stores the security key status flags.
    pub XeKeysGetStatus(status: *mut u32) -> ()
);

import!(
    /// Copies the key `key` into `buffer`; `size` holds the buffer size on entry and the key size on return.
    /// Key 0x14 is the 12-digit console serial number.
    pub XeKeysGetKey(key: u16, buffer: *mut u8, size: *mut u32) -> NtStatus
);

import!(
    /// Stores the 5-byte console ID and, when `text` is not null, its 12-character text form.
    pub XeKeysGetConsoleID(raw: *mut u8, text: *mut u8) -> NtStatus
);

import!(
    /// Writes a 64-dword front buffer flip into `commands`: when `system_buffer` holds a buffer, fetch
    /// constant 15 from the 6-dword `fetch_constant` (base address virtual) and an indirect buffer call
    /// of it, then an event write of `identifier` to `identifier_address` (virtual), padded with a NOP.
    /// The five outputs are written only when `system_buffer` carries optional state.
    pub VdSwap(
        commands: *mut u32,
        fetch_constant: *const u32,
        identifier_address: u32,
        system_buffer: *const SystemCommandBuffer,
        identifier: u32,
        out0: *mut u32,
        out1: *mut u32,
        out2: *mut u32,
        out3: *mut u32,
        out4: *mut u32,
    ) -> ()
);

import!(
    /// True while the power-down deferred procedure call runs.
    pub safe HalIsExecutingPowerDownDpc() -> Boolean8
);

import!(
    /// A step of EDRAM retraining. Returns 0 when nothing is due:
    /// `VdHSIOTrainCount` (export 0x26B) is nonzero and `flags` has bit 1
    /// clear. In state 0 it sets `*state` to 1, returns 1 and leaves
    /// `*commands` alone, so a call with a null `*commands` asks whether
    /// retraining is due. D3D then calls it with `*commands` at free ring
    /// space until it returns 0, and takes the advanced `*commands` as the
    /// end of what was written. Returns 0 without effect when `*state` is
    /// above 7, `capacity` (dwords) below 0x928 or `work_bytes` below 0x5C.
    /// A null `state` traps.
    pub VdRetrainEDRAM(flags: u32, commands: *mut *mut u32, capacity: u32, state: *mut u32, work: *mut u8, work_bytes: u32) -> u32
);

import!(
    /// When `VdHSIOTrainCount` is 0: sends message 7 to the system
    /// management controller and stores byte 6 of the reply as the current
    /// EDRAM temperature of the retraining state. Returns 0.
    pub VdRetrainEDRAMWorker(reserved: u32) -> u32
);

import!(
    /// Calls `callback` with each display mode the attached display
    /// supports; the callback returns nonzero to continue.
    pub VdEnumerateVideoModes(callback: u32, context: u32) -> ()
);

import!(
    /// Fills `data` with the display discovery result; nonzero on success.
    pub VdGetDisplayDiscoveryData(data: *mut DisplayDiscoveryData) -> u32
);

import!(
    /// Replaces a 4-byte configuration setting with `(old & and_mask) | or_mask`.
    /// The 16-bit arguments pass zero-extended in full registers.
    pub ExReadModifyWriteXConfigSettingUlong(category: u16, setting: u16, and_mask: u32, or_mask: u32) -> NtStatus
);
