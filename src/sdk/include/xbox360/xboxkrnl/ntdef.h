/*
 * Kernel types that most imports use: counted strings, object attributes,
 * I/O status blocks and the dispatcher header of waitable objects.
 */

#ifndef XBOX360_XBOXKRNL_NTDEF_H
#define XBOX360_XBOXKRNL_NTDEF_H

#include <xbox360/base.h>

XBOX360_EXTERN_C_START

typedef UCHAR KPROCESSOR_MODE;
typedef UCHAR KIRQL, *PKIRQL;
typedef LONG KPRIORITY;

#define KernelMode 0
#define UserMode   1

/* Counted 8-bit string; lengths are in bytes. */
typedef struct _STRING {
    USHORT Length;
    USHORT MaximumLength;
    PCHAR Buffer;
} STRING, *PSTRING;

typedef STRING ANSI_STRING, *PANSI_STRING;
typedef STRING OBJECT_STRING, *POBJECT_STRING;

/* Counted 16-bit string; lengths are in bytes. */
typedef struct _UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PWSTR Buffer;
} UNICODE_STRING, *PUNICODE_STRING;

C_ASSERT(sizeof(STRING) == 8);
C_ASSERT(sizeof(UNICODE_STRING) == 8);

/* Object attributes. The console layout has no length or security fields. */
typedef struct _OBJECT_ATTRIBUTES {
    HANDLE RootDirectory;
    POBJECT_STRING ObjectName;
    ULONG Attributes;
} OBJECT_ATTRIBUTES, *POBJECT_ATTRIBUTES;

C_ASSERT(sizeof(OBJECT_ATTRIBUTES) == 12);

#define OBJ_INHERIT       0x00000002
#define OBJ_PERMANENT     0x00000010
#define OBJ_EXCLUSIVE     0x00000020
#define OBJ_CASE_INSENSITIVE 0x00000040
#define OBJ_OPENIF        0x00000080
#define OBJ_OPENLINK      0x00000100
#define OBJ_KERNEL_HANDLE 0x00000200

/* Final status and information of an I/O request. */
typedef struct _IO_STATUS_BLOCK {
    union {
        NTSTATUS Status;
        PVOID Pointer;
    };
    ULONG_PTR Information;
} IO_STATUS_BLOCK, *PIO_STATUS_BLOCK;

C_ASSERT(sizeof(IO_STATUS_BLOCK) == 8);

typedef VOID (*PIO_APC_ROUTINE)(PVOID ApcContext, PIO_STATUS_BLOCK IoStatusBlock, ULONG Reserved);

/* Header of every object a thread can wait on. */
typedef struct _DISPATCHER_HEADER {
    UCHAR Type;
    UCHAR Absolute;
    UCHAR ProcessType;
    UCHAR Inserted;
    LONG SignalState;
    LIST_ENTRY WaitListHead;
} DISPATCHER_HEADER, *PDISPATCHER_HEADER;

C_ASSERT(sizeof(DISPATCHER_HEADER) == 16);

typedef enum _EVENT_TYPE {
    NotificationEvent = 0,
    SynchronizationEvent = 1,
} EVENT_TYPE;

typedef struct _KEVENT {
    DISPATCHER_HEADER Header;
} KEVENT, *PKEVENT;

C_ASSERT(sizeof(KEVENT) == 16);

XBOX360_EXTERN_C_END

#endif
