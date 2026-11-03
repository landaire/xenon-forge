/*
 * Base types and macros of the Xbox 360 headers.
 *
 * Titles use 32-bit pointers, int and long on 64-bit registers. Clang's
 * powerpc-unknown-none-eabi and powerpc64-unknown-xbox360-ilp32 targets both give
 * this data model. The headers include no system header.
 *
 * Every import is declared with XBOX360_IMPORT(library, ordinal). The
 * annotation names the library and export ordinal. xbox360-bindgen reads it
 * to generate the import thunks that C and Rust code link with. A module
 * exports a function with XBOX360_EXPORT(ordinal, function).
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

/*
 * Export `function` at `ordinal`, an integer literal from 1 to 0xFFFF, at
 * file scope. xex-link rejects two exports at one ordinal. Ordinals that
 * share a function are declared once each.
 *
 * Callers use the console calling convention, which expects r14-r31 back in
 * full 64 bits. Code of the powerpc (SysV) code generation saves only their
 * low words, so the export is an entry like the `_start` of the C runtime:
 * it saves r14-r31 in full, reserves the home area that console callees
 * store their arguments in, and calls `function` with r3-r10 unchanged.
 * `function` takes at most eight 32-bit integer or pointer arguments and
 * returns a 32-bit result or none.
 *
 * The export is an 8-byte `.xexports` record, as `xbox360_sys::export!`
 * writes: the big-endian ordinal, then the address of the entry. xex-link
 * turns the records into the XEX export table.
 */
#define XBOX360_EXPORT(ordinal, function) \
    XBOX360_EXPORT_IMPL(ordinal, function, __xbox360_export_##ordinal)

/* XBOX360_EXPORT with `name`, the symbol of the entry. */
#define XBOX360_EXPORT_IMPL(ordinal, function, name)                                \
    _Static_assert((ordinal) >= 1 && (ordinal) <= 0xFFFF,                           \
                   "export ordinals are 1 to 0xFFFF");                              \
    __attribute__((used)) static __typeof__(function) *const name##_target          \
        __asm__(#name "_target") = function;                                        \
    extern VOID name(VOID) __asm__(#name);                                          \
    __attribute__((used, retain, section(".xexports"))) static const struct {       \
        ULONG Ordinal;                                                              \
        VOID (*Entry)(VOID);                                                        \
    } name##_record = {(ordinal), name};                                            \
    __asm__(".pushsection .text." #name ", \"ax\", @progbits\n"                     \
            ".globl " #name "\n"                                                    \
            ".type " #name ", @function\n"                                          \
            ".balign 4\n" #name ":\n"                                               \
            "stwu 1, -0xe0(1)\n"                                                    \
            "mflr 0\n"                                                              \
            "stw 0, 0x8(1)\n"                                                       \
            ".irp reg, 14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31\n"     \
            "std \\reg, 0x50 + 8 * (\\reg - 14)(1)\n"                               \
            ".endr\n"                                                               \
            "lis 12, " #name "_target@ha\n"                                         \
            "lwz 12, " #name "_target@l(12)\n"                                      \
            "mtctr 12\n"                                                            \
            "bctrl\n"                                                               \
            ".irp reg, 14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31\n"     \
            "ld \\reg, 0x50 + 8 * (\\reg - 14)(1)\n"                                \
            ".endr\n"                                                               \
            "lwz 0, 0x8(1)\n"                                                       \
            "mtlr 0\n"                                                              \
            "addi 1, 1, 0xe0\n"                                                     \
            "blr\n"                                                                 \
            ".size " #name ", . - " #name "\n"                                      \
            ".popsection")

#endif
