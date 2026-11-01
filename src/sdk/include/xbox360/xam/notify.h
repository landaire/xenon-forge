/*
 * XAM notifications and messages to XAM applications.
 */

#ifndef XBOX360_XAM_NOTIFY_H
#define XBOX360_XAM_NOTIFY_H

#include <xbox360/xam/xamdef.h>

XBOX360_EXTERN_C_START

/* Notification areas of XamNotifyCreateListenerInternal. */
#define XNOTIFY_SYSTEM 0x00000001

/**
 * Takes the next queued notification of Listener and stores its id and parameter.
 * A nonzero MatchId takes only notifications with that id.
 * Returns nonzero when a notification was taken.
 */
XAM_IMPORT(0x28B) BOOLEAN XNotifyGetNext(HANDLE Listener, DWORD MatchId, PDWORD Id, PDWORD Parameter);

/**
 * Sends the notification Id with Parameter to every listener. The 2.0.6717 dashboard
 * sends 0x8000000F with the bit of the user whose background changed and does not read
 * the result, so none is declared.
 */
XAM_IMPORT(0x28E) VOID XNotifyBroadcast(DWORD Id, DWORD Parameter);

/** Creates a notification listener for the notification areas in Mask. Returns the listener handle. */
XAM_IMPORT(0x291) HANDLE XamNotifyCreateListenerInternal(ULONGLONG Mask, DWORD IsSystem, DWORD MaxVersion);

/**
 * Calls a XAM application in the calling process. The meaning of the parameters depends
 * on Message.
 */
XAM_IMPORT(0x1F4) HRESULT XMsgInProcessCall(DWORD App, DWORD Message, PDWORD Parameter1, PDWORD Parameter2);

/**
 * Calls a XAM application synchronously. The dashboard reports a language change with
 * App 0xFE, Message 0x8000000D and no buffer.
 */
XAM_IMPORT(0x1F6) HRESULT XMsgSystemProcessCall(DWORD App, DWORD Message, PVOID Buffer, DWORD BufferLength);

/**
 * Starts an I/O request to a XAM application. Completes through Overlapped when not NULL.
 *
 * With Overlapped, the result is the Win32 code ERROR_IO_PENDING (997) and not an
 * HRESULT, so it reads as a success with facility 0. The outcome of the request is in
 * Overlapped.
 */
XAM_IMPORT(0x1F7) HRESULT XMsgStartIORequest(
    DWORD App,
    DWORD Message,
    PXOVERLAPPED Overlapped,
    PVOID Buffer,
    DWORD BufferLength);

/**
 * Cancels the request of Overlapped. With Wait, returns after the request completed.
 * The 2.0.6717 dashboard always waits and does not use the result.
 */
XAM_IMPORT(0x1F8) DWORD XMsgCancelIORequest(PXOVERLAPPED Overlapped, BOOL Wait);

/**
 * Starts an I/O request to a XAM application with an extra parameter block. The 2.0.6717
 * dashboard queries a video license with App 0xFA, Message 0x7005D, a 16-byte request and
 * Extended pointing to the words 0x20080002 and 0. Without Overlapped the call completes
 * before it returns; the dashboard then also takes a nonzero thread error as failure.
 */
XAM_IMPORT(0x1FC) HRESULT XMsgStartIORequestEx(
    DWORD App,
    DWORD Message,
    PXOVERLAPPED Overlapped,
    PVOID Buffer,
    DWORD BufferLength,
    PDWORD Extended);

XBOX360_EXTERN_C_END

#endif
