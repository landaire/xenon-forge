/*
 * Hardware XMA audio decoder contexts.
 */

#ifndef XBOX360_XBOXKRNL_XMA_H
#define XBOX360_XBOXKRNL_XMA_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/*
 * Hardware context of the XMA decoder, 64 bytes in an array that the decoder
 * reads from physical memory. Bitfields are assigned from the most
 * significant bit. Buffer offsets and counts are in the units of the
 * decoder: input packets of 2 KB, output blocks of 256 bytes.
 */
typedef struct _XMA_CONTEXT_DATA {
    /* DWORD 0 */
    DWORD OutputBufferWriteOffset : 5;
    DWORD OutputBufferBlockCount : 5;
    DWORD InputBuffer1Valid : 1;
    DWORD InputBuffer0Valid : 1;
    DWORD LoopCount : 8;
    DWORD InputBuffer0PacketCount : 12;

    /* DWORD 1 */
    DWORD OutputBufferValid : 1;
    DWORD Reserved1A : 1;
    DWORD IsStereo : 1;
    DWORD SampleRate : 2;
    DWORD Reserved1B : 3;
    DWORD SubframeDecodeCount : 4;
    DWORD LoopSubframeSkip : 3;
    DWORD Reserved1C : 3;
    DWORD LoopSubframeEnd : 2;
    DWORD InputBuffer1PacketCount : 12;

    /* DWORD 2 */
    DWORD Reserved2 : 6;
    DWORD InputBufferReadOffset : 26;

    /* DWORD 3 */
    DWORD Reserved3 : 6;
    DWORD LoopStartOffset : 26;

    /* DWORD 4 */
    DWORD Reserved4A : 1;
    DWORD PacketMetadata : 5;
    DWORD LoopEndOffset : 26;

    /* DWORDs 5 to 8: physical addresses. */
    DWORD InputBuffer0;
    DWORD InputBuffer1;
    DWORD OutputBuffer;
    DWORD WorkBuffer;

    /* DWORD 9 */
    DWORD Reserved9 : 27;
    DWORD OutputBufferReadOffset : 5;

    /* DWORDs 10 to 15. */
    DWORD Reserved[6];
} XMA_CONTEXT_DATA, *PXMA_CONTEXT_DATA;

C_ASSERT(__builtin_offsetof(XMA_CONTEXT_DATA, InputBuffer0) == 0x14);
C_ASSERT(sizeof(XMA_CONTEXT_DATA) == 0x40);

/* Loop region of a context. Offsets are bit offsets in the input stream. */
typedef struct _XMA_LOOP_DATA {
    DWORD LoopStartOffset;
    DWORD LoopEndOffset;
    UCHAR LoopCount;
    UCHAR LoopSubframeEnd;
    UCHAR LoopSubframeSkip;
    UCHAR Reserved;
} XMA_LOOP_DATA, *PXMA_LOOP_DATA;

C_ASSERT(sizeof(XMA_LOOP_DATA) == 0xC);

/*
 * Initial state of a context for XMAInitializeContext. Buffers are virtual
 * addresses of physically contiguous memory.
 */
typedef struct _XMA_CONTEXT_INIT {
    PVOID InputBuffer0;
    DWORD InputBuffer0PacketCount;
    PVOID InputBuffer1;
    DWORD InputBuffer1PacketCount;
    DWORD InputBufferReadOffset;
    PVOID OutputBuffer;
    DWORD OutputBufferBlockCount;
    PVOID WorkBuffer;
    DWORD SubframeDecodeCount;
    DWORD IsStereo;
    DWORD SampleRate;
    XMA_LOOP_DATA LoopData;
} XMA_CONTEXT_INIT, *PXMA_CONTEXT_INIT;

C_ASSERT(__builtin_offsetof(XMA_CONTEXT_INIT, LoopData) == 0x2C);
C_ASSERT(sizeof(XMA_CONTEXT_INIT) == 0x38);

/** Allocates one hardware XMA decoder context. */
XBOXKRNL_IMPORT(0x224) HRESULT XMACreateContext(PXMA_CONTEXT_DATA *Context);

/** Initializes an allocated XMA decoder context. */
XBOXKRNL_IMPORT(0x225) VOID XMAInitializeContext(PXMA_CONTEXT_DATA Context, const XMA_CONTEXT_INIT *ContextInit);

/** Releases a hardware XMA decoder context. */
XBOXKRNL_IMPORT(0x226) VOID XMAReleaseContext(PXMA_CONTEXT_DATA Context);

/** Enables a hardware XMA decoder context. */
XBOXKRNL_IMPORT(0x227) VOID XMAEnableContext(PXMA_CONTEXT_DATA Context);

/** Disables a hardware XMA decoder context. Wait waits for in-flight decoder work. */
XBOXKRNL_IMPORT(0x228) HRESULT XMADisableContext(PXMA_CONTEXT_DATA Context, BOOL Wait);

/** Returns the decoder's output ring write offset in 256-byte blocks. */
XBOXKRNL_IMPORT(0x229) DWORD XMAGetOutputBufferWriteOffset(PXMA_CONTEXT_DATA Context);

/** Sets the output ring read offset in 256-byte blocks. */
XBOXKRNL_IMPORT(0x22A) VOID XMASetOutputBufferReadOffset(PXMA_CONTEXT_DATA Context, DWORD Offset);

/** Returns the output ring read offset in 256-byte blocks. */
XBOXKRNL_IMPORT(0x22B) DWORD XMAGetOutputBufferReadOffset(PXMA_CONTEXT_DATA Context);

/** Marks the output ring of an XMA decoder context valid. */
XBOXKRNL_IMPORT(0x22C) VOID XMASetOutputBufferValid(PXMA_CONTEXT_DATA Context);

/** Returns whether the output ring is valid. */
XBOXKRNL_IMPORT(0x22D) BOOL XMAIsOutputBufferValid(PXMA_CONTEXT_DATA Context);

/** Marks input buffer zero of an XMA decoder context valid. */
XBOXKRNL_IMPORT(0x22E) VOID XMASetInputBuffer0Valid(PXMA_CONTEXT_DATA Context);

/** Returns whether input buffer zero is valid. */
XBOXKRNL_IMPORT(0x22F) BOOL XMAIsInputBuffer0Valid(PXMA_CONTEXT_DATA Context);

/** Waits until the decoder is not working on Context. */
XBOXKRNL_IMPORT(0x235) VOID XMABlockWhileInUse(PXMA_CONTEXT_DATA Context);

XBOX360_EXTERN_C_END

#endif
