/*
 * XEX loader: loaded module records, XEX headers and the Xex routines.
 */

#ifndef XBOX360_XBOXKRNL_XEX_H
#define XBOX360_XBOXKRNL_XEX_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* XEX_HEADER.ModuleFlags, and the module flags of XexLoadImage. */
#define XEX_MODULE_FLAG_TITLE            0x00000001
#define XEX_MODULE_FLAG_EXPORTS_TO_TITLE 0x00000002
#define XEX_MODULE_FLAG_SYSTEM_DEBUGGER  0x00000004
#define XEX_MODULE_FLAG_DLL              0x00000008
#define XEX_MODULE_FLAG_PATCH            0x00000010
#define XEX_MODULE_FLAG_PATCH_FULL       0x00000020
#define XEX_MODULE_FLAG_PATCH_DELTA      0x00000040
#define XEX_MODULE_FLAG_USER_MODE        0x00000080

/* Options of XexLoadImage, beside the module flags. */
#define XEX_LOAD_FLAG_REQUIRE_DEVICE_ID  0x20000000
#define XEX_LOAD_FLAG_REQUIRE_BOUND_PATH 0x40000000
#define XEX_LOAD_FLAG_NO_FAILURE_REPORT  0x80000000

/*
 * Packed XEX version: 4, 4, 16 and 8 bits of major, minor, build and qfe
 * from the top. As a minimum version, XEX_VERSION_ANY accepts every image.
 */
#define XEX_VERSION(Major, Minor, Build, Qfe) \
    ((ULONG)(((Major) & 0xF) << 28 | ((Minor) & 0xF) << 24 | ((Build) & 0xFFFF) << 8 | ((Qfe) & 0xFF)))
#define XEX_VERSION_ANY 0

/* Module handle argument that stands for the module of the caller. */
#define XEX_MODULE_HANDLE_CALLER ((PLDR_DATA_TABLE_ENTRY)-1)

/* Reason the loader passes to the entry point of a module. */
#define DLL_PROCESS_DETACH 0
#define DLL_PROCESS_ATTACH 1

/*
 * Entry of a C or C++ DLL or system module that links xbox360//src/sdk:crt-dll,
 * which calls it with C linkage. FALSE on DLL_PROCESS_ATTACH fails the load.
 */
BOOL DllMain(HANDLE Module, DWORD Reason, PVOID Reserved);

/*
 * Optional header entry. The low byte of Key gives the form of the data:
 * 0, Value is the data; 1, Value holds one ULONG; 0xFF, Value is the offset
 * of data that starts with its ULONG size; otherwise Value is the offset of
 * Key & 0xFF ULONGs. XEX headers are big-endian.
 */
typedef struct _XEX_OPTIONAL_HEADER {
    ULONG Key;
    ULONG Value;
} XEX_OPTIONAL_HEADER, *PXEX_OPTIONAL_HEADER;

C_ASSERT(sizeof(XEX_OPTIONAL_HEADER) == 8);

#define XEX_HEADER_MAGIC 0x58455832 /* 'XEX2' */

typedef struct _XEX_HEADER {
    ULONG Magic;
    ULONG ModuleFlags;
    ULONG SizeOfHeaders;
    ULONG SizeOfDiscardableHeaders;
    ULONG SecurityInfo;
    ULONG OptionalHeaderCount;
    XEX_OPTIONAL_HEADER OptionalHeaders[1];
} XEX_HEADER, *PXEX_HEADER;

C_ASSERT(sizeof(XEX_HEADER) == 0x20);
C_ASSERT(__builtin_offsetof(XEX_HEADER, OptionalHeaders) == 0x18);

/* Optional header keys. */
#define XEX_HEADER_RESOURCE_INFO           0x000002FF
#define XEX_HEADER_FILE_FORMAT_INFO        0x000003FF
#define XEX_HEADER_DELTA_PATCH_DESCRIPTOR  0x000005FF
#define XEX_HEADER_CONSOLE_FEATURES        0x00004004
#define XEX_HEADER_VALIDITY_PERIOD         0x00004104
#define XEX_HEADER_CONSOLE_ID_LIST         0x000042FF
#define XEX_HEADER_BOUNDING_PATH           0x000080FF
#define XEX_HEADER_DEVICE_ID               0x00008105
#define XEX_HEADER_ORIGINAL_BASE_ADDRESS   0x00010001
#define XEX_HEADER_ENTRY_POINT             0x00010100
#define XEX_HEADER_IMAGE_BASE_ADDRESS      0x00010201
#define XEX_HEADER_IMPORT_LIBRARIES        0x000103FF
#define XEX_HEADER_CHECKSUM_TIMESTAMP      0x00018002
#define XEX_HEADER_ENABLED_FOR_CALLCAP     0x00018102
#define XEX_HEADER_ENABLED_FOR_FASTCAP     0x00018200
#define XEX_HEADER_ORIGINAL_PE_NAME        0x000183FF
/* 16-byte entries {Name[8], Major, Minor, Build, Qfe} after the size. */
#define XEX_HEADER_STATIC_LIBRARIES        0x000200FF
#define XEX_HEADER_TLS_INFO                0x00020104
#define XEX_HEADER_DEFAULT_STACK_SIZE      0x00020200
#define XEX_HEADER_FILESYSTEM_CACHE_SIZE   0x00020301
#define XEX_HEADER_SYSTEM_FLAGS            0x00030000
#define XEX_HEADER_EXECUTION_INFO          0x00040006
#define XEX_HEADER_TITLE_WORKSPACE_SIZE    0x00040201
#define XEX_HEADER_LAN_KEY                 0x00040404
#define XEX_HEADER_ADDITIONAL_TITLE_MEMORY 0x00040801

/* Data of XEX_HEADER_EXECUTION_INFO. */
typedef struct _XEX_EXECUTION_INFO {
    ULONG MediaId;
    ULONG Version;
    ULONG BaseVersion;
    ULONG TitleId;
    UCHAR Platform;
    UCHAR ExecutableType;
    UCHAR DiscNumber;
    UCHAR DiscCount;
    ULONG SavegameId;
} XEX_EXECUTION_INFO, *PXEX_EXECUTION_INFO;

C_ASSERT(sizeof(XEX_EXECUTION_INFO) == 0x18);
C_ASSERT(__builtin_offsetof(XEX_EXECUTION_INFO, TitleId) == 0xC);
C_ASSERT(__builtin_offsetof(XEX_EXECUTION_INFO, SavegameId) == 0x14);

/* Imports of a module: Count entries, NULL for an import that is not loaded. */
typedef struct _LDR_IMPORT_TABLE {
    ULONG Count;
    struct _LDR_DATA_TABLE_ENTRY *Entries[1];
} LDR_IMPORT_TABLE, *PLDR_IMPORT_TABLE;

C_ASSERT(sizeof(LDR_IMPORT_TABLE) == 8);

/*
 * The loader's record of a loaded module. Its address is the module
 * handle. The loader changes the links and the flags under its lock.
 */
typedef struct _LDR_DATA_TABLE_ENTRY {
    LIST_ENTRY InLoadOrderLinks;
    LIST_ENTRY InClosureOrderLinks;
    LIST_ENTRY InInitializationOrderLinks;
    PVOID NtHeadersBase; /* address the image is loaded at */
    PVOID ImageBase;
    ULONG SizeOfNtImage;
    UNICODE_STRING FullDllName;
    UNICODE_STRING BaseDllName;
    ULONG Flags;
    ULONG SizeOfFullImage;
    PVOID EntryPoint;
    USHORT LoadCount;
    USHORT ModuleIndex;
    PVOID OriginalBaseAddress; /* function table in kernel 17559 and later */
    ULONG CheckSum;
    ULONG ModuleFlags; /* XEX_HEADER.ModuleFlags */
    ULONG TimeDateStamp;
    PLDR_IMPORT_TABLE LoadedImports;
    PXEX_HEADER XexHeaderBase;
    union {
        ANSI_STRING LoadFileName;
        struct {
            struct _LDR_DATA_TABLE_ENTRY *ClosureRoot;
            struct _LDR_DATA_TABLE_ENTRY *TraversalParent;
        } s;
    } u;
} LDR_DATA_TABLE_ENTRY, *PLDR_DATA_TABLE_ENTRY;

C_ASSERT(sizeof(LDR_DATA_TABLE_ENTRY) == 0x64);
C_ASSERT(__builtin_offsetof(LDR_DATA_TABLE_ENTRY, NtHeadersBase) == 0x18);
C_ASSERT(__builtin_offsetof(LDR_DATA_TABLE_ENTRY, SizeOfNtImage) == 0x20);
C_ASSERT(__builtin_offsetof(LDR_DATA_TABLE_ENTRY, FullDllName) == 0x24);
C_ASSERT(__builtin_offsetof(LDR_DATA_TABLE_ENTRY, BaseDllName) == 0x2C);
C_ASSERT(__builtin_offsetof(LDR_DATA_TABLE_ENTRY, Flags) == 0x34);
C_ASSERT(__builtin_offsetof(LDR_DATA_TABLE_ENTRY, TimeDateStamp) == 0x50);
C_ASSERT(__builtin_offsetof(LDR_DATA_TABLE_ENTRY, XexHeaderBase) == 0x58);

/* LDR_DATA_TABLE_ENTRY.Flags */
#define LDR_FLAG_LOAD_IN_PROGRESS 0x00001000
#define LDR_FLAG_UNLOAD_PENDING   0x00002000

/** Module handle of the running executable: the address of its loader entry. */
XBOXKRNL_IMPORT(0x193) extern PLDR_DATA_TABLE_ENTRY *__imp_XexExecutableModuleHandle;
#define XexExecutableModuleHandle (*__imp_XexExecutableModuleHandle)

/** Finds a loaded module by name and stores its handle. */
XBOXKRNL_IMPORT(0x195) NTSTATUS XexGetModuleHandle(PCSZ ModuleName, PLDR_DATA_TABLE_ENTRY *ModuleHandle);

/**
 * Finds the resource SectionName of a module and stores its address and
 * size. SectionName must equal the stored name, which is at most 8 bytes,
 * byte for byte, so case is significant. XEX_MODULE_HANDLE_CALLER as Module
 * names the caller's module. Fails with STATUS_NOT_FOUND when the module
 * has no such resource.
 */
XBOXKRNL_IMPORT(0x196) NTSTATUS XexGetModuleSection(
    PLDR_DATA_TABLE_ENTRY Module,
    PCSZ SectionName,
    PVOID *SectionData,
    PULONG SectionSize
);

/**
 * Finds an export of a module by ordinal and stores its address. Ordinal
 * may instead be the address of a NUL-terminated export name.
 */
XBOXKRNL_IMPORT(0x197) NTSTATUS XexGetProcedureAddress(
    PLDR_DATA_TABLE_ENTRY ModuleHandle,
    ULONG Ordinal,
    PVOID *ProcedureAddress
);

/**
 * Loads a XEX image and stores its module handle. MinimumVersion is the
 * oldest version accepted (see XEX_VERSION).
 */
XBOXKRNL_IMPORT(0x199) NTSTATUS XexLoadImage(
    PCSZ ImageName,
    ULONG LoadFlags,
    ULONG MinimumVersion,
    PLDR_DATA_TABLE_ENTRY *ModuleHandle
);

/** Finds the loader entry of the module that contains an address. Returns the image base. */
XBOXKRNL_IMPORT(0x19C) PVOID XexPcToFileHeader(PVOID PcValue, PLDR_DATA_TABLE_ENTRY *DataTableEntry);

/** Unloads a XEX image. */
XBOXKRNL_IMPORT(0x1A1) NTSTATUS XexUnloadImage(PLDR_DATA_TABLE_ENTRY ModuleHandle);

XBOX360_EXTERN_C_END

#endif
