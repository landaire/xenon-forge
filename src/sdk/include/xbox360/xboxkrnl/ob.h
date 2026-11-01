/*
 * Object manager (Ob): object types, access masks, object directories and
 * symbolic links.
 */

#ifndef XBOX360_XBOXKRNL_OB_H
#define XBOX360_XBOXKRNL_OB_H

#include <xbox360/base.h>
#include <xbox360/xboxkrnl/ntdef.h>
#include <xbox360/xboxkrnl/ke.h>

XBOX360_EXTERN_C_START

/* Standard and generic rights of an ACCESS_MASK. */
#define DELETE          0x00010000
#define READ_CONTROL    0x00020000
#define WRITE_DAC       0x00040000
#define WRITE_OWNER     0x00080000
#define SYNCHRONIZE     0x00100000
#define MAXIMUM_ALLOWED 0x02000000
#define GENERIC_ALL     0x10000000
#define GENERIC_EXECUTE 0x20000000
#define GENERIC_WRITE   0x40000000
#define GENERIC_READ    0x80000000

/* Allocate the object from title or system memory regardless of the caller. */
#define OBJ_TITLE_OBJECT  0x00001000
#define OBJ_SYSTEM_OBJECT 0x00002000

struct _OBJECT_TYPE;

typedef PVOID (*OB_ALLOCATE_METHOD)(SIZE_T NumberOfBytes, ULONG Tag, ULONG PoolType);
typedef VOID (*OB_FREE_METHOD)(PVOID Pointer);
typedef VOID (*OB_CLOSE_METHOD)(PVOID Object, ULONG SystemHandleCount);
typedef VOID (*OB_DELETE_METHOD)(PVOID Object);
typedef NTSTATUS (*OB_PARSE_METHOD)(
    PVOID ParseObject,
    struct _OBJECT_TYPE *ObjectType,
    ULONG Attributes,
    POBJECT_STRING CompleteName,
    POBJECT_STRING RemainingName,
    PVOID Context,
    PVOID *Object);

/* An object type, such as ExThreadObjectType. Object calls take its address. */
typedef struct _OBJECT_TYPE {
    OB_ALLOCATE_METHOD AllocateProcedure;
    OB_FREE_METHOD FreeProcedure;
    OB_CLOSE_METHOD CloseProcedure;
    OB_DELETE_METHOD DeleteProcedure;
    OB_PARSE_METHOD ParseProcedure;
    PVOID DefaultObject;
    ULONG PoolTag;
} OBJECT_TYPE, *POBJECT_TYPE;

C_ASSERT(sizeof(OBJECT_TYPE) == 28);

/* Entry of NtQueryDirectoryObject; the name characters follow it. */
typedef struct _OBJECT_DIRECTORY_INFORMATION {
    OBJECT_STRING Name;
    /* PoolTag of the object's type. */
    ULONG Type;
} OBJECT_DIRECTORY_INFORMATION, *POBJECT_DIRECTORY_INFORMATION;

C_ASSERT(sizeof(OBJECT_DIRECTORY_INFORMATION) == 12);

/** Creates an object directory. */
XBOXKRNL_IMPORT(0x0D0) NTSTATUS NtCreateDirectoryObject(
    PHANDLE DirectoryHandle,
    POBJECT_ATTRIBUTES ObjectAttributes);

/** Opens an object directory. */
XBOXKRNL_IMPORT(0x0DE) NTSTATUS NtOpenDirectoryObject(
    PHANDLE DirectoryHandle,
    POBJECT_ATTRIBUTES ObjectAttributes);

/** Reads one entry of an object directory. Context holds the enumeration position. */
XBOXKRNL_IMPORT(0x0E5) NTSTATUS NtQueryDirectoryObject(
    HANDLE DirectoryHandle,
    POBJECT_DIRECTORY_INFORMATION DirectoryInformation,
    ULONG Length,
    BOOLEAN RestartScan,
    PULONG Context,
    PULONG ReturnLength);

/** Allocates an object of type ObjectType with a body of ObjectBodySize bytes. */
XBOXKRNL_IMPORT(0x102) NTSTATUS ObCreateObject(
    POBJECT_TYPE ObjectType,
    POBJECT_ATTRIBUTES ObjectAttributes,
    ULONG ObjectBodySize,
    PVOID *Object);

/** Creates a symbolic link object from SymbolicLinkName to DeviceName. */
XBOXKRNL_IMPORT(0x103) NTSTATUS ObCreateSymbolicLink(
    POBJECT_STRING SymbolicLinkName,
    POBJECT_STRING DeviceName);

/** Deletes the symbolic link object SymbolicLinkName. */
XBOXKRNL_IMPORT(0x104) NTSTATUS ObDeleteSymbolicLink(POBJECT_STRING SymbolicLinkName);

/** Releases a reference to an object. */
XBOXKRNL_IMPORT(0x105) VOID ObDereferenceObject(PVOID Object);

/**
 * Inserts an object into the handle table and object directory. The handle
 * is stored at Handle.
 */
XBOXKRNL_IMPORT(0x108) NTSTATUS ObInsertObject(
    PVOID Object,
    POBJECT_ATTRIBUTES ObjectAttributes,
    ULONG ObjectPointerBias,
    PHANDLE Handle);

/** Finds a thread object by thread ID and references it. */
XBOXKRNL_IMPORT(0x10A) NTSTATUS ObLookupAnyThreadByThreadId(HANDLE ThreadId, PKTHREAD *Thread);

/** Removes the permanent flag of a named object. */
XBOXKRNL_IMPORT(0x10C) VOID ObMakeTemporaryObject(PVOID Object);

/** Opens a named object and stores a handle. */
XBOXKRNL_IMPORT(0x10D) NTSTATUS ObOpenObjectByName(
    POBJECT_ATTRIBUTES ObjectAttributes,
    POBJECT_TYPE ObjectType,
    PVOID ParseContext,
    PHANDLE Handle);

/** References the object of a handle. A NULL ObjectType accepts any type. */
XBOXKRNL_IMPORT(0x110) NTSTATUS ObReferenceObjectByHandle(
    HANDLE Handle,
    POBJECT_TYPE ObjectType,
    PVOID *ReturnedObject);

/** References a named object. Attributes are OBJ_* flags. */
XBOXKRNL_IMPORT(0x111) NTSTATUS ObReferenceObjectByName(
    POBJECT_STRING ObjectName,
    ULONG Attributes,
    POBJECT_TYPE ObjectType,
    PVOID ParseContext,
    PVOID *Object);

/** Reads the target of a symbolic link object into OutputString. */
XBOXKRNL_IMPORT(0x113) NTSTATUS ObTranslateSymbolicLink(
    POBJECT_ATTRIBUTES ObjectAttributes,
    POBJECT_STRING OutputString);

XBOX360_EXTERN_C_END

#endif
