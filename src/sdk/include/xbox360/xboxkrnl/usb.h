/*
 * USB devices: controller radio and USB camera driver.
 */

#ifndef XBOX360_XBOXKRNL_USB_H
#define XBOX360_XBOXKRNL_USB_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* Completion routine of an asynchronous USB camera request. */
typedef VOID (*PXUSBCAM_COMPLETION_ROUTINE)(LONG Status, PVOID Context);

/** Turns the controller radio on or off. */
XBOXKRNL_IMPORT(0x1EF) ULONG XInputdSetRFPowerMode(ULONG Enable);

/** Writes one USB camera configuration value. */
XBOXKRNL_IMPORT(0x21C) LONG XUsbcamSetConfig(
    ULONG Camera,
    ULONG Request,
    ULONG Selector,
    const UCHAR *Value,
    ULONG Length,
    PXUSBCAM_COMPLETION_ROUTINE CompletionRoutine,
    PVOID Context);

/** State of the USB camera driver. The dashboard reads 2 as ready. */
XBOXKRNL_IMPORT(0x21D) ULONG XUsbcamGetState(VOID);

/**
 * Creates the USB camera driver context in Workspace and stores its camera
 * handle. Returns zero on success.
 */
XBOXKRNL_IMPORT(0x222) ULONG XUsbcamCreate(PVOID Workspace, ULONG WorkspaceSize, PULONG Camera);

/** Destroys a USB camera driver context. */
XBOXKRNL_IMPORT(0x223) VOID XUsbcamDestroy(ULONG Camera);

XBOX360_EXTERN_C_END

#endif
