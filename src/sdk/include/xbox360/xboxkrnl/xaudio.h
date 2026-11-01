/*
 * Audio render driver.
 */

#ifndef XBOX360_XBOXKRNL_XAUDIO_H
#define XBOX360_XBOXKRNL_XAUDIO_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* Argument of a render driver client callback, built for each frame. */
typedef struct _XAUDIORENDERDRVCALLBACKDATA {
    PVOID Context;
} XAUDIORENDERDRVCALLBACKDATA, *PXAUDIORENDERDRVCALLBACKDATA;

C_ASSERT(sizeof(XAUDIORENDERDRVCALLBACKDATA) == 4);

typedef VOID (*PXAUDIORENDERDRVCALLBACK)(PXAUDIORENDERDRVCALLBACKDATA CallbackData);

/* Render driver client: the callback runs once per audio frame. */
typedef struct _XAUDIORENDERDRVCLIENT {
    PXAUDIORENDERDRVCALLBACK Callback;
    PVOID Context;
} XAUDIORENDERDRVCLIENT, *PXAUDIORENDERDRVCLIENT;

C_ASSERT(sizeof(XAUDIORENDERDRVCLIENT) == 8);

/** Registers an audio render client. The client handle is stored at Client. */
XBOXKRNL_IMPORT(0x1F3) HRESULT XAudioRegisterRenderDriverClient(const XAUDIORENDERDRVCLIENT *ClientData, PVOID *Client);

/** Removes an audio render client. */
XBOXKRNL_IMPORT(0x1F4) HRESULT XAudioUnregisterRenderDriverClient(PVOID Client);

/** Submits a frame of samples for an audio render client. */
XBOXKRNL_IMPORT(0x1F5) HRESULT XAudioSubmitRenderDriverFrame(PVOID Client, const FLOAT *Samples);

/** Returns the audio render driver thread object. */
XBOXKRNL_IMPORT(0x1FE) struct _KTHREAD *XAudioGetRenderDriverThread(VOID);

/** Sets the speaker configuration (the XConfig audio flags value). */
XBOXKRNL_IMPORT(0x200) HRESULT XAudioSetSpeakerConfig(ULONG SpeakerConfig, ULONG Reserved);

XBOX360_EXTERN_C_END

#endif
