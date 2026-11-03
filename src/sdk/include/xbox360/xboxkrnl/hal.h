/*
 * Hardware abstraction layer: the system management controller (SMC),
 * firmware reentry, console hardware information and kernel version.
 */

#ifndef XBOX360_XBOXKRNL_HAL_H
#define XBOX360_XBOXKRNL_HAL_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/*
 * A request to the SMC and its reply are 16 bytes; byte 0 is the command.
 * Commands below 0x80 are queries: the reply repeats the command in byte 0
 * and reply byte n + 1 is Data[n]. Commands from 0x80 on get no reply, so
 * HalSendSMCMessage takes a null Reply for them. The SMC does not answer
 * a query it does not know, and HalSendSMCMessage with a Reply then waits
 * for good on a retail kernel.
 *
 * The names and layouts follow SMC firmware 510301; the SMC firmware
 * differs between consoles. Query SMC_COMMAND_QUERY_VERSION for the build.
 */
#define SMC_MESSAGE_LENGTH 16

/* SMC commands and message kinds (byte 0 of an SMC_MESSAGE). */
#define SMC_COMMAND_QUERY_POWER_UP_CAUSE 0x01 /* reply SMC_POWER_UP_CAUSE_REPLY */
#define SMC_COMMAND_QUERY_TEMPERATURES   0x07 /* reply SMC_TEMPERATURES_REPLY */
#define SMC_COMMAND_I2C                  0x11
#define SMC_COMMAND_QUERY_VERSION        0x12 /* reply SMC_VERSION_REPLY */
#define SMC_COMMAND_QUERY_IR_ADDRESS     0x16 /* low nibble of reply byte 1 */
#define SMC_COMMAND_TRAY_STATE           0x83 /* byte 1 is 0x60 plus the state */
#define SMC_COMMAND_SET_FAN_ALGORITHM    0x88 /* SMC_FAN_ALGORITHM_REQUEST */
#define SMC_COMMAND_SET_FAN_SPEED        0x89 /* SMC_FAN_SPEED_REQUEST */
#define SMC_COMMAND_SET_POWER_LED        0x8C /* SMC_POWER_LED_REQUEST */
#define SMC_COMMAND_SET_FAN2_SPEED       0x94 /* SMC_FAN_SPEED_REQUEST; some SMC builds only */
#define SMC_COMMAND_SET_IR_ADDRESS       0x95 /* low nibble of byte 1 */
#define SMC_COMMAND_SET_RING_OF_LIGHT    0x99 /* SMC_RING_OF_LIGHT_REQUEST */

/* Bus address of the EEPROM that holds the debug monitor settings. */
#define SMC_I2C_SETTINGS_EEPROM 0xD4

typedef struct _SMC_MESSAGE {
    UCHAR Command;
    UCHAR Data[SMC_MESSAGE_LENGTH - 1];
} SMC_MESSAGE, *PSMC_MESSAGE;

C_ASSERT(sizeof(SMC_MESSAGE) == 16);

/*
 * Reply of SMC_COMMAND_QUERY_POWER_UP_CAUSE. The values of Cause are not
 * established.
 */
typedef struct _SMC_POWER_UP_CAUSE_REPLY {
    UCHAR Command;
    UCHAR Cause;
    UCHAR Reserved1;
    UCHAR BootAttempts;
    UCHAR Status; /* SMC_POWER_UP_STATUS_* */
    UCHAR Reserved2[11];
} SMC_POWER_UP_CAUSE_REPLY, *PSMC_POWER_UP_CAUSE_REPLY;

C_ASSERT(sizeof(SMC_POWER_UP_CAUSE_REPLY) == 16);

/* SMC_POWER_UP_CAUSE_REPLY.Status: the SMC configuration block is invalid. */
#define SMC_POWER_UP_STATUS_CONFIG_INVALID 0x01

/*
 * A temperature of SMC_TEMPERATURES_REPLY: 8.8 fixed point degrees,
 * fraction byte first. Degrees Celsius is inferred from the kernel's log.
 */
typedef struct _SMC_TEMPERATURE {
    UCHAR Fraction; /* 1/256 degrees */
    UCHAR Degrees;
} SMC_TEMPERATURE, *PSMC_TEMPERATURE;

C_ASSERT(sizeof(SMC_TEMPERATURE) == 2);

/*
 * Reply of SMC_COMMAND_QUERY_TEMPERATURES. Only EdramTemperature has a
 * known source; the devices Temperature0, Temperature1 and Temperature3
 * measure are not established. FanSpeed is the requested speed in
 * percent, not the measured duty.
 */
typedef struct _SMC_TEMPERATURES_REPLY {
    UCHAR Command;
    SMC_TEMPERATURE Temperature0;
    SMC_TEMPERATURE Temperature1;
    SMC_TEMPERATURE EdramTemperature;
    SMC_TEMPERATURE Temperature3;
    UCHAR FanSpeed;
    UCHAR Reserved[6];
} SMC_TEMPERATURES_REPLY, *PSMC_TEMPERATURES_REPLY;

C_ASSERT(sizeof(SMC_TEMPERATURES_REPLY) == 16);
C_ASSERT(__builtin_offsetof(SMC_TEMPERATURES_REPLY, EdramTemperature) == 5);
C_ASSERT(__builtin_offsetof(SMC_TEMPERATURES_REPLY, FanSpeed) == 9);

/*
 * Reply of SMC_COMMAND_QUERY_VERSION. Version holds the firmware build as
 * hex digits: 51 03 01 is build 510301. BootCause0 and BootCause1 are the
 * bytes the host last set with SMC command 0x9c; the kernel reads the boot
 * cause from BootCause1 bits 7-4. PowerButtonPresses counts power button
 * presses. Layout of SMC 510301.
 */
typedef struct _SMC_VERSION_REPLY {
    UCHAR Command;
    UCHAR Version[3];
    UCHAR BootCause0;
    UCHAR BootCause1;
    UCHAR PowerButtonPresses;
    UCHAR Reserved[9];
} SMC_VERSION_REPLY, *PSMC_VERSION_REPLY;

C_ASSERT(sizeof(SMC_VERSION_REPLY) == 16);
C_ASSERT(__builtin_offsetof(SMC_VERSION_REPLY, BootCause0) == 4);
C_ASSERT(__builtin_offsetof(SMC_VERSION_REPLY, PowerButtonPresses) == 6);

/*
 * Request of SMC_COMMAND_SET_FAN_ALGORITHM. SMC_FAN_ALGORITHM_FULL_SPEED
 * is dangerous: the SMC keeps it until power-down and stops polling the
 * temperature sensors meanwhile, so it reports a sensor timeout and shuts
 * the console down.
 */
typedef struct _SMC_FAN_ALGORITHM_REQUEST {
    UCHAR Command;
    UCHAR Algorithm; /* SMC_FAN_ALGORITHM_* */
    UCHAR Reserved[14];
} SMC_FAN_ALGORITHM_REQUEST, *PSMC_FAN_ALGORITHM_REQUEST;

C_ASSERT(sizeof(SMC_FAN_ALGORITHM_REQUEST) == 16);

#define SMC_FAN_ALGORITHM_QUIET      0x01
#define SMC_FAN_ALGORITHM_FULL_SPEED 0x02

/*
 * Request of SMC_COMMAND_SET_FAN_SPEED and SMC_COMMAND_SET_FAN2_SPEED.
 * Speed with SMC_FAN_SPEED_FIXED set selects a fixed duty of
 * Speed & SMC_FAN_SPEED_PERCENT_MASK percent, at most 100; with it clear,
 * the SMC controls the fan. SMC_COMMAND_SET_FAN2_SPEED drives a second fan
 * output and exists only in some SMC builds (21010d has it, 510301
 * ignores it). The setting lasts until the console restarts.
 */
typedef struct _SMC_FAN_SPEED_REQUEST {
    UCHAR Command;
    UCHAR Speed;
    UCHAR Reserved[14];
} SMC_FAN_SPEED_REQUEST, *PSMC_FAN_SPEED_REQUEST;

C_ASSERT(sizeof(SMC_FAN_SPEED_REQUEST) == 16);

#define SMC_FAN_SPEED_AUTOMATIC    0x00
#define SMC_FAN_SPEED_FIXED        0x80
#define SMC_FAN_SPEED_PERCENT_MASK 0x7F
#define SMC_FAN_SPEED_MAX_PERCENT  100

/*
 * Request of SMC_COMMAND_SET_POWER_LED. Mode SMC_POWER_LED_AUTOMATIC
 * returns the LED to the SMC, with Option bit 0 as a one-shot request
 * whose effect is not established. Modes SMC_POWER_LED_PATTERN_FIRST to
 * SMC_POWER_LED_PATTERN_LAST select a pattern of the SMC's table, whose
 * looks are not established. The SMC ignores other modes.
 */
typedef struct _SMC_POWER_LED_REQUEST {
    UCHAR Command;
    UCHAR Mode;
    UCHAR Option;
    UCHAR Reserved[13];
} SMC_POWER_LED_REQUEST, *PSMC_POWER_LED_REQUEST;

C_ASSERT(sizeof(SMC_POWER_LED_REQUEST) == 16);

#define SMC_POWER_LED_AUTOMATIC       1
#define SMC_POWER_LED_OPTION_ONE_SHOT 0x01
#define SMC_POWER_LED_PATTERN_FIRST   2
#define SMC_POWER_LED_PATTERN_LAST    9

/*
 * Request of SMC_COMMAND_SET_RING_OF_LIGHT. With SMC_RING_OF_LIGHT_OVERRIDE
 * in Flags, the high nibble of Quadrants turns the four ring quadrants on
 * (bit set) or off; which bit lights which physical quadrant is not
 * established. Without it, control returns to the system. Keep the low
 * nibble of Quadrants zero: SMC build 21010d also reads it. The ring has
 * no colour or brightness control, and the setting lasts until the
 * console restarts.
 */
typedef struct _SMC_RING_OF_LIGHT_REQUEST {
    UCHAR Command;
    UCHAR Flags;
    UCHAR Quadrants;
    UCHAR Reserved[13];
} SMC_RING_OF_LIGHT_REQUEST, *PSMC_RING_OF_LIGHT_REQUEST;

C_ASSERT(sizeof(SMC_RING_OF_LIGHT_REQUEST) == 16);

#define SMC_RING_OF_LIGHT_OVERRIDE       0x01
#define SMC_RING_OF_LIGHT_QUADRANT_SHIFT 4
#define SMC_RING_OF_LIGHT_QUADRANT_MASK  0xF0

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

/*
 * XBOX_HARDWARE_INFO.Flags. The top four bits hold the hardware generation.
 * DEVKIT_KERNEL is set by the static initializer of development kit
 * kernels (15574, 16547) and clear in retail kernels (17559). No kernel
 * sets TEST_KIT; XBDM reports a test kit when it is set.
 */
#define XBOX_HW_FLAG_GENERATION_DETECTED      0x00000004 /* VdDetectSystemType set the generation */
#define XBOX_HW_FLAG_HOST_BRIDGE_EBEB         0x00000008
#define XBOX_HW_FLAG_DEVKIT_KERNEL            0x00000010
#define XBOX_HW_FLAG_HDD                      0x00000020
#define XBOX_HW_FLAG_KERNEL_DEBUGGER_ENABLED  0x00000080 /* enabled at boot */
#define XBOX_HW_FLAG_KERNEL_DEBUGGER_ATTACHED 0x00000100
#define XBOX_HW_FLAG_LARGE_MEMORY             0x00010000 /* 1 GB of memory */
#define XBOX_HW_FLAG_MMC_FLASH                0x00020000
#define XBOX_HW_FLAG_TEST_KIT                 0x02000000

/*
 * Hardware generation (motherboard), Flags >> XBOX_HW_GENERATION_SHIFT,
 * as XBDM names it. Other values have no known name.
 */
#define XBOX_HW_GENERATION_SHIFT      28
#define XBOX_HW_GENERATION_XENON      0
#define XBOX_HW_GENERATION_ZEPHYR     1
#define XBOX_HW_GENERATION_FALCON     2
#define XBOX_HW_GENERATION_JASPER     3
#define XBOX_HW_GENERATION_TRINITY    4
#define XBOX_HW_GENERATION_CORONA     5
#define XBOX_HW_GENERATION_WINCHESTER 6
#define XBOX_HW_GENERATION_RIDGEWAY   8

/* Platform, Flags & XBOX_HW_PLATFORM_MASK, as XBDM names it. */
#define XBOX_HW_PLATFORM_MASK       0x00000003
#define XBOX_HW_PLATFORM_ALPHA      0
#define XBOX_HW_PLATFORM_MONGREL    1
#define XBOX_HW_PLATFORM_SHIVA      2
#define XBOX_HW_PLATFORM_WATERNOOSE 3

/*
 * XBOX_HARDWARE_INFO.BootFlags, which the boot loaders and the hypervisor
 * set and pass to the kernel. SB and SD are the second and fourth stage
 * boot loaders. Bit 0x0800 is tested by SB and has no known meaning.
 */
#define XBOX_BOOT_FLAG_UNBOUND_BOOT_LOADER        0x0001 /* SB console binding is zero; SMC check skipped */
#define XBOX_BOOT_FLAG_LOCK_FUSES_BLOWN           0x0002 /* fuse row 0 bits 0-55 nonzero */
#define XBOX_BOOT_FLAG_CONSOLE_TYPE_MATCH         0x0004 /* fuse row 1 equals the SB console type */
#define XBOX_BOOT_FLAG_FUSE_VERSION_MATCH         0x0008 /* fuse row 2 count equals the SB version */
#define XBOX_BOOT_FLAG_CONSOLE_KEY_VALID          0x0010 /* fuse rows 3-6 hold a valid console key */
#define XBOX_BOOT_FLAG_DEVKIT_MEMORY              0x0080 /* memory above 512 MB */
#define XBOX_BOOT_FLAG_KD_LITE_ONLY               0x0100 /* kernel debugger lite only; setter unknown */
#define XBOX_BOOT_FLAG_SHADOW_BOOTED              0x0200 /* started by a shadow boot */
#define XBOX_BOOT_FLAG_CDROM_SYSTEM_ROOT          0x0400 /* shadow boot from CD-ROM; \SystemRoot is the CD-ROM */
#define XBOX_BOOT_FLAG_FLASH_PATCH1               0x1000 /* SD loaded the first flash patch slot */
#define XBOX_BOOT_FLAG_FLASH_PATCH2               0x2000 /* SD loaded the second flash patch slot */
#define XBOX_BOOT_FLAG_UPDATE_SEQUENCE_PENDING    0x4000 /* the patch requests the next update sequence */
#define XBOX_BOOT_FLAG_UPDATE_SEQUENCE_PROGRAMMED 0x8000 /* the update sequence fuses hold it */

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
