/*
 * Video display driver: display modes, the GPU command processor and front
 * buffer swaps.
 */

#ifndef XBOX360_XBOXKRNL_VD_H
#define XBOX360_XBOXKRNL_VD_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* Display information block that VdGetCurrentDisplayInformation fills. */
typedef struct _VD_DISPLAY_INFORMATION {
    USHORT SurfaceWidth;
    USHORT SurfaceHeight;
    UCHAR ColorSpace;
    UCHAR Reserved005[3];
    ULONG Left;
    ULONG Top;
    ULONG Right;
    ULONG Bottom;
    ULONG ScaledWidth;
    ULONG ScaledHeight;
    ULONG Reserved020[14];
} VD_DISPLAY_INFORMATION, *PVD_DISPLAY_INFORMATION;

C_ASSERT(__builtin_offsetof(VD_DISPLAY_INFORMATION, ColorSpace) == 4);
C_ASSERT(__builtin_offsetof(VD_DISPLAY_INFORMATION, Left) == 8);
C_ASSERT(__builtin_offsetof(VD_DISPLAY_INFORMATION, ScaledHeight) == 0x1C);
C_ASSERT(sizeof(VD_DISPLAY_INFORMATION) == 0x58);

/*
 * System command buffer descriptor that VdGetSystemCommandBuffer fills and
 * VdSwap reads.
 */
typedef struct _VD_SYSTEM_COMMAND_BUFFER {
    /* GPU address of the buffer, or 0 when the kernel has none. */
    ULONG Address;
    ULONG Dwords;
    /* Nonzero when the following state is valid for the swap outputs. */
    ULONG State;
    ULONG Reserved00C[34];
} VD_SYSTEM_COMMAND_BUFFER, *PVD_SYSTEM_COMMAND_BUFFER;

C_ASSERT(sizeof(VD_SYSTEM_COMMAND_BUFFER) == 0x94);

/* Parameter block of VdSetDisplayModeOverride. */
typedef struct _VD_DISPLAY_MODE_OVERRIDE_PARAMETERS {
    /* sizeof(VD_DISPLAY_MODE_OVERRIDE_PARAMETERS). */
    ULONG Size;
    ULONG Reserved004[2];
    /* Retry index of the override. */
    UCHAR Attempt;
    UCHAR Reserved00D[3];
} VD_DISPLAY_MODE_OVERRIDE_PARAMETERS, *PVD_DISPLAY_MODE_OVERRIDE_PARAMETERS;

C_ASSERT(__builtin_offsetof(VD_DISPLAY_MODE_OVERRIDE_PARAMETERS, Attempt) == 0xC);
C_ASSERT(sizeof(VD_DISPLAY_MODE_OVERRIDE_PARAMETERS) == 0x10);

/*
 * Block that VdGetDisplayDiscoveryData fills. Only bytes +0, +4 and the
 * unaligned ULONG at +0x1D are known to be used.
 */
typedef struct _VD_DISPLAY_DISCOVERY_DATA {
    ULONG Reserved000[20];
} VD_DISPLAY_DISCOVERY_DATA, *PVD_DISPLAY_DISCOVERY_DATA;

C_ASSERT(sizeof(VD_DISPLAY_DISCOVERY_DATA) == 0x50);

/*
 * Mode reported by VdEnumerateVideoModes. Source and Preference rank the
 * EDID entry the mode came from; DisplayType is the display type value.
 */
#pragma pack(push, 2)
typedef struct _VD_VIDEO_MODE_INFO {
    USHORT Size;
    USHORT Width;
    USHORT Height;
    UCHAR Source;
    UCHAR Preference;
    FLOAT RefreshRate;
    ULONG ModeID : 8;
    ULONG Reserved : 19;
    ULONG Audio : 1;
    ULONG YCbCr : 1;
    ULONG Interlaced : 1;
    ULONG StandardAspectRatio : 1;
    ULONG Valid : 1;
    ULONG PixelRate;
    UCHAR DisplayType;
    UCHAR PixelRepetition;
} VD_VIDEO_MODE_INFO, *PVD_VIDEO_MODE_INFO;
#pragma pack(pop)

C_ASSERT(__builtin_offsetof(VD_VIDEO_MODE_INFO, RefreshRate) == 8);
C_ASSERT(sizeof(VD_VIDEO_MODE_INFO) == 0x16);

/* Called for each mode by VdEnumerateVideoModes; returns zero to stop. */
typedef ULONG (*PVD_ENUMERATE_VIDEO_MODES_ROUTINE)(const VD_VIDEO_MODE_INFO *Mode, PVOID Context);

/* Called by VdShutdownEngines with the context given to VdInitializeEngines. */
typedef VOID (*PVD_ENGINES_SHUTDOWN_ROUTINE)(PVOID Context);

/* Called by the graphics interrupt handler with (0, Context). */
typedef VOID (*PVD_GRAPHICS_INTERRUPT_ROUTINE)(ULONG Source, PVOID Context);

/**
 * Makes the command processor write its ring read pointer, in dwords, to
 * PhysicalAddress (dword aligned). BlockSizeLog2 is the read pointer block
 * size field of the ring control register.
 */
XBOXKRNL_IMPORT(0x1B6) VOID VdEnableRingBufferRPtrWriteBack(ULONG PhysicalAddress, ULONG BlockSizeLog2);

/** Fills a display information structure. */
XBOXKRNL_IMPORT(0x1BA) VOID VdGetCurrentDisplayInformation(PVD_DISPLAY_INFORMATION DisplayInformation);

/**
 * Copies the next of the kernel's three system command buffer descriptors
 * to Buffer and stores in Identifier the value the GPU writes to the
 * identifier address once that buffer has run.
 */
XBOXKRNL_IMPORT(0x1BD) VOID VdGetSystemCommandBuffer(PVD_SYSTEM_COMMAND_BUFFER Buffer, PULONG Identifier);

/**
 * Gives the GPU command processor to the calling title: loads the CP
 * microcode (the kernel's own when both pointers are null), stores
 * ShutdownRoutine and Context for VdShutdownEngines and resets the engines.
 * Unused is not read. Returns 1. Traps when the engines are already owned.
 */
XBOXKRNL_IMPORT(0x1C2) ULONG VdInitializeEngines(
    ULONG Unused,
    PVD_ENGINES_SHUTDOWN_ROUTINE ShutdownRoutine,
    PVOID Context,
    const ULONG *PrefetchMicrocode,
    const ULONG *EngineMicrocode);

/**
 * Points the command processor ring at PhysicalAddress with
 * 2^(SizeLog2 + 1) dwords and resets its read and write pointers.
 */
XBOXKRNL_IMPORT(0x1C3) VOID VdInitializeRingBuffer(ULONG PhysicalAddress, ULONG SizeLog2);

/** Reads a video encoder register. */
XBOXKRNL_IMPORT(0x1CB) ULONG VdReadDVERegisterUlong(ULONG Register);

/**
 * Programs the configured video mode. Flags selects display-off,
 * forced-output, color-space, and pixel-format options.
 */
XBOXKRNL_IMPORT(0x1D3) VOID VdSetDisplayMode(ULONG Flags);

/**
 * Overrides the display mode; nonzero on success. Scan is 1 for interlaced
 * and 2 for progressive [INFERENCE]; Parameters may be null. All zero
 * arguments remove the override. RefreshRate is passed in f1 and the console
 * ABI also reserves r5 for it, so Scan and Parameters are in r6 and r7.
 */
XBOXKRNL_IMPORT(0x1D4) ULONG VdSetDisplayModeOverride(
    ULONG Width,
    ULONG Height,
    DOUBLE RefreshRate,
    ULONG Scan,
    const VD_DISPLAY_MODE_OVERRIDE_PARAMETERS *Parameters);

/** Sets the routine the graphics interrupt handler calls with (0, Context), or none when null. */
XBOXKRNL_IMPORT(0x1D5) VOID VdSetGraphicsInterruptCallback(PVD_GRAPHICS_INTERRUPT_ROUTINE Routine, PVOID Context);

/**
 * Sets the virtual address of the dword where the GPU writes the identifier
 * of each finished system command buffer, or none when null.
 */
XBOXKRNL_IMPORT(0x1D9) VOID VdSetSystemCommandBufferGpuIdentifierAddress(PULONG Address);

/**
 * Waits for the GPU to idle, returns the command processor to the kernel's
 * own ring and calls the shutdown routine of VdInitializeEngines.
 */
XBOXKRNL_IMPORT(0x1DC) VOID VdShutdownEngines(VOID);

/** Writes a video encoder register. */
XBOXKRNL_IMPORT(0x1E0) ULONG VdWriteDVERegisterUlong(ULONG Register, ULONG Value);

/**
 * Writes a 64-dword front buffer flip into Commands: when SystemBuffer holds
 * a buffer, fetch constant 15 from the 6-dword FetchConstant (base address
 * virtual) and an indirect buffer call of it, then an event write of
 * Identifier to IdentifierAddress (virtual), padded with a NOP. The five
 * outputs are written only when SystemBuffer carries optional state.
 */
XBOXKRNL_IMPORT(0x25B) VOID VdSwap(
    PULONG Commands,
    const ULONG *FetchConstant,
    PULONG IdentifierAddress,
    const VD_SYSTEM_COMMAND_BUFFER *SystemBuffer,
    ULONG Identifier,
    PULONG FrontBuffer,
    PULONG TextureFormat,
    PULONG ColorSpace,
    PULONG Width,
    PULONG Height);

/**
 * A step of EDRAM retraining. Returns 0 when nothing is due: VdHSIOTrainCount
 * (export 0x26B) is nonzero and Flags has bit 1 clear. In state 0 it sets
 * *State to 1, returns 1 and leaves *Commands alone, so a call with a null
 * *Commands asks whether retraining is due. D3D then calls it with *Commands
 * at free ring space until it returns 0, and takes the advanced *Commands as
 * the end of what was written. Returns 0 without effect when *State is above
 * 7, Capacity (dwords) below 0x928 or WorkBytes below 0x5C. A null State
 * traps.
 */
XBOXKRNL_IMPORT(0x269) ULONG VdRetrainEDRAM(
    ULONG Flags,
    PULONG *Commands,
    ULONG Capacity,
    PULONG State,
    PUCHAR Work,
    ULONG WorkBytes);

/**
 * When VdHSIOTrainCount is 0: sends message 7 to the system management
 * controller and stores byte 6 of the reply as the current EDRAM temperature
 * of the retraining state. Returns 0.
 */
XBOXKRNL_IMPORT(0x26A) ULONG VdRetrainEDRAMWorker(ULONG Reserved);

/**
 * Calls Routine with each display mode the attached display supports; the
 * routine returns nonzero to continue.
 */
XBOXKRNL_IMPORT(0x270) VOID VdEnumerateVideoModes(PVD_ENUMERATE_VIDEO_MODES_ROUTINE Routine, PVOID Context);

/** Fills Data with the display discovery result; nonzero on success. */
XBOXKRNL_IMPORT(0x27E) ULONG VdGetDisplayDiscoveryData(PVD_DISPLAY_DISCOVERY_DATA Data);

XBOX360_EXTERN_C_END

#endif
