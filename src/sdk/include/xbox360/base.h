/*
 * Base types and macros of the Xbox 360 headers.
 *
 * Titles use 32-bit pointers, int and long on 64-bit registers. Clang's
 * powerpc-unknown-none-eabi and powerpc64-unknown-xbox360-ilp32 targets both give
 * this data model. The headers include no system header.
 *
 * Every import is declared with XBOX360_IMPORT(library, ordinal). The
 * annotation names the library and export ordinal. xbox360-bindgen reads it
 * to generate the import thunks that C and Rust code link with.
 */

#ifndef XBOX360_BASE_H
#define XBOX360_BASE_H

#ifdef __cplusplus
#define XBOX360_EXTERN_C_START extern "C" {
#define XBOX360_EXTERN_C_END   }
#else
#define XBOX360_EXTERN_C_START
#define XBOX360_EXTERN_C_END
#endif

/*
 * An import of `library` (xboxkrnl, xam or xbdm) by `ordinal`. The thunk
 * library defines a function import under its own name. A data import is
 * declared as its slot, `__imp_<name>`, which holds the address of the
 * export after the loader resolves it.
 */
#define XBOX360_IMPORT(library, ordinal) \
    __attribute__((annotate("xbox360_import," #library "," #ordinal)))

#define XBOXKRNL_IMPORT(ordinal) XBOX360_IMPORT(xboxkrnl, ordinal)
#define XAM_IMPORT(ordinal)      XBOX360_IMPORT(xam, ordinal)
#define XBDM_IMPORT(ordinal)     XBOX360_IMPORT(xbdm, ordinal)

#define DECLSPEC_NORETURN         __attribute__((noreturn))
#define DECLSPEC_ALIGN(Alignment) __attribute__((aligned(Alignment)))
#define C_ASSERT(Expression)      _Static_assert(Expression, #Expression)

XBOX360_EXTERN_C_START

typedef void VOID, *PVOID;
typedef const void *PCVOID;

typedef char CHAR, *PCHAR, *PSTR;
typedef const char *PCSTR, *PCSZ;
typedef unsigned char UCHAR, *PUCHAR;
typedef unsigned char BYTE, *PBYTE;
typedef const unsigned char *PCBYTE;

typedef short SHORT, *PSHORT;
typedef unsigned short USHORT, *PUSHORT;
typedef unsigned short WORD, *PWORD;
/* wchar_t is 32 bits on powerpc-unknown-none-eabi; the console's is 16. */
typedef unsigned short WCHAR, *PWCHAR, *PWSTR;
typedef const unsigned short *PCWSTR;

typedef int INT, *PINT;
typedef unsigned int UINT, *PUINT;
typedef long LONG, *PLONG;
typedef unsigned long ULONG, *PULONG;
typedef unsigned long DWORD, *PDWORD;
typedef long long LONGLONG, *PLONGLONG;
typedef unsigned long long ULONGLONG, *PULONGLONG;
typedef unsigned long long QWORD, *PQWORD;
typedef float FLOAT, *PFLOAT;
typedef double DOUBLE, *PDOUBLE;

typedef LONG LONG_PTR, *PLONG_PTR;
typedef ULONG ULONG_PTR, *PULONG_PTR;
typedef ULONG SIZE_T, *PSIZE_T;
typedef ULONG DWORD_PTR, *PDWORD_PTR;

typedef UCHAR BOOLEAN, *PBOOLEAN;
typedef int BOOL, *PBOOL;

typedef LONG NTSTATUS;
typedef LONG HRESULT;
typedef PVOID HANDLE, *PHANDLE;
typedef ULONG ACCESS_MASK, *PACCESS_MASK;

/* A user's Xbox Live identity. */
typedef ULONGLONG XUID, *PXUID;

#define TRUE  1
#define FALSE 0
#ifndef NULL
#ifdef __cplusplus
#define NULL 0
#else
#define NULL ((void *)0)
#endif
#endif

#define NT_SUCCESS(Status) ((NTSTATUS)(Status) >= 0)

/* Doubly linked list entry; a list head is its own entry when empty. */
typedef struct _LIST_ENTRY {
    struct _LIST_ENTRY *Flink;
    struct _LIST_ENTRY *Blink;
} LIST_ENTRY, *PLIST_ENTRY;

typedef struct _SINGLE_LIST_ENTRY {
    struct _SINGLE_LIST_ENTRY *Next;
} SINGLE_LIST_ENTRY, *PSINGLE_LIST_ENTRY;

/* 64-bit integers also addressed as two halves; the high half comes first. */
typedef union _LARGE_INTEGER {
    struct {
        LONG HighPart;
        ULONG LowPart;
    };
    LONGLONG QuadPart;
} LARGE_INTEGER, *PLARGE_INTEGER;

typedef union _ULARGE_INTEGER {
    struct {
        ULONG HighPart;
        ULONG LowPart;
    };
    ULONGLONG QuadPart;
} ULARGE_INTEGER, *PULARGE_INTEGER;

C_ASSERT(sizeof(PVOID) == 4);
C_ASSERT(sizeof(LONG) == 4);
C_ASSERT(sizeof(LONGLONG) == 8);
C_ASSERT(sizeof(LIST_ENTRY) == 8);
C_ASSERT(sizeof(LARGE_INTEGER) == 8);

XBOX360_EXTERN_C_END

#endif
