/*
 * Executive: pool memory, threads, notifications, configuration settings,
 * events, semaphores, waits and the loaded image name.
 */

#ifndef XBOX360_XBOXKRNL_EX_H
#define XBOX360_XBOXKRNL_EX_H

#include <xbox360/xboxkrnl/ntdef.h>
#include <xbox360/xboxkrnl/ke.h>

XBOX360_EXTERN_C_START

/*
 * Pool of ExAllocatePoolTypeWithTag. CurrentProcessPool selects the title
 * pool for a title thread and the system pool otherwise.
 */
typedef enum _POOL_TYPE {
    CurrentProcessPool = 0,
    TitlePool = 1,
    SystemPool = 2,
    TitleDebugPool = 3,
    SystemDebugPool = 4,
} POOL_TYPE;

/*
 * CreationFlags of ExCreateThread. Bits 24 to 29 select the hardware
 * threads the new thread may run on.
 */
#define EX_CREATE_FLAG_SUSPENDED           0x00000001
#define EX_CREATE_FLAG_SYSTEM              0x00000002
/* Allocate the TLS data block with the thread and copy the TLS template into it. */
#define EX_CREATE_FLAG_TLS_DATA            0x00000008
/* Priority class 13; takes precedence over EX_CREATE_FLAG_PRIORITY_CLASS_LOW. */
#define EX_CREATE_FLAG_PRIORITY_CLASS_HIGH 0x00000020
/* Priority class 5. */
#define EX_CREATE_FLAG_PRIORITY_CLASS_LOW  0x00000040
/* ThreadHandle receives a referenced thread object instead of a handle. */
#define EX_CREATE_FLAG_RETURN_KTHREAD      0x00000080
/* Entry thread of a title. */
#define EX_CREATE_FLAG_TITLE_ENTRY         0x00000100
/* A debug monitor does not stop the thread. */
#define EX_CREATE_FLAG_EXEMPT_FROM_STOPS   0x00000200
/* A debug monitor does not debug the thread. */
#define EX_CREATE_FLAG_NOT_DEBUGGABLE      0x00000400
#define EX_CREATE_FLAG_PROCESSOR0          0x01000000
#define EX_CREATE_FLAG_PROCESSOR1          0x02000000
#define EX_CREATE_FLAG_PROCESSOR2          0x04000000
#define EX_CREATE_FLAG_PROCESSOR3          0x08000000
#define EX_CREATE_FLAG_PROCESSOR4          0x10000000
#define EX_CREATE_FLAG_PROCESSOR5          0x20000000
#define EX_CREATE_AFFINITY_SHIFT           24

/*
 * Thread notification. The routine gets Creating 1 when the thread starts
 * and 0 when it ends. Routines of higher Priority run first. The kernel
 * owns ListEntry while the record is registered.
 */
struct _EX_THREAD_REGISTRATION;

typedef VOID (*PEX_THREAD_NOTIFICATION_ROUTINE)(
    struct _EX_THREAD_REGISTRATION *Registration,
    PKTHREAD Thread,
    BOOLEAN Creating
);

typedef struct _EX_THREAD_REGISTRATION {
    PEX_THREAD_NOTIFICATION_ROUTINE NotificationRoutine;
    LONG Priority;
    LIST_ENTRY ListEntry;
} EX_THREAD_REGISTRATION, *PEX_THREAD_REGISTRATION;

C_ASSERT(sizeof(EX_THREAD_REGISTRATION) == 16);
C_ASSERT(__builtin_offsetof(EX_THREAD_REGISTRATION, ListEntry) == 8);

/* Title termination notification, laid out as EX_THREAD_REGISTRATION. */
struct _EX_TITLE_TERMINATE_REGISTRATION;

typedef VOID (*PEX_TITLE_TERMINATE_ROUTINE)(struct _EX_TITLE_TERMINATE_REGISTRATION *Registration);

typedef struct _EX_TITLE_TERMINATE_REGISTRATION {
    PEX_TITLE_TERMINATE_ROUTINE NotificationRoutine;
    LONG Priority;
    LIST_ENTRY ListEntry;
} EX_TITLE_TERMINATE_REGISTRATION, *PEX_TITLE_TERMINATE_REGISTRATION;

C_ASSERT(sizeof(EX_TITLE_TERMINATE_REGISTRATION) == 16);
C_ASSERT(__builtin_offsetof(EX_TITLE_TERMINATE_REGISTRATION, ListEntry) == 8);

/*
 * Categories of the configuration settings. Setting 0 of a category is the
 * whole category record.
 */
#define XCONFIG_STATIC_CATEGORY               0
#define XCONFIG_STATISTIC_CATEGORY            1
#define XCONFIG_SECURED_CATEGORY              2
#define XCONFIG_USER_CATEGORY                 3
#define XCONFIG_XNET_MACHINE_ACCOUNT_CATEGORY 4
#define XCONFIG_XNET_PARAMETERS_CATEGORY      5
#define XCONFIG_MEDIA_CENTER_CATEGORY         6
#define XCONFIG_CONSOLE_CATEGORY              7
#define XCONFIG_DVD_CATEGORY                  8
#define XCONFIG_IPTV_CATEGORY                 9
#define XCONFIG_SYSTEM_CATEGORY               10

/* Settings of XCONFIG_SECURED_CATEGORY. */
#define XCONFIG_SECURED_GAME_REGION  3
#define XCONFIG_SECURED_SYSTEM_FLAGS 6

/* Settings of XCONFIG_USER_CATEGORY. */
#define XCONFIG_USER_RETAIL_FLAGS 12

/* Services of ExDebugMonitorService. */
#define ExDmRegisterDebugMonitor      0
#define ExDmGetDebugMonitorData       1
#define ExDmEnableProfiling           2
#define ExDmGetHvStatistics           3
#define ExDmGetCdRomEmulationRoutines 4
#define ExDmGetCdRomSenseData         5
#define ExDmGetPreDebugRoutine        6
#define ExDmSetDvdEmulationMode       7
#define ExDmQueryDevkitStatistics     8

/* Sizes of ExLoadedImageName and ExLoadedCommandLine. */
#define EX_LOADED_IMAGE_NAME_SIZE   256
#define EX_LOADED_COMMAND_LINE_SIZE 512

struct _EXCEPTION_RECORD;
struct _CONTEXT;

/** Writes a crash dump for an exception record and context. */
XBOXKRNL_IMPORT(0x006) NTSTATUS DumpWriteDump(
    struct _EXCEPTION_RECORD *ExceptionRecord,
    struct _CONTEXT *ContextRecord,
    ULONG Flags
);

/**
 * Allocates NumberOfBytes of pool memory with a four-character tag.
 * Returns the address or NULL.
 */
XBOXKRNL_IMPORT(0x00A) PVOID ExAllocatePoolWithTag(SIZE_T NumberOfBytes, ULONG Tag);

/**
 * Allocates pool memory of type PoolType with a four-character tag.
 * Returns the address or NULL.
 */
XBOXKRNL_IMPORT(0x00B) PVOID ExAllocatePoolTypeWithTag(SIZE_T NumberOfBytes, ULONG Tag, POOL_TYPE PoolType);

/**
 * Console game region. The dashboard 2.0.6717 compares the first 16 bits
 * with 0x0101 to select Japanese video mode names.
 */
XBOXKRNL_IMPORT(0x00C) extern PUSHORT __imp_ExConsoleGameRegion;
#define ExConsoleGameRegion (*__imp_ExConsoleGameRegion)

/**
 * Creates a thread. With SystemRoutine NULL the thread enters StartRoutine
 * with StartContext. Returning from the start routine ends the thread with
 * the returned value as its exit code, as ExTerminateThread does. Zero
 * CreationFlags start it at once. ThreadHandle receives the thread handle,
 * or the referenced thread object with EX_CREATE_FLAG_RETURN_KTHREAD.
 */
XBOXKRNL_IMPORT(0x00D) NTSTATUS ExCreateThread(
    PHANDLE ThreadHandle,
    SIZE_T StackSize,
    PULONG ThreadId,
    PKSYSTEM_ROUTINE SystemRoutine,
    PKSTART_ROUTINE StartRoutine,
    PVOID StartContext,
    ULONG CreationFlags
);

/** Frees pool memory. */
XBOXKRNL_IMPORT(0x00F) VOID ExFreePool(PVOID P);

/**
 * Reads a configuration setting into Buffer. The setting size is stored at
 * ResultSize when not null.
 */
XBOXKRNL_IMPORT(0x010) NTSTATUS ExGetXConfigSetting(
    USHORT Category,
    USHORT Setting,
    PVOID Buffer,
    USHORT BufferSize,
    PUSHORT ResultSize
);

/**
 * Registers a thread creation and deletion callback when Register is true
 * and removes it otherwise. Registration must stay valid while registered.
 */
XBOXKRNL_IMPORT(0x014) VOID ExRegisterThreadNotification(PEX_THREAD_REGISTRATION Registration, BOOLEAN Register);

/**
 * Registers a title termination callback when Register is true and removes
 * it otherwise. Registration must stay valid while registered.
 */
XBOXKRNL_IMPORT(0x015) VOID ExRegisterTitleTerminateNotification(
    PEX_TITLE_TERMINATE_REGISTRATION Registration,
    BOOLEAN Register
);

/** Writes a configuration setting from Buffer. */
XBOXKRNL_IMPORT(0x018) NTSTATUS ExSetXConfigSetting(USHORT Category, USHORT Setting, PVOID Buffer, USHORT BufferSize);

/** Ends the current thread with ExitStatus without unwinding its stack. */
XBOXKRNL_IMPORT(0x019) DECLSPEC_NORETURN VOID ExTerminateThread(NTSTATUS ExitStatus);

/** Object type of thread objects. */
XBOXKRNL_IMPORT(0x01B) extern struct _OBJECT_TYPE *__imp_ExThreadObjectType;
#define ExThreadObjectType (*__imp_ExThreadObjectType)

/** Calls a debug monitor service (ExDm*) of the kernel. */
XBOXKRNL_IMPORT(0x1AA) NTSTATUS ExDebugMonitorService(ULONG Service, PVOID Argument1, ULONG Argument2, PVOID Argument3);

/** Command line of the running executable, NUL-terminated. */
XBOXKRNL_IMPORT(0x1AE) extern CHAR (*__imp_ExLoadedCommandLine)[EX_LOADED_COMMAND_LINE_SIZE];
#define ExLoadedCommandLine (*__imp_ExLoadedCommandLine)

/** Object path of the running executable, NUL-terminated. */
XBOXKRNL_IMPORT(0x1AF) extern CHAR (*__imp_ExLoadedImageName)[EX_LOADED_IMAGE_NAME_SIZE];
#define ExLoadedImageName (*__imp_ExLoadedImageName)

/** Replaces a 4-byte configuration setting with (old & AndMask) | OrMask. */
XBOXKRNL_IMPORT(0x2BC) NTSTATUS ExReadModifyWriteXConfigSettingUlong(
    USHORT Category,
    USHORT Setting,
    ULONG AndMask,
    ULONG OrMask
);

/** Sets an event to not signaled. */
XBOXKRNL_IMPORT(0x0CE) NTSTATUS NtClearEvent(HANDLE EventHandle);

/** Closes a handle. */
XBOXKRNL_IMPORT(0x0CF) NTSTATUS NtClose(HANDLE Handle);

/** Creates an event. */
XBOXKRNL_IMPORT(0x0D1) NTSTATUS NtCreateEvent(
    PHANDLE EventHandle,
    POBJECT_ATTRIBUTES ObjectAttributes,
    EVENT_TYPE EventType,
    BOOLEAN InitialState
);

/** Creates a semaphore with a count and limit. */
XBOXKRNL_IMPORT(0x0D5) NTSTATUS NtCreateSemaphore(
    PHANDLE SemaphoreHandle,
    POBJECT_ATTRIBUTES ObjectAttributes,
    LONG InitialCount,
    LONG MaximumCount
);

/** Releases a semaphore. The previous count is stored at PreviousCount when not null. */
XBOXKRNL_IMPORT(0x0F3) NTSTATUS NtReleaseSemaphore(HANDLE SemaphoreHandle, LONG ReleaseCount, PLONG PreviousCount);

/** Sets an event to signaled. The previous state is stored at PreviousState when not null. */
XBOXKRNL_IMPORT(0x0F6) NTSTATUS NtSetEvent(HANDLE EventHandle, PLONG PreviousState);

/** Sets the system time as a FILETIME. The previous time is stored at PreviousTime when not null. */
XBOXKRNL_IMPORT(0x0F9) NTSTATUS NtSetSystemTime(PLARGE_INTEGER SystemTime, PLARGE_INTEGER PreviousTime);

/** Waits on a handle. A null Timeout waits without limit. */
XBOXKRNL_IMPORT(0x0FD) NTSTATUS NtWaitForSingleObjectEx(
    HANDLE Handle,
    KPROCESSOR_MODE WaitMode,
    BOOLEAN Alertable,
    PLARGE_INTEGER Timeout
);

/** Waits on several handles. */
XBOXKRNL_IMPORT(0x0FE) NTSTATUS NtWaitForMultipleObjectsEx(
    ULONG Count,
    const HANDLE *Handles,
    WAIT_TYPE WaitType,
    KPROCESSOR_MODE WaitMode,
    BOOLEAN Alertable,
    PLARGE_INTEGER Timeout
);

/** Yields the remainder of the time slice of the current thread. */
XBOXKRNL_IMPORT(0x101) NTSTATUS NtYieldExecution(VOID);

XBOX360_EXTERN_C_END

#endif
