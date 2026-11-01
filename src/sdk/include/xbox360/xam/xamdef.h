/*
 * XAM types that most imports use.
 */

#ifndef XBOX360_XAM_XAMDEF_H
#define XBOX360_XAM_XAMDEF_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

struct _XOVERLAPPED;

typedef VOID (*PXOVERLAPPED_COMPLETION_ROUTINE)(DWORD ErrorCode, DWORD Length, struct _XOVERLAPPED *Overlapped);

/*
 * State of an asynchronous XAM request. InternalLow holds the Win32 error
 * code, ERROR_IO_PENDING until the request completes.
 */
typedef struct _XOVERLAPPED {
    ULONG_PTR InternalLow;
    ULONG_PTR InternalHigh;
    ULONG_PTR InternalContext;
    /* Event that XAM signals on completion, or NULL. */
    HANDLE hEvent;
    PXOVERLAPPED_COMPLETION_ROUTINE pCompletionRoutine;
    DWORD_PTR dwCompletionContext;
    DWORD dwExtendedError;
} XOVERLAPPED, *PXOVERLAPPED;

C_ASSERT(sizeof(XOVERLAPPED) == 0x1c);

XBOX360_EXTERN_C_END

#endif
