/*
 * Hardware abstraction layer: the system management controller (SMC),
 * firmware reentry, console hardware information and kernel version.
 */

#ifndef XBOX360_XBOXKRNL_HAL_H
#define XBOX360_XBOXKRNL_HAL_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* A request to the SMC and its reply are 16 bytes; byte 0 is the command. */
#define SMC_MESSAGE_LENGTH 16

/* SMC commands and message kinds (byte 0 of an SMC_MESSAGE). */
#define SMC_COMMAND_I2C              0x11
#define SMC_COMMAND_QUERY_IR_ADDRESS 0x16 /* low nibble of reply byte 1 */
#define SMC_COMMAND_TRAY_STATE       0x83 /* byte 1 is 0x60 plus the state */
#define SMC_COMMAND_SET_IR_ADDRESS   0x95 /* low nibble of byte 1 */

/* Bus address of the EEPROM that holds the debug monitor settings. */
#define SMC_I2C_SETTINGS_EEPROM 0xD4

typedef struct _SMC_MESSAGE {
    UCHAR Command;
    UCHAR Data[SMC_MESSAGE_LENGTH - 1];
} SMC_MESSAGE, *PSMC_MESSAGE;

C_ASSERT(sizeof(SMC_MESSAGE) == 16);

struct _HAL_SMC_REGISTRATION;

/* SMC notification routine; it runs at dispatch level. */
typedef VOID (*PHAL_SMC_NOTIFICATION)(struct _HAL_SMC_REGISTRATION *Registration, PSMC_MESSAGE Message);

/*
 * Registration of an SMC notification routine. Routines of higher
 * Priority run first. The kernel owns ListEntry while it is registered.
 */
typedef struct _HAL_SMC_REGISTRATION {
    PHAL_SMC_NOTIFICATION NotificationRoutine;
    LONG Priority;
    LIST_ENTRY ListEntry;
} HAL_SMC_REGISTRATION, *PHAL_SMC_REGISTRATION;

C_ASSERT(sizeof(HAL_SMC_REGISTRATION) == 16);
C_ASSERT(__builtin_offsetof(HAL_SMC_REGISTRATION, ListEntry) == 8);

/* Routine of HalReturnToFirmware. */
typedef enum _FIRMWARE_REENTRY {
    HalHaltRoutine = 0,
    HalRebootRoutine = 1,
    HalKdRebootRoutine = 2,
    HalReserved3Routine = 3,
    HalFatalErrorRebootRoutine = 4, /* reset through the SMC */
    HalPowerDownRoutine = 5,
    HalRebootQuiesceRoutine = 6, /* restarts the system software */
    HalForcedPowerDownRoutine = 7,
    HalForcedRebootQuiesceRoutine = 8,
} FIRMWARE_REENTRY;

/* XBOX_HARDWARE_INFO.Flags. The top four bits hold the hardware generation. */
#define XBOX_HW_FLAG_HOST_BRIDGE_EBEB         0x00000008
#define XBOX_HW_FLAG_AUTO_PROFILE             0x00000010 /* setting 3, 8 is available */
#define XBOX_HW_FLAG_HDD                      0x00000020
#define XBOX_HW_FLAG_KERNEL_DEBUGGER_ENABLED  0x00000080 /* enabled at boot */
#define XBOX_HW_FLAG_KERNEL_DEBUGGER_ATTACHED 0x00000100
#define XBOX_HW_FLAG_LARGE_MEMORY             0x00010000 /* 1 GB of memory */
#define XBOX_HW_FLAG_MMC_FLASH                0x00020000
#define XBOX_HW_FLAG_TEST_KIT                 0x02000000
#define XBOX_HW_GENERATION_SHIFT              28
#define XBOX_HW_GENERATION_TRINITY            4

/* XBOX_HARDWARE_INFO.BootFlags: the console booted from a disc. */
#define XBOX_BOOT_FLAG_FROM_DISC 0x0400

/* Console hardware description. The debug monitor changes Flags at run time. */
typedef struct _XBOX_HARDWARE_INFO {
    ULONG Flags;
    UCHAR NumberOfProcessors;
    UCHAR Revision;
    UCHAR Reserved1[6];
    USHORT BootLoaderData;
    USHORT BootFlags;
} XBOX_HARDWARE_INFO, *PXBOX_HARDWARE_INFO;

C_ASSERT(sizeof(XBOX_HARDWARE_INFO) == 0x10);
C_ASSERT(__builtin_offsetof(XBOX_HARDWARE_INFO, BootFlags) == 0xE);

typedef struct _XBOX_KRNL_VERSION {
    USHORT Major;
    USHORT Minor;
    USHORT Build;
    USHORT Qfe;
} XBOX_KRNL_VERSION, *PXBOX_KRNL_VERSION;

C_ASSERT(sizeof(XBOX_KRNL_VERSION) == 8);

/** Returns the AV pack value the hardware reports. */
XBOXKRNL_IMPORT(0x022) ULONG HalGetCurrentAVPack(VOID);

/** Opens or closes the optical drive tray. */
XBOXKRNL_IMPORT(0x024) VOID HalOpenCloseODDTray(ULONG Open);

/**
 * Registers a system management controller event callback when Register
 * is true and removes it otherwise. Registration must stay valid while
 * registered.
 */
XBOXKRNL_IMPORT(0x027) VOID HalRegisterSMCNotification(PHAL_SMC_REGISTRATION Registration, BOOLEAN Register);

/**
 * Restarts or shuts down the console. Stock XBDM code continues after the
 * call, so it is not declared to diverge.
 */
XBOXKRNL_IMPORT(0x028) VOID HalReturnToFirmware(FIRMWARE_REENTRY Routine);

/**
 * Sends a 16-byte message to the system management controller and
 * receives the 16-byte reply.
 */
XBOXKRNL_IMPORT(0x029) VOID HalSendSMCMessage(PSMC_MESSAGE Request, PSMC_MESSAGE Reply);

/** True while the power-down deferred procedure call runs. */
XBOXKRNL_IMPORT(0x267) ULONG HalIsExecutingPowerDownDpc(VOID);

/** Console hardware information. */
XBOXKRNL_IMPORT(0x156) extern PXBOX_HARDWARE_INFO __imp_XboxHardwareInfo;
#define XboxHardwareInfo (*__imp_XboxHardwareInfo)

/** Version of the base kernel. */
XBOXKRNL_IMPORT(0x157) extern PXBOX_KRNL_VERSION __imp_XboxKrnlBaseVersion;
#define XboxKrnlBaseVersion (*__imp_XboxKrnlBaseVersion)

/** Version of the running kernel. */
XBOXKRNL_IMPORT(0x158) extern PXBOX_KRNL_VERSION __imp_XboxKrnlVersion;
#define XboxKrnlVersion (*__imp_XboxKrnlVersion)

XBOX360_EXTERN_C_END

#endif
