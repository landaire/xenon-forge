/*
 * Run-time library (Rtl): critical sections, singly linked lists, thread
 * contexts, image headers, debug output and formatting.
 */

#ifndef XBOX360_XBOXKRNL_RTL_H
#define XBOX360_XBOXKRNL_RTL_H

#include <xbox360/base.h>
#include <xbox360/xboxkrnl/ntdef.h>
#include <xbox360/xboxkrnl/ke.h>

XBOX360_EXTERN_C_START

/* Register groups of a CONTEXT that hold values (ContextFlags). */
/* MSR, IAR, LR and CTR. */
#define CONTEXT_CONTROL        0x00000001
/* FPR and FPSCR. */
#define CONTEXT_FLOATING_POINT 0x00000002
/* GPR, CR and XER. */
#define CONTEXT_INTEGER        0x00000004
/* VR and VSCR. */
#define CONTEXT_VECTOR         0x00000010
/* Every register group, as RtlCaptureContext requests. */
#define CONTEXT_FULL (CONTEXT_CONTROL | CONTEXT_FLOATING_POINT | CONTEXT_INTEGER | CONTEXT_VECTOR)

/* The registers of a thread. The vector registers need 16-byte alignment. */
typedef struct DECLSPEC_ALIGN(16) _CONTEXT {
    ULONG ContextFlags;
    ULONG Msr;
    ULONG Iar;
    ULONG Lr;
    ULONGLONG Ctr;
    ULONGLONG Gpr[32];
    ULONG Cr;
    ULONG Xer;
    /* Floating point values as IEEE bits. */
    ULONGLONG Fpscr;
    ULONGLONG Fpr[32];
    ULONG UserModeState;
    ULONG Reserved1;
    ULONG Vscr[4];
    ULONG Vr[128][4];
} CONTEXT, *PCONTEXT;

C_ASSERT(__builtin_offsetof(CONTEXT, Ctr) == 0x10);
C_ASSERT(__builtin_offsetof(CONTEXT, Gpr) == 0x18);
C_ASSERT(__builtin_offsetof(CONTEXT, Cr) == 0x118);
C_ASSERT(__builtin_offsetof(CONTEXT, Fpscr) == 0x120);
C_ASSERT(__builtin_offsetof(CONTEXT, Fpr) == 0x128);
C_ASSERT(__builtin_offsetof(CONTEXT, Vscr) == 0x230);
C_ASSERT(__builtin_offsetof(CONTEXT, Vr) == 0x240);
C_ASSERT(sizeof(CONTEXT) == 0xA40);

/*
 * Critical section: a synchronization event that waiters block on, with the
 * spin count in the second byte of the event header, and the ownership
 * state. LockCount is -1 when the section is free.
 */
typedef struct _RTL_CRITICAL_SECTION {
    union {
        KEVENT Event;
        struct {
            UCHAR Type;
            UCHAR SpinCount;
        } Synchronization;
    };
    /* Count of entries minus one. */
    LONG LockCount;
    /* Count of recursive entries of the owner. */
    LONG RecursionCount;
    /* Thread that owns the section, or NULL. */
    struct _KTHREAD *OwningThread;
} RTL_CRITICAL_SECTION, *PRTL_CRITICAL_SECTION;

C_ASSERT(__builtin_offsetof(RTL_CRITICAL_SECTION, LockCount) == 0x10);
C_ASSERT(__builtin_offsetof(RTL_CRITICAL_SECTION, RecursionCount) == 0x14);
C_ASSERT(__builtin_offsetof(RTL_CRITICAL_SECTION, OwningThread) == 0x18);
C_ASSERT(sizeof(RTL_CRITICAL_SECTION) == 0x1C);

/*
 * Head of a sequenced singly linked list: Next in the high word, Depth and
 * Sequence in the low word, updated as one doubleword.
 */
typedef union _SLIST_HEADER {
    ULONGLONG Alignment;
    struct {
        SINGLE_LIST_ENTRY Next;
        USHORT Depth;
        USHORT Sequence;
    };
} SLIST_HEADER, *PSLIST_HEADER;

C_ASSERT(sizeof(SLIST_HEADER) == 8);

/*
 * PE image headers. Image files are little-endian: multibyte fields hold
 * the stored bytes and must be byte-swapped on read.
 */

/* IMAGE_FILE_HEADER.Characteristics */
#define IMAGE_FILE_DLL 0x2000

typedef struct _IMAGE_FILE_HEADER {
    USHORT Machine;
    USHORT NumberOfSections;
    ULONG TimeDateStamp;
    ULONG PointerToSymbolTable;
    ULONG NumberOfSymbols;
    USHORT SizeOfOptionalHeader;
    USHORT Characteristics;
} IMAGE_FILE_HEADER, *PIMAGE_FILE_HEADER;

C_ASSERT(sizeof(IMAGE_FILE_HEADER) == 20);

typedef struct _IMAGE_DATA_DIRECTORY {
    ULONG VirtualAddress;
    ULONG Size;
} IMAGE_DATA_DIRECTORY, *PIMAGE_DATA_DIRECTORY;

C_ASSERT(sizeof(IMAGE_DATA_DIRECTORY) == 8);

#define IMAGE_NUMBEROF_DIRECTORY_ENTRIES 16

typedef struct _IMAGE_OPTIONAL_HEADER {
    USHORT Magic;
    UCHAR MajorLinkerVersion;
    UCHAR MinorLinkerVersion;
    ULONG SizeOfCode;
    ULONG SizeOfInitializedData;
    ULONG SizeOfUninitializedData;
    ULONG AddressOfEntryPoint;
    ULONG BaseOfCode;
    ULONG BaseOfData;
    ULONG ImageBase;
    ULONG SectionAlignment;
    ULONG FileAlignment;
    USHORT MajorOperatingSystemVersion;
    USHORT MinorOperatingSystemVersion;
    USHORT MajorImageVersion;
    USHORT MinorImageVersion;
    USHORT MajorSubsystemVersion;
    USHORT MinorSubsystemVersion;
    ULONG Win32VersionValue;
    ULONG SizeOfImage;
    ULONG SizeOfHeaders;
    ULONG CheckSum;
    USHORT Subsystem;
    USHORT DllCharacteristics;
    ULONG SizeOfStackReserve;
    ULONG SizeOfStackCommit;
    ULONG SizeOfHeapReserve;
    ULONG SizeOfHeapCommit;
    ULONG LoaderFlags;
    ULONG NumberOfRvaAndSizes;
    IMAGE_DATA_DIRECTORY DataDirectory[IMAGE_NUMBEROF_DIRECTORY_ENTRIES];
} IMAGE_OPTIONAL_HEADER, *PIMAGE_OPTIONAL_HEADER;

C_ASSERT(__builtin_offsetof(IMAGE_OPTIONAL_HEADER, SizeOfImage) == 56);
C_ASSERT(__builtin_offsetof(IMAGE_OPTIONAL_HEADER, CheckSum) == 64);
C_ASSERT(sizeof(IMAGE_OPTIONAL_HEADER) == 224);

/* NT headers, as RtlImageNtHeader returns them. The section table follows the optional header. */
typedef struct _IMAGE_NT_HEADERS {
    ULONG Signature;
    IMAGE_FILE_HEADER FileHeader;
    IMAGE_OPTIONAL_HEADER OptionalHeader;
} IMAGE_NT_HEADERS, *PIMAGE_NT_HEADERS;

C_ASSERT(__builtin_offsetof(IMAGE_NT_HEADERS, OptionalHeader) == 24);
C_ASSERT(sizeof(IMAGE_NT_HEADERS) == 248);

#define IMAGE_SIZEOF_SHORT_NAME 8

typedef struct _IMAGE_SECTION_HEADER {
    /* The name, padded with NULs. A name of eight bytes has no NUL. */
    UCHAR Name[IMAGE_SIZEOF_SHORT_NAME];
    ULONG VirtualSize;
    /* Offset of the section from the image base. */
    ULONG VirtualAddress;
    ULONG SizeOfRawData;
    ULONG PointerToRawData;
    ULONG PointerToRelocations;
    ULONG PointerToLinenumbers;
    USHORT NumberOfRelocations;
    USHORT NumberOfLinenumbers;
    ULONG Characteristics;
} IMAGE_SECTION_HEADER, *PIMAGE_SECTION_HEADER;

C_ASSERT(sizeof(IMAGE_SECTION_HEADER) == 40);

/* IMAGE_DEBUG_DIRECTORY.Type */
#define IMAGE_DEBUG_TYPE_CODEVIEW 2

/* One entry of the debug data directory. */
typedef struct _IMAGE_DEBUG_DIRECTORY {
    ULONG Characteristics;
    ULONG TimeDateStamp;
    USHORT MajorVersion;
    USHORT MinorVersion;
    ULONG Type;
    ULONG SizeOfData;
    /* Offset of the debug data from the image base. */
    ULONG AddressOfRawData;
    ULONG PointerToRawData;
} IMAGE_DEBUG_DIRECTORY, *PIMAGE_DEBUG_DIRECTORY;

C_ASSERT(__builtin_offsetof(IMAGE_DEBUG_DIRECTORY, AddressOfRawData) == 20);
C_ASSERT(sizeof(IMAGE_DEBUG_DIRECTORY) == 28);

/* Index of a PE data directory. */
typedef enum _IMAGE_DIRECTORY_ENTRY {
    /* The function table (.pdata). */
    IMAGE_DIRECTORY_ENTRY_EXCEPTION = 3,
    IMAGE_DIRECTORY_ENTRY_DEBUG = 6,
} IMAGE_DIRECTORY_ENTRY;

/** Raises a breakpoint exception to the attached debugger. */
XBOXKRNL_IMPORT(0x001) VOID DbgBreakPoint(VOID);

/** Writes a formatted string to the debug output. */
XBOXKRNL_IMPORT(0x003) ULONG DbgPrint(PCSZ Format, ...);

/** Removes all entries from a singly linked list. Returns the first entry. */
XBOXKRNL_IMPORT(0x02B) PSINGLE_LIST_ENTRY InterlockedFlushSList(PSLIST_HEADER ListHead);

/** Removes the first entry from a singly linked list. Returns the entry or NULL. */
XBOXKRNL_IMPORT(0x02C) PSINGLE_LIST_ENTRY InterlockedPopEntrySList(PSLIST_HEADER ListHead);

/** Reports a failed assertion to the debugger. */
XBOXKRNL_IMPORT(0x118) VOID RtlAssert(
    PVOID FailedAssertion,
    PVOID FileName,
    ULONG LineNumber,
    PCSZ Message);

/** Stores the context of the current thread. */
XBOXKRNL_IMPORT(0x119) VOID RtlCaptureContext(PCONTEXT ContextRecord);

/** Enters a critical section. */
XBOXKRNL_IMPORT(0x125) VOID RtlEnterCriticalSection(PRTL_CRITICAL_SECTION CriticalSection);

/** Stores the low and high stack limits of the current thread. */
XBOXKRNL_IMPORT(0x12A) VOID RtlGetStackLimits(PULONG_PTR StackLimit, PULONG_PTR StackBase);

/** Returns the value or address of an optional header field of a XEX header. Key is an XEX_HEADER_* key. */
XBOXKRNL_IMPORT(0x12B) PVOID RtlImageXexHeaderField(PVOID XexHeaderBase, ULONG Key);

/** Initializes a critical section. */
XBOXKRNL_IMPORT(0x12E) VOID RtlInitializeCriticalSection(PRTL_CRITICAL_SECTION CriticalSection);

/** Initializes a critical section with a spin count. */
XBOXKRNL_IMPORT(0x12F) NTSTATUS RtlInitializeCriticalSectionAndSpinCount(
    PRTL_CRITICAL_SECTION CriticalSection,
    ULONG SpinCount);

/** Leaves a critical section. */
XBOXKRNL_IMPORT(0x130) VOID RtlLeaveCriticalSection(PRTL_CRITICAL_SECTION CriticalSection);

/** Converts an NTSTATUS value to a Win32 error code. */
XBOXKRNL_IMPORT(0x135) ULONG RtlNtStatusToDosError(NTSTATUS Status);

/** Tries to enter a critical section. True when entered. */
XBOXKRNL_IMPORT(0x141) BOOLEAN RtlTryEnterCriticalSection(PRTL_CRITICAL_SECTION CriticalSection);

/**
 * Formats into Buffer from a va_list. ArgList is a console va_list: 8-byte
 * slots, 32-bit values in the low word.
 */
XBOXKRNL_IMPORT(0x14D) INT _vsnprintf(PCHAR Buffer, SIZE_T Count, PCSZ Format, PVOID ArgList);

/** Returns the address of a PE data directory of an image. Its size is stored at Size. */
XBOXKRNL_IMPORT(0x1A8) PVOID RtlImageDirectoryEntryToData(
    PVOID Base,
    BOOLEAN MappedAsImage,
    IMAGE_DIRECTORY_ENTRY DirectoryEntry,
    PULONG Size);

/** Returns the PE NT headers of an image, or NULL. */
XBOXKRNL_IMPORT(0x1A9) PIMAGE_NT_HEADERS RtlImageNtHeader(PVOID Base);

XBOX360_EXTERN_C_END

#endif
