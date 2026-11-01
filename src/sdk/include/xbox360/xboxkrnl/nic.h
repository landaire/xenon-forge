/*
 * Network controller driver statistics.
 */

#ifndef XBOX360_XBOXKRNL_NIC_H
#define XBOX360_XBOXKRNL_NIC_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* Devices behind the NIC driver, as bits of a device mask. */
#define NIC_DEVICE_EMAC     0x00000001
#define NIC_DEVICE_WIRELESS 0x00000002
#define NIC_DEVICE_KD       0x00000010

/* Classes of a received frame by its destination address. */
typedef enum _NIC_FRAME_CLASS {
    NicFrameOther,
    NicFramePrimaryUnicast,
    NicFrameSecondaryUnicast,
    NicFrameMulticast,
    NicFrameBroadcast,
    NicFrameClassCount
} NIC_FRAME_CLASS;

/*
 * Counters of one device; NicGetStats adds them over the devices of a mask.
 * The structure is 4-byte packed.
 */
#pragma pack(push, 4)
typedef struct _NIC_STATS {
    ULONGLONG XmitBytes;
    ULONGLONG RecvBytes[NicFrameClassCount];
    ULONG XmitPackets;
    ULONG RecvPackets[NicFrameClassCount];
    ULONG XmitUnderrunErrors;
    ULONG XmitOtherErrors;
    ULONG RecvOverrunErrors;
    ULONG RecvFrameErrors;
    ULONG RecvOtherErrors;
} NIC_STATS, *PNIC_STATS;
#pragma pack(pop)

C_ASSERT(__builtin_offsetof(NIC_STATS, XmitPackets) == 0x30);
C_ASSERT(sizeof(NIC_STATS) == 0x5C);

/** Fills a network interface statistics structure for the NIC_DEVICE_* bits of DeviceMask. */
XBOXKRNL_IMPORT(0x209) ULONG NicGetStats(ULONG DeviceMask, PNIC_STATS Stats);

XBOX360_EXTERN_C_END

#endif
