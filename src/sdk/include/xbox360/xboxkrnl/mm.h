/*
 * Memory manager: virtual and physical memory, page protection and memory
 * statistics.
 */

#ifndef XBOX360_XBOXKRNL_MM_H
#define XBOX360_XBOXKRNL_MM_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* Allocation and free types of the virtual memory calls. */
#define MEM_COMMIT      0x00001000
#define MEM_RESERVE     0x00002000
#define MEM_DECOMMIT    0x00004000
#define MEM_RELEASE     0x00008000
#define MEM_FREE        0x00010000
#define MEM_PRIVATE     0x00020000
#define MEM_RESET       0x00080000
#define MEM_TOP_DOWN    0x00100000
#define MEM_NOZERO      0x00800000
#define MEM_LARGE_PAGES 0x20000000 /* 64 KB pages */
#define MEM_HEAP        0x40000000
#define MEM_16MB_PAGES  0x80000000 /* 16 MB pages */

/*
 * Page protection. The physical memory calls take MEM_LARGE_PAGES or
 * MEM_16MB_PAGES in the same word.
 */
#define PAGE_NOACCESS          0x0001
#define PAGE_READONLY          0x0002
#define PAGE_READWRITE         0x0004
#define PAGE_WRITECOPY         0x0008
#define PAGE_EXECUTE           0x0010
#define PAGE_EXECUTE_READ      0x0020
#define PAGE_EXECUTE_READWRITE 0x0040
#define PAGE_EXECUTE_WRITECOPY 0x0080
#define PAGE_GUARD             0x0100
#define PAGE_NOCACHE           0x0200
#define PAGE_WRITECOMBINE      0x0400

/*
 * Memory region of the physical and virtual memory calls. The kernel
 * rejects other values. The debug regions count as the extra memory of a
 * development kit.
 */
#define MM_CURRENT_PROCESS_REGION 0 /* title region for a title thread, else system */
#define MM_TITLE_REGION           1
#define MM_SYSTEM_REGION          2
#define MM_TITLE_DEBUG_REGION     3
#define MM_SYSTEM_DEBUG_REGION    4

/* Region that contains an address, as NtQueryVirtualMemory fills it. */
typedef struct _MEMORY_BASIC_INFORMATION {
    PVOID BaseAddress;
    PVOID AllocationBase;
    ULONG AllocationProtect;
    SIZE_T RegionSize;
    ULONG State; /* MEM_COMMIT, MEM_RESERVE or MEM_FREE */
    ULONG Protect;
    ULONG Type; /* MEM_PRIVATE for an allocated region */
} MEMORY_BASIC_INFORMATION, *PMEMORY_BASIC_INFORMATION;

C_ASSERT(sizeof(MEMORY_BASIC_INFORMATION) == 0x1C);
C_ASSERT(__builtin_offsetof(MEMORY_BASIC_INFORMATION, RegionSize) == 0xC);

/* Memory statistics of MmQueryStatistics. The caller sets Length. */
typedef struct _MM_STATISTICS {
    ULONG Length;
    ULONG TotalPhysicalPages;
    ULONG KernelPages;
    ULONG TitleAvailablePages;
    ULONG TitleTotalVirtualMemoryBytes;
    ULONG TitleReservedVirtualMemoryBytes;
    ULONG TitlePhysicalPages;
    ULONG TitlePoolPages;
    ULONG TitleStackPages;
    ULONG TitleImagePages;
    ULONG TitleHeapPages;
    ULONG TitleVirtualPages;
    ULONG TitlePageTablePages;
    ULONG TitleCachePages;
    ULONG SystemAvailablePages;
    ULONG SystemTotalVirtualMemoryBytes;
    ULONG SystemReservedVirtualMemoryBytes;
    ULONG SystemPhysicalPages;
    ULONG SystemPoolPages;
    ULONG SystemStackPages;
    ULONG SystemImagePages;
    ULONG SystemHeapPages;
    ULONG SystemVirtualPages;
    ULONG SystemPageTablePages;
    ULONG SystemCachePages;
    ULONG HighestPhysicalPage;
} MM_STATISTICS, *PMM_STATISTICS;

C_ASSERT(sizeof(MM_STATISTICS) == 0x68);
C_ASSERT(__builtin_offsetof(MM_STATISTICS, TitleAvailablePages) == 0xC);
C_ASSERT(__builtin_offsetof(MM_STATISTICS, SystemAvailablePages) == 0x38);
C_ASSERT(__builtin_offsetof(MM_STATISTICS, HighestPhysicalPage) == 0x64);

/** Allocates physical memory. Returns the address or NULL. */
XBOXKRNL_IMPORT(0x0B9) PVOID MmAllocatePhysicalMemory(LONG MemoryRegionType, SIZE_T NumberOfBytes, ULONG Protect);

/**
 * Allocates physical memory within an address range and alignment.
 * Returns the address or NULL.
 */
XBOXKRNL_IMPORT(0x0BA) PVOID MmAllocatePhysicalMemoryEx(
    LONG MemoryRegionType,
    SIZE_T NumberOfBytes,
    ULONG Protect,
    ULONG_PTR LowestAcceptableAddress,
    ULONG_PTR HighestAcceptableAddress,
    ULONG_PTR Alignment
);

/** Frees physical memory that MmAllocatePhysicalMemoryEx allocated in MemoryRegionType. */
XBOXKRNL_IMPORT(0x0BD) VOID MmFreePhysicalMemory(LONG MemoryRegionType, PVOID BaseAddress);

/** Translates a mapped virtual address to its physical address. */
XBOXKRNL_IMPORT(0x0BE) ULONG MmGetPhysicalAddress(PVOID BaseAddress);

/** True when an address is mapped. */
XBOXKRNL_IMPORT(0x0BF) BOOLEAN MmIsAddressValid(PVOID VirtualAddress);

/** Returns the page protection of an address. */
XBOXKRNL_IMPORT(0x0C4) ULONG MmQueryAddressProtect(PVOID VirtualAddress);

/** Returns the size of the allocation that contains an address. */
XBOXKRNL_IMPORT(0x0C5) SIZE_T MmQueryAllocationSize(PVOID BaseAddress);

/** Fills MemoryStatistics, whose Length field the caller sets. */
XBOXKRNL_IMPORT(0x0C6) NTSTATUS MmQueryStatistics(PMM_STATISTICS MemoryStatistics);

/** Returns the address when the debugger may read it, else NULL. */
XBOXKRNL_IMPORT(0x1AB) PVOID MmDbgReadCheck(PVOID VirtualAddress);

/** Releases a mapping that MmDbgWriteCheck created. */
XBOXKRNL_IMPORT(0x1AC) VOID MmDbgReleaseAddress(PVOID VirtualAddress, PULONG Opaque);

/**
 * Maps VirtualAddress writable for the debugger. Returns the mapped
 * address, else NULL. Opaque receives the state MmDbgReleaseAddress restores.
 */
XBOXKRNL_IMPORT(0x1AD) PVOID MmDbgWriteCheck(PVOID VirtualAddress, PULONG Opaque);

/** Reserves or commits virtual memory. Base and size are updated in place. */
XBOXKRNL_IMPORT(0x0CC) NTSTATUS NtAllocateVirtualMemory(
    PVOID *BaseAddress,
    PSIZE_T RegionSize,
    ULONG AllocationType,
    ULONG Protect,
    LONG MemoryRegionType
);

/** Decommits or releases virtual memory. Base and size are updated in place. */
XBOXKRNL_IMPORT(0x0DC) NTSTATUS NtFreeVirtualMemory(
    PVOID *BaseAddress,
    PSIZE_T RegionSize,
    ULONG FreeType,
    LONG MemoryRegionType
);

/** Fills a memory basic information structure for an address. */
XBOXKRNL_IMPORT(0x0EE) NTSTATUS NtQueryVirtualMemory(
    PVOID BaseAddress,
    PMEMORY_BASIC_INFORMATION MemoryInformation,
    LONG MemoryRegionType
);

XBOX360_EXTERN_C_END

#endif
