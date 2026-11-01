/*
 * XAM loader, system, Win32-style, automation, media and play timer imports.
 */

#ifndef XBOX360_XAM_SYSTEM_H
#define XBOX360_XAM_SYSTEM_H

#include <xbox360/xboxkrnl/ntdef.h>
#include <xbox360/xam/xamdef.h>

XBOX360_EXTERN_C_START

/* Wait time without limit. */
#define INFINITE 0xFFFFFFFF

/* Results of WaitForSingleObject. */
#define WAIT_OBJECT_0    0x00000000
#define WAIT_ABANDONED_0 0x00000080
#define WAIT_TIMEOUT     0x00000102
#define WAIT_FAILED      0xFFFFFFFF

/* Flags of RtlAllocateHeap and RtlFreeHeap. */
#define HEAP_ZERO_MEMORY 0x00000008

/* Options of DuplicateHandle. */
#define DUPLICATE_CLOSE_SOURCE 0x00000001
#define DUPLICATE_SAME_ACCESS  0x00000002

/* CreateFileA result of a failed call. */
#define INVALID_HANDLE_VALUE ((HANDLE)(LONG_PTR)-1)

/* Creation disposition of CreateFileA. */
typedef enum _CREATION_DISPOSITION {
    CREATE_NEW = 1,
    CREATE_ALWAYS = 2,
    OPEN_EXISTING = 3,
    OPEN_ALWAYS = 4,
    TRUNCATE_EXISTING = 5
} CREATION_DISPOSITION;

/* Date and time, as GetLocalTime stores it. */
typedef struct _SYSTEMTIME {
    WORD wYear;
    WORD wMonth;
    WORD wDayOfWeek;
    WORD wDay;
    WORD wHour;
    WORD wMinute;
    WORD wSecond;
    WORD wMilliseconds;
} SYSTEMTIME, *PSYSTEMTIME;

C_ASSERT(sizeof(SYSTEMTIME) == 0x10);

/* Video mode, filled by XGetVideoMode. Layout from Xenia. */
typedef struct _XVIDEO_MODE {
    DWORD dwDisplayWidth;
    DWORD dwDisplayHeight;
    /* Nonzero for an interlaced mode. */
    BOOL fIsInterlaced;
    /* Nonzero for a 16:9 display. */
    BOOL fIsWideScreen;
    /* Nonzero for a high definition mode. */
    BOOL fIsHiDef;
    FLOAT RefreshRate;
    DWORD VideoStandard;
    DWORD Reserved[5];
} XVIDEO_MODE, *PXVIDEO_MODE;

C_ASSERT(sizeof(XVIDEO_MODE) == 0x30);
C_ASSERT(__builtin_offsetof(XVIDEO_MODE, RefreshRate) == 0x14);

/* Record of XamPlayTimerGetData and XamPlayTimerSetData. */
typedef struct _XPLAY_TIMER_DATA {
    /* Next reset, written by XamPlayTimerGetNextResetDate. */
    DWORD NextReset[2];
    /* Period flags: 0 is off, 2 daily, 4 weekly. */
    DWORD Period;
    /* Play time allowed per period, in milliseconds. */
    DWORD Allowance;
    /* Play time remaining in the period, in milliseconds. */
    DWORD Remaining;
} XPLAY_TIMER_DATA, *PXPLAY_TIMER_DATA;

C_ASSERT(sizeof(XPLAY_TIMER_DATA) == 0x14);
C_ASSERT(__builtin_offsetof(XPLAY_TIMER_DATA, Remaining) == 0x10);

/** Applies the configured inactivity timeout. */
XAM_IMPORT(0x1A2) DWORD XamSetInactivityTimeFromConfig(DWORD Apply);

/**
 * Writes the media type of the disc in the console drive to MediaType and its title id to
 * TitleId. The 2.0.6717 dashboard reads media 1 as an Xbox 360 game and -1 as unknown.
 */
XAM_IMPORT(0x1A3) VOID XamLoaderGetMediaInfo(PDWORD MediaType, PDWORD TitleId);

/**
 * Ends the running title and launches the executable at Path (NULL or a NUL-terminated path)
 * with Flags. A NULL Path returns to the dashboard.
 * Call it only from a thread of the title process: XAM then ends the title and the calling
 * thread with it.
 */
XAM_IMPORT(0x1A4) DECLSPEC_NORETURN VOID XamLoaderLaunchTitle(PCSTR Path, DWORD Flags);

/**
 * Launches the executable at Path with Flags, as XamLoaderLaunchTitle does. The purpose of
 * Second and Third (NUL-terminated strings or NULL) is not known; the 2.0.6717 dashboard
 * passes NULL for all three strings to launch the disc.
 */
XAM_IMPORT(0x1A5) VOID XamLoaderLaunchTitleEx(PCSTR Path, PCSTR Second, PCSTR Third, DWORD Flags);

/**
 * Sets the launch data the next title reads with XamLoaderGetLaunchData. Returns a Win32
 * error code.
 */
XAM_IMPORT(0x1A6) DWORD XamLoaderSetLaunchData(PCVOID Data, DWORD Size);

/**
 * Copies the launch data the previous title set to Buffer, at most Size bytes.
 * Returns a Win32 error code; nonzero when no launch data is set.
 */
XAM_IMPORT(0x1A8) DWORD XamLoaderGetLaunchData(PVOID Buffer, DWORD Size);

/**
 * Ends the running title.
 * Call it only from a thread of the title process: XAM then ends the title and the calling
 * thread with it.
 */
XAM_IMPORT(0x1A9) DECLSPEC_NORETURN VOID XamLoaderTerminateTitle(VOID);

/** Returns the optical drive tray state. */
XAM_IMPORT(0x1AA) DWORD XamLoaderGetDvdTrayState(VOID);

/** Returns 1 when TitleId is a system title id. */
XAM_IMPORT(0x1BB) BOOLEAN XamIsSystemTitleId(DWORD TitleId);

/** Returns the title id of the title that ran before the current one, or 0. */
XAM_IMPORT(0x1BE) DWORD XamLoaderGetPriorTitleId(VOID);

/** Returns nonzero when TitleId is an original Xbox title. */
XAM_IMPORT(0x1BF) BOOLEAN XamIsXbox1TitleId(DWORD TitleId);

/**
 * Gives the PNG of the default gamer picture: stores its address at Image and its length at
 * Size. The 2.0.6717 dashboard does not read the result.
 */
XAM_IMPORT(0x1C5) DWORD XamGetDefaultSystemImage(PCBYTE *Image, PDWORD Size);

/**
 * Writes the NUL-terminated path of the Windows Connect Now configuration file found on a
 * storage device to Path, at most Capacity bytes. Returns ERROR_FILE_NOT_FOUND when no device
 * holds one, or another Win32 error code.
 */
XAM_IMPORT(0x1C6) DWORD XamGetWCNConfigFile(PSTR Path, DWORD Capacity);

/** Mutes (nonzero) or unmutes the sounds of the system UI. */
XAM_IMPORT(0x1CA) VOID XamMuteSound(DWORD Mute);

/**
 * Sets the dashboard context XAM reports to the system UI. The 2.0.6717 dashboard passes 3
 * when initial setup starts and 0 when it completes.
 */
XAM_IMPORT(0x1CC) VOID XamSetDashContext(DWORD Context);

/** Returns the title id of the running title, 0 for none. */
XAM_IMPORT(0x1CF) DWORD XamGetCurrentTitleId(VOID);

/**
 * Returns the Win32 result of the request of Overlapped, or ERROR_IO_INCOMPLETE while it runs
 * and Wait is FALSE. Stores the result length at Length when not NULL.
 */
XAM_IMPORT(0x1FB) DWORD XamGetOverlappedResult(PXOVERLAPPED Overlapped, PDWORD Length, BOOL Wait);

/**
 * Unloads the XAM applications stacked above the dashboard. The 2.0.6717 dashboard passes 0
 * for Flags; the result is not used.
 */
XAM_IMPORT(0x246) DWORD XamAppUnloadStack(DWORD Flags);

/**
 * Creates an enumeration handle served by XAM application App: XamEnumerate on it sends
 * EnumerateMessage, closing it sends CloseMessage. The handle carries a private block of
 * PrivateSize bytes for the application. The 2.0.6717 dashboard passes user -1 and Flags
 * 0x20000001 for its media enumerations. Returns a Win32 error code.
 */
XAM_IMPORT(0x24E) DWORD XamCreateEnumeratorHandle(DWORD UserIndex, DWORD App, DWORD EnumerateMessage,
    DWORD CloseMessage, DWORD PrivateSize, DWORD Count, DWORD Flags, PHANDLE Handle);

/**
 * Stores the object of the private block of an enumeration Handle at Object, with a reference
 * the caller releases with ObDereferenceObject. The 2.0.6717 dashboard zeroes PrivateSize bytes
 * from the object's +0x18, passes that address in its start request and does not read the
 * result, so none is declared.
 */
XAM_IMPORT(0x24F) VOID XamGetPrivateEnumStructureFromHandle(HANDLE Handle, PVOID *Object);

/**
 * Reads the next records of an enumeration into Buffer. Stores the record count at Items when
 * not NULL. Returns a Win32 error code: ERROR_NO_MORE_FILES after the last record, or
 * ERROR_IO_PENDING with an Overlapped, which then holds the count in its length.
 */
XAM_IMPORT(0x250) DWORD XamEnumerate(HANDLE Handle, DWORD Flags, PVOID Buffer, DWORD Size, PDWORD Items,
    PXOVERLAPPED Overlapped);

/**
 * Unloads the system application App. The 2.0.6717 dashboard passes app 0xF6 with Unknown 1
 * before a format, before it deletes the package named IptvClient, and in the IPTV client
 * download (0x9228B9A0); the meaning of Unknown is not known and the result is not read.
 */
XAM_IMPORT(0x252) DWORD XamUnloadSysApp(DWORD App, DWORD Unknown);

/** Returns the system software version. */
XAM_IMPORT(0x282) DWORD XamGetSystemVersion(VOID);

/**
 * Writes the cached name of TitleId as NUL-terminated UTF-16 to Buffer, at most *Length units.
 * Returns a Win32 error code.
 */
XAM_IMPORT(0x2B6) DWORD XamGetCachedTitleName(DWORD TitleId, PWSTR Buffer, PDWORD Length);

/**
 * Clears XAM's content and title caches. The 2.0.6717 dashboard passes zero during storage
 * maintenance and ignores the result.
 */
XAM_IMPORT(0x2B7) DWORD XamCacheReset(DWORD Flags);

/** Returns nonzero when the running system UI was opened by a title. */
XAM_IMPORT(0x2EB) DWORD XamIsSysUiInvokedByTitle(VOID);

/** Returns nonzero while a system UI is shown. */
XAM_IMPORT(0x2EC) BOOLEAN XamIsUIActive(VOID);

/**
 * Writes the locator of the system resource Name to Locator, at most Capacity UTF-16 units:
 * `section://<xam.xex module handle>,shrdres#<name>`, or a `memory://` locator for eight legacy
 * names. In system software 2.0.21256.0 it asserts (`twi 31`) unless the running title is one
 * of two system titles, which halts other titles under the debug monitor.
 */
XAM_IMPORT(0x31C) HRESULT XamBuildLegacySystemResourceLocator(PCWSTR Name, PWSTR Locator, DWORD Capacity);

/** Returns the audio flags (XConfig user setting 0xB). */
XAM_IMPORT(0x3CA) DWORD XGetAudioFlags(VOID);

/** Returns the AV pack value. */
XAM_IMPORT(0x3CB) DWORD XGetAVPack(VOID);

/** Returns the game region of the console (XConfig). */
XAM_IMPORT(0x3CC) DWORD XGetGameRegion(VOID);

/** Stores the current video mode. */
XAM_IMPORT(0x3D1) VOID XGetVideoMode(PXVIDEO_MODE VideoMode);

/** Returns the console language. */
XAM_IMPORT(0x3D2) DWORD XamGetLanguage(VOID);

/** Binds an automation controller to a user slot. */
XAM_IMPORT(0x3D5) DWORD XAutomationpBindController(DWORD UserIndex);

/** Removes the automation controller from a user slot. */
XAM_IMPORT(0x3D6) DWORD XAutomationpUnbindController(DWORD UserIndex);

/** Sends a Guide button press from the automation controller of a user slot. */
XAM_IMPORT(0x3D7) DWORD XAutomationpInputXenonButton(DWORD UserIndex);

/** Allocates from a heap. Returns the address or NULL. */
XAM_IMPORT(0x3FC) PVOID RtlAllocateHeap(HANDLE HeapHandle, DWORD Flags, SIZE_T Size);

/** Frees a heap block. */
XAM_IMPORT(0x3FF) BOOL RtlFreeHeap(HANDLE HeapHandle, DWORD Flags, PVOID BaseAddress);

/** Returns the ID of the current thread. */
XAM_IMPORT(0x410) DWORD GetCurrentThreadId(VOID);

/** Waits on a handle for Milliseconds, or without limit for INFINITE. */
XAM_IMPORT(0x425) DWORD WaitForSingleObject(HANDLE Handle, DWORD Milliseconds);

/** Sets an event to not signaled. */
XAM_IMPORT(0x42B) BOOL ResetEvent(HANDLE Event);

/** Stores the performance counter. */
XAM_IMPORT(0x42F) BOOL QueryPerformanceCounter(PLARGE_INTEGER PerformanceCount);

/** Stores the performance counter frequency. */
XAM_IMPORT(0x430) BOOL QueryPerformanceFrequency(PLARGE_INTEGER Frequency);

/** Decrements the suspend count of a thread. Returns the previous count. */
XAM_IMPORT(0x43D) DWORD ResumeThread(HANDLE Thread);

/** Stores the local time. */
XAM_IMPORT(0x444) VOID GetLocalTime(PSYSTEMTIME SystemTime);

/** Creates an event. Returns the handle or NULL. */
XAM_IMPORT(0x446) HANDLE CreateEventA(PVOID EventAttributes, BOOL ManualReset, BOOL InitialState, PCSTR Name);

/**
 * Creates or opens a file. FileName is a NUL-terminated path. Returns the handle or
 * INVALID_HANDLE_VALUE.
 */
XAM_IMPORT(0x447) HANDLE CreateFileA(PCSTR FileName, ACCESS_MASK DesiredAccess, DWORD ShareMode,
    PVOID SecurityAttributes, CREATION_DISPOSITION CreationDisposition, DWORD FlagsAndAttributes,
    HANDLE TemplateFile);

/** Duplicates a handle. */
XAM_IMPORT(0x45B) BOOL DuplicateHandle(HANDLE SourceProcessHandle, HANDLE SourceHandle,
    HANDLE TargetProcessHandle, PHANDLE TargetHandle, ACCESS_MASK DesiredAccess, BOOL InheritHandle,
    DWORD Options);

/** Removes the IPTV service registration. */
XAM_IMPORT(0x517) HRESULT XamIptvUninstall(VOID);

/**
 * Stores the DVR storage in megabytes: the space given to recordings, the space they use, and
 * the largest allocation the service accepts.
 */
XAM_IMPORT(0x518) HRESULT XamGetDvrStorage(PDWORD Allocated, PDWORD Used, PDWORD Maximum);

/**
 * Sets the space given to recordings, in megabytes. Completes through Overlapped; the 2.0.6717
 * dashboard treats a negative result as a failure.
 */
XAM_IMPORT(0x519) HRESULT XamSetDvrStorage(DWORD Allocated, PXOVERLAPPED Overlapped);

/** Stores a pointer to the NUL-terminated UTF-16 name of the IPTV service at Name. */
XAM_IMPORT(0x51A) HRESULT XamIptvGetServiceName(PCWSTR *Name);

/** Returns the default heap of the process. */
XAM_IMPORT(0x7D2) HANDLE GetProcessHeap(VOID);

/**
 * Copies the background download queue of UserIndex to Items, 0x250-byte records with the
 * package path at +0x138. Count holds the capacity in records on entry and the number copied
 * on return. The 2.0.6717 dashboard passes a capacity of 6 and reads only Count.
 */
XAM_IMPORT(0x9C7) DWORD XamBackgroundDownloadGetItems(DWORD UserIndex, PVOID Items, PDWORD Count);

/** Copies the play timer record to Data. The 2.0.6717 dashboard does not read the result. */
XAM_IMPORT(0xA5A) VOID XamPlayTimerGetData(PXPLAY_TIMER_DATA Data);

/**
 * Replaces the play timer record with Data. The 2.0.6717 dashboard passes 1 for Save and does
 * not read the result.
 */
XAM_IMPORT(0xA5B) VOID XamPlayTimerSetData(const XPLAY_TIMER_DATA *Data, DWORD Save);

/** Resumes the play timer. The 2.0.6717 dashboard passes 1. */
XAM_IMPORT(0xA5C) VOID XamPlayTimerResume(DWORD Resume);

/** Suspends the play timer. */
XAM_IMPORT(0xA5D) VOID XamPlayTimerSuspend(VOID);

/** Whether the play timer is counting. */
XAM_IMPORT(0xA5E) BOOLEAN XamPlayTimerIsRunning(VOID);

/** Whether a play timer period is set. */
XAM_IMPORT(0xA5F) BOOLEAN XamPlayTimerIsEnabled(VOID);

/** Writes the next reset of Period to Data->NextReset. */
XAM_IMPORT(0xA60) VOID XamPlayTimerGetNextResetDate(DWORD Period, PXPLAY_TIMER_DATA Data);

/** Posts the play timer notification. */
XAM_IMPORT(0xA64) VOID XamPlayTimerForceNotification(VOID);

/** The security violations the system has detected, as a 64-bit value. */
XAM_IMPORT(0xAF3) ULONGLONG XamGetSecurityViolationsDetected(VOID);

XBOX360_EXTERN_C_END

#endif
