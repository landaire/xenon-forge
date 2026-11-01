/*
 * Kernel executive (Ke): dispatcher objects (semaphores, timers, DPCs),
 * spin locks, interrupt levels, waits, threads and processor registers.
 */

#ifndef XBOX360_XBOXKRNL_KE_H
#define XBOX360_XBOXKRNL_KE_H

#include <xbox360/base.h>
#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* Interrupt request levels. */
#define PASSIVE_LEVEL  0
#define APC_LEVEL      1
#define DISPATCH_LEVEL 2

/* KeSetEvent priority increment for the waiters an event releases. */
#define EVENT_INCREMENT 1

/* Bug check codes; codes not named here are module-specific. */
#define KMODE_EXCEPTION_NOT_HANDLED 0x0000001E

/* KeTlsAlloc result when no index is free. */
#define TLS_OUT_OF_INDEXES 0xFFFFFFFF

/* A set of hardware threads; bit n is hardware thread n. */
typedef ULONG KAFFINITY, *PKAFFINITY;

/* Spin lock: one word, 0 when free. */
typedef ULONG_PTR KSPIN_LOCK, *PKSPIN_LOCK;

/* Process type of a thread, as KeGetCurrentProcessType reports it. */
typedef enum _KPROCESS_TYPE {
    PROCESS_TYPE_IDLE = 0,
    /* The title process. */
    PROCESS_TYPE_TITLE = 1,
    PROCESS_TYPE_SYSTEM = 2,
} KPROCESS_TYPE;

typedef enum _TIMER_TYPE {
    /* Stays signaled until set again and releases every waiter. */
    NotificationTimer = 0,
    /* Releases one waiter and resets itself. */
    SynchronizationTimer = 1,
} TIMER_TYPE;

/* How a wait on several objects ends. */
typedef enum _WAIT_TYPE {
    /* When every object is signaled. */
    WaitAll = 0,
    /* When any object is signaled; the status is STATUS_WAIT_0 plus its index. */
    WaitAny = 1,
} WAIT_TYPE;

/* Why a thread waits. */
typedef enum _KWAIT_REASON {
    Executive = 0,
    DelayExecution = 1,
    Suspended = 2,
    UserRequest = 3,
    WrQueue = 4,
    WrFsCacheIn = 5,
    WrFsCacheOut = 6,
    WrExecutive = 7,
    WrCriticalSection = 8,
} KWAIT_REASON;

typedef struct _KSEMAPHORE {
    DISPATCHER_HEADER Header;
    LONG Limit;
} KSEMAPHORE, *PKSEMAPHORE;

C_ASSERT(sizeof(KSEMAPHORE) == 0x14);

struct _KDPC;

/* Routine of a deferred procedure call. */
typedef VOID (*PKDEFERRED_ROUTINE)(
    struct _KDPC *Dpc,
    PVOID DeferredContext,
    PVOID SystemArgument1,
    PVOID SystemArgument2);

/*
 * Deferred procedure call. InsertedNumber is the hardware thread whose
 * queue holds the DPC plus one, 0 when not queued. TargetNumber is the
 * hardware thread that runs it plus one, 0 for the one that queues it;
 * KeInitializeDpc clears it, so set it afterwards.
 */
typedef struct _KDPC {
    SHORT Type;
    UCHAR InsertedNumber;
    UCHAR TargetNumber;
    LIST_ENTRY DpcListEntry;
    PKDEFERRED_ROUTINE DeferredRoutine;
    PVOID DeferredContext;
    PVOID SystemArgument1;
    PVOID SystemArgument2;
} KDPC, *PKDPC;

C_ASSERT(__builtin_offsetof(KDPC, TargetNumber) == 3);
C_ASSERT(__builtin_offsetof(KDPC, DeferredRoutine) == 0x0C);
C_ASSERT(sizeof(KDPC) == 0x1C);

typedef struct _KTIMER {
    DISPATCHER_HEADER Header;
    ULARGE_INTEGER DueTime;
    LIST_ENTRY TimerListEntry;
    PKDPC Dpc;
    LONG Period;
} KTIMER, *PKTIMER;

C_ASSERT(__builtin_offsetof(KTIMER, DueTime) == 0x10);
C_ASSERT(__builtin_offsetof(KTIMER, TimerListEntry) == 0x18);
C_ASSERT(__builtin_offsetof(KTIMER, Dpc) == 0x20);
C_ASSERT(sizeof(KTIMER) == 0x28);

/* Links a waiting thread to one object of its wait. */
typedef struct _KWAIT_BLOCK {
    LIST_ENTRY WaitListEntry;
    struct _KTHREAD *Thread;
    PVOID Object;
    struct _KWAIT_BLOCK *NextWaitBlock;
    USHORT WaitKey;
    USHORT WaitType;
} KWAIT_BLOCK, *PKWAIT_BLOCK;

C_ASSERT(__builtin_offsetof(KWAIT_BLOCK, WaitKey) == 0x14);
C_ASSERT(sizeof(KWAIT_BLOCK) == 0x18);

/* Start routine of a thread. */
typedef VOID (*PKSTART_ROUTINE)(PVOID StartContext);

/* Routine that a new thread runs first; it calls StartRoutine(StartContext). */
typedef VOID (*PKSYSTEM_ROUTINE)(PKSTART_ROUTINE StartRoutine, PVOID StartContext);

/* Routine that KeIpiGenericCall runs on every processor. */
typedef ULONG_PTR (*PKIPI_BROADCAST_WORKER)(ULONG_PTR Argument);

/*
 * Registers the kernel saves on an interrupt or exception, up to the last
 * field in use.
 */
typedef struct _KTRAP_FRAME {
    ULONG StackFrameHeader[0x14];
    ULONGLONG Gpr[32];
    /* Exception record or interrupt state. */
    ULONG Reserved1[0x14];
    ULONGLONG Ctr;
    ULONG Lr;
    ULONG Iar;
    ULONG Cr;
    /* Low word of the saved MSR. */
    ULONG Msr;
    ULONG Xer;
    ULONG UserModeState;
} KTRAP_FRAME, *PKTRAP_FRAME;

C_ASSERT(__builtin_offsetof(KTRAP_FRAME, Gpr) == 0x50);
C_ASSERT(__builtin_offsetof(KTRAP_FRAME, Lr) == 0x1A8);
C_ASSERT(__builtin_offsetof(KTRAP_FRAME, Iar) == 0x1AC);
C_ASSERT(__builtin_offsetof(KTRAP_FRAME, Msr) == 0x1B4);
C_ASSERT(__builtin_offsetof(KTRAP_FRAME, Xer) == 0x1B8);
C_ASSERT(sizeof(KTRAP_FRAME) == 0x1C0);

/*
 * Thread object, up to the last field in use. The kernel allocates it and
 * frees it after the thread ends.
 */
typedef struct _KTHREAD {
    /* SignalState is nonzero once the thread has ended. */
    DISPATCHER_HEADER Header;
    UCHAR Reserved1[0x48];
    /* 1 ms clock ticks that the thread ran. */
    ULONG RunTime;
    /* Highest address of the stack. */
    PVOID StackBase;
    /* Lowest address of the stack. */
    PVOID StackLimit;
    /* Frame the thread saved when it last stopped running. */
    PVOID SwitchFrame;
    /* Thread local storage block. */
    PVOID TlsData;
    UCHAR Reserved2[4];
    UCHAR Priority;
    UCHAR Reserved3;
    /* KPROCESS_TYPE of the process that owns the thread. */
    UCHAR ProcessType;
    /* KPROCESS_TYPE that KeGetCurrentProcessType reports for the thread. */
    UCHAR CurrentProcessType;
    UCHAR Reserved4[0x10];
    /* Process that owns the thread. */
    struct _KPROCESS *Process;
    UCHAR Reserved5[0x30];
    UCHAR PriorityDecrement;
    UCHAR BasePriority;
    UCHAR Reserved6;
    UCHAR Saturation;
    UCHAR SuspendCount;
    UCHAR Reserved7[2];
    /* Hardware thread the thread runs or last ran on. */
    UCHAR CurrentProcessor;
    UCHAR Reserved8[9];
    /* Priority that KeSetBasePriorityThread adds its increment to. */
    UCHAR ProcessPriority;
    /* Highest base priority that KeSetBasePriorityThread sets. */
    UCHAR PriorityLimit;
    UCHAR Reserved9[0x45];
    /* Link in the thread list of the process. */
    LIST_ENTRY ThreadListEntry;
    UCHAR Reserved10[0x18];
    /* Creation time as a FILETIME. */
    ULARGE_INTEGER CreateTime;
    UCHAR Reserved11[8];
    NTSTATUS ExitStatus;
    UCHAR Reserved12[8];
    ULONG UniqueThread;
    PVOID StartAddress;
    UCHAR Reserved13[8];
    /* Word for the debug monitor's own use. */
    ULONG DebugMonitorData;
    /* Win32 last error. */
    ULONG LastWin32ErrorCode;
    UCHAR Reserved14[8];
    /* ExCreateThread creation flags. */
    ULONG CreateOptions;
} KTHREAD, *PKTHREAD;

C_ASSERT(__builtin_offsetof(KTHREAD, RunTime) == 0x58);
C_ASSERT(__builtin_offsetof(KTHREAD, TlsData) == 0x68);
C_ASSERT(__builtin_offsetof(KTHREAD, Priority) == 0x70);
C_ASSERT(__builtin_offsetof(KTHREAD, CurrentProcessType) == 0x73);
C_ASSERT(__builtin_offsetof(KTHREAD, Process) == 0x84);
C_ASSERT(__builtin_offsetof(KTHREAD, PriorityDecrement) == 0xB8);
C_ASSERT(__builtin_offsetof(KTHREAD, SuspendCount) == 0xBC);
C_ASSERT(__builtin_offsetof(KTHREAD, CurrentProcessor) == 0xBF);
C_ASSERT(__builtin_offsetof(KTHREAD, ProcessPriority) == 0xC9);
C_ASSERT(__builtin_offsetof(KTHREAD, PriorityLimit) == 0xCA);
C_ASSERT(__builtin_offsetof(KTHREAD, ThreadListEntry) == 0x110);
C_ASSERT(__builtin_offsetof(KTHREAD, CreateTime) == 0x130);
C_ASSERT(__builtin_offsetof(KTHREAD, ExitStatus) == 0x140);
C_ASSERT(__builtin_offsetof(KTHREAD, UniqueThread) == 0x14C);
C_ASSERT(__builtin_offsetof(KTHREAD, StartAddress) == 0x150);
C_ASSERT(__builtin_offsetof(KTHREAD, DebugMonitorData) == 0x15C);
C_ASSERT(__builtin_offsetof(KTHREAD, LastWin32ErrorCode) == 0x160);
C_ASSERT(__builtin_offsetof(KTHREAD, CreateOptions) == 0x16C);
C_ASSERT(sizeof(KTHREAD) == 0x170);

/* Process object, up to the thread list. */
typedef struct _KPROCESS {
    /* Guards ThreadListHead. */
    KSPIN_LOCK ThreadListLock;
    /* Head of the list of KTHREAD.ThreadListEntry links. */
    LIST_ENTRY ThreadListHead;
} KPROCESS, *PKPROCESS;

C_ASSERT(__builtin_offsetof(KPROCESS, ThreadListHead) == 4);
C_ASSERT(sizeof(KPROCESS) == 12);

/* Times that the kernel updates on each clock tick. */
typedef struct _KTIME_STAMP_BUNDLE {
    ULONGLONG InterruptTime;
    ULONGLONG SystemTime;
    /* Milliseconds since boot. */
    ULONG TickCount;
    ULONG Reserved;
} KTIME_STAMP_BUNDLE, *PKTIME_STAMP_BUNDLE;

C_ASSERT(__builtin_offsetof(KTIME_STAMP_BUNDLE, TickCount) == 0x10);
C_ASSERT(sizeof(KTIME_STAMP_BUNDLE) == 0x18);

/* Special purpose registers of KeGetSpecialPurposeRegister and KeSetSpecialPurposeRegister. */
/* Data address breakpoint register. */
#define SPR_DABR          0x3F5
/* Performance monitor control: bit 1 freezes and bit 2 enables the counters. */
#define SPR_MONITOR_CONTROL 0x3F6
/* Performance monitor event selection. */
#define SPR_EVENT_SELECT  0x3FA

/* Performance monitor bus registers of KeGetPMWRegister and KeSetPMWRegister. */
#define PMW_NCU0     0x480
#define PMW_NCU1     0x580
#define PMW_NCU2     0x680
#define PMW_CIU      0x708
#define PMW_BIU      0xB50
#define PMW_L2_GROUP 0x310
#define PMW_L2       0x318
#define PMW_L2_ECC   0x860

/* Performance monitor registers of KeSetPRVRegister; values are in the high word. */
#define PRV_RUN              0x1C0
#define PRV_INTERVAL_CONTROL 0x1C8
#define PRV_TRIGGER          0x1D8
#define PRV_INTERVAL         0x1E0
#define PRV_INTERVAL_RELOAD  0x1E8
/* Value and control registers of counter Counter (0 to 15). */
#define PRV_COUNTER_VALUE(Counter)   (0x200 + (Counter) * 8)
#define PRV_COUNTER_CONTROL(Counter) (0x280 + (Counter) * 8)

/** Acquires a spin lock. The caller is already at DISPATCH_LEVEL. */
XBOXKRNL_IMPORT(0x04D) VOID KeAcquireSpinLockAtRaisedIrql(PKSPIN_LOCK SpinLock);

/** Halts the system with a bug check code. */
XBOXKRNL_IMPORT(0x052) DECLSPEC_NORETURN VOID KeBugCheck(ULONG BugCheckCode);

/** Halts the system with a bug check code and four parameters. */
XBOXKRNL_IMPORT(0x053) DECLSPEC_NORETURN VOID KeBugCheckEx(
    ULONG BugCheckCode,
    ULONG_PTR BugCheckParameter1,
    ULONG_PTR BugCheckParameter2,
    ULONG_PTR BugCheckParameter3,
    ULONG_PTR BugCheckParameter4);

/** Cancels a timer. True when the timer was set. */
XBOXKRNL_IMPORT(0x054) BOOLEAN KeCancelTimer(PKTIMER Timer);

/* CONTEXT is defined in rtl.h; DEBUG_MONITOR_DATA belongs to the debug monitor service. */
struct _CONTEXT;
struct _DEBUG_MONITOR_DATA;

/** Builds a thread context from kernel trap frames. */
XBOXKRNL_IMPORT(0x056) VOID KeContextFromKframes(PKTRAP_FRAME TrapFrame, struct _CONTEXT *ContextRecord);

/**
 * Address of the debug monitor data block that the kernel reads, or 0.
 * The debug monitor stores it. While a block is published, the kernel
 * calls the routines it names, so the block must stay valid and its
 * routine fields must hold routines of the expected form.
 */
XBOXKRNL_IMPORT(0x059) extern struct _DEBUG_MONITOR_DATA **__imp_KeDebugMonitorData;
#define KeDebugMonitorData (*__imp_KeDebugMonitorData)

/** Suspends the current thread for Interval (100 ns units; negative is relative). */
XBOXKRNL_IMPORT(0x05A) NTSTATUS KeDelayExecutionThread(
    KPROCESSOR_MODE WaitMode,
    BOOLEAN Alertable,
    PLARGE_INTEGER Interval);

/** Enables the processor performance monitor. */
XBOXKRNL_IMPORT(0x05E) ULONG KeEnablePPUPerformanceMonitor(VOID);

/** Disables normal kernel APC delivery to the current thread. */
XBOXKRNL_IMPORT(0x05F) VOID KeEnterCriticalRegion(VOID);

/** Flushes the data cache for an address range. */
XBOXKRNL_IMPORT(0x061) VOID KeFlushCacheRange(PVOID BaseAddress, ULONG Length);

/** Returns the process type of the current thread. */
XBOXKRNL_IMPORT(0x066) KPROCESS_TYPE KeGetCurrentProcessType(VOID);

/** Reads a performance monitor register. */
XBOXKRNL_IMPORT(0x067) ULONGLONG KeGetPMWRegister(ULONG Register);

/** Reads a special purpose register. Only the low word of the result is returned. */
XBOXKRNL_IMPORT(0x06A) ULONG KeGetSpecialPurposeRegister(ULONG Register);

/** Initializes a deferred procedure call object. */
XBOXKRNL_IMPORT(0x06F) VOID KeInitializeDpc(
    PKDPC Dpc,
    PKDEFERRED_ROUTINE DeferredRoutine,
    PVOID DeferredContext);

/** Initializes an event object. */
XBOXKRNL_IMPORT(0x070) VOID KeInitializeEvent(PKEVENT Event, EVENT_TYPE Type, BOOLEAN State);

/** Initializes a semaphore object with a count and limit. */
XBOXKRNL_IMPORT(0x074) VOID KeInitializeSemaphore(PKSEMAPHORE Semaphore, LONG Count, LONG Limit);

/**
 * Initializes a timer object for processes of ProcessType. Stock XBDM
 * passes PROCESS_TYPE_SYSTEM for its capture timer.
 */
XBOXKRNL_IMPORT(0x075) VOID KeInitializeTimerEx(PKTIMER Timer, TIMER_TYPE Type, UCHAR ProcessType);

/** Queues a deferred procedure call. True when it was queued. */
XBOXKRNL_IMPORT(0x07B) BOOLEAN KeInsertQueueDpc(
    PKDPC Dpc,
    PVOID SystemArgument1,
    PVOID SystemArgument2);

/** Runs BroadcastFunction(Context) on every processor at IPI level. */
XBOXKRNL_IMPORT(0x07C) ULONG_PTR KeIpiGenericCall(
    PKIPI_BROADCAST_WORKER BroadcastFunction,
    ULONG_PTR Context);

/** Enables normal kernel APC delivery to the current thread. */
XBOXKRNL_IMPORT(0x07D) VOID KeLeaveCriticalRegion(VOID);

/** Returns the timebase frequency. */
XBOXKRNL_IMPORT(0x083) ULONGLONG KeQueryPerformanceFrequency(VOID);

/** Stores the system time as a FILETIME. */
XBOXKRNL_IMPORT(0x084) VOID KeQuerySystemTime(PLARGE_INTEGER CurrentTime);

/** Raises the IRQL to DISPATCH_LEVEL. Returns the previous IRQL. */
XBOXKRNL_IMPORT(0x085) KIRQL KeRaiseIrqlToDpcLevel(VOID);

/** Releases a semaphore object. Returns the previous count. */
XBOXKRNL_IMPORT(0x088) LONG KeReleaseSemaphore(
    PKSEMAPHORE Semaphore,
    KPRIORITY Increment,
    LONG Adjustment,
    BOOLEAN Wait);

/** Releases a spin lock without changing the IRQL. */
XBOXKRNL_IMPORT(0x089) VOID KeReleaseSpinLockFromRaisedIrql(PKSPIN_LOCK SpinLock);

/** Removes a deferred procedure call from the queue. True when it was queued. */
XBOXKRNL_IMPORT(0x08E) BOOLEAN KeRemoveQueueDpc(PKDPC Dpc);

/** Sets an event object to not signaled. Returns the previous state. */
XBOXKRNL_IMPORT(0x08F) LONG KeResetEvent(PKEVENT Event);

/** Decrements the suspend count of a thread object. Returns the previous count. */
XBOXKRNL_IMPORT(0x092) ULONG KeResumeThread(PKTHREAD Thread);

/** Sets the processor affinity of a thread. The previous affinity is stored at PreviousAffinity. */
XBOXKRNL_IMPORT(0x097) NTSTATUS KeSetAffinityThread(
    PKTHREAD Thread,
    KAFFINITY Affinity,
    PKAFFINITY PreviousAffinity);

/** Sets the base priority increment of a thread object. */
XBOXKRNL_IMPORT(0x099) LONG KeSetBasePriorityThread(PKTHREAD Thread, LONG Increment);

/**
 * Sets an event object to signaled. Returns the previous state. Increment
 * is usually EVENT_INCREMENT.
 */
XBOXKRNL_IMPORT(0x09D) LONG KeSetEvent(PKEVENT Event, KPRIORITY Increment, BOOLEAN Wait);

/** Writes a performance monitor register. The export takes the 64-bit Value in one register. */
XBOXKRNL_IMPORT(0x09F) VOID KeSetPMWRegister(ULONG Register, ULONGLONG Value);

/** Writes a performance monitor register. The export takes the 64-bit Value in one register. */
XBOXKRNL_IMPORT(0x0A1) VOID KeSetPRVRegister(ULONG Register, ULONGLONG Value);

/** Sets the priority of a thread object. Returns the previous priority. */
XBOXKRNL_IMPORT(0x0A3) KPRIORITY KeSetPriorityThread(PKTHREAD Thread, KPRIORITY Priority);

/** Writes a special purpose register. The export takes the 64-bit Value in one register. */
XBOXKRNL_IMPORT(0x0A5) ULONG KeSetSpecialPurposeRegister(ULONG Register, ULONGLONG Value);

/**
 * Sets a timer to expire at DueTime (100 ns units; negative is relative to
 * now), then every Period milliseconds when Period is not 0. Dpc is queued
 * at expiry when not NULL. True when the timer was already set.
 * DueTime is a LARGE_INTEGER passed by value, in one register.
 */
XBOXKRNL_IMPORT(0x0A7) BOOLEAN KeSetTimerEx(PKTIMER Timer, LONGLONG DueTime, LONG Period, PKDPC Dpc);

/** Busy-waits for MicroSeconds. */
XBOXKRNL_IMPORT(0x0A8) VOID KeStallExecutionProcessor(ULONG MicroSeconds);

/** Increments the suspend count of a thread object. Returns the previous count. */
XBOXKRNL_IMPORT(0x0A9) ULONG KeSuspendThread(PKTHREAD Thread);

/** Invalidates the instruction cache for an address range. */
XBOXKRNL_IMPORT(0x0AB) VOID KeSweepIcacheRange(PVOID BaseAddress, SIZE_T Length);

/** Kernel time stamp bundle. */
XBOXKRNL_IMPORT(0x0AD) extern PKTIME_STAMP_BUNDLE __imp_KeTimeStampBundle;
#define KeTimeStampBundle (*__imp_KeTimeStampBundle)

/** Tries to acquire a spin lock at DISPATCH_LEVEL. True when acquired. */
XBOXKRNL_IMPORT(0x0AE) BOOLEAN KeTryToAcquireSpinLockAtRaisedIrql(PKSPIN_LOCK SpinLock);

/**
 * Waits on several dispatcher objects. WaitBlockArray may be NULL for up
 * to four objects.
 */
XBOXKRNL_IMPORT(0x0AF) NTSTATUS KeWaitForMultipleObjects(
    ULONG Count,
    PVOID *Object,
    WAIT_TYPE WaitType,
    KWAIT_REASON WaitReason,
    KPROCESSOR_MODE WaitMode,
    BOOLEAN Alertable,
    PLARGE_INTEGER Timeout,
    PKWAIT_BLOCK WaitBlockArray);

/** Waits on a dispatcher object. A NULL Timeout waits without limit. */
XBOXKRNL_IMPORT(0x0B0) NTSTATUS KeWaitForSingleObject(
    PVOID Object,
    KWAIT_REASON WaitReason,
    KPROCESSOR_MODE WaitMode,
    BOOLEAN Alertable,
    PLARGE_INTEGER Timeout);

/** Raises the IRQL to DISPATCH_LEVEL and acquires a spin lock. Returns the previous IRQL. */
XBOXKRNL_IMPORT(0x0B1) KIRQL KfAcquireSpinLock(PKSPIN_LOCK SpinLock);

/** Raises the IRQL. Returns the previous IRQL. */
XBOXKRNL_IMPORT(0x0B2) KIRQL KfRaiseIrql(KIRQL NewIrql);

/** Lowers the IRQL. */
XBOXKRNL_IMPORT(0x0B3) VOID KfLowerIrql(KIRQL NewIrql);

/** Releases a spin lock and restores OldIrql. */
XBOXKRNL_IMPORT(0x0B4) VOID KfReleaseSpinLock(PKSPIN_LOCK SpinLock, KIRQL OldIrql);

/** Allocates a thread local storage index. Returns TLS_OUT_OF_INDEXES when none is free. */
XBOXKRNL_IMPORT(0x152) ULONG KeTlsAlloc(VOID);

/** Frees a thread local storage index. True on success. */
XBOXKRNL_IMPORT(0x153) BOOLEAN KeTlsFree(ULONG Index);

/** Returns the value of a thread local storage index for the current thread. */
XBOXKRNL_IMPORT(0x154) PVOID KeTlsGetValue(ULONG Index);

/** Sets the value of a thread local storage index for the current thread. True on success. */
XBOXKRNL_IMPORT(0x155) BOOLEAN KeTlsSetValue(ULONG Index, PVOID Value);

XBOX360_EXTERN_C_END

#endif
