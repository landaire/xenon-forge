/*
 * XAM controller input: state, capabilities, vibration and keystrokes.
 */

#ifndef XBOX360_XAM_INPUT_H
#define XBOX360_XAM_INPUT_H

#include <xbox360/xam/xamdef.h>

XBOX360_EXTERN_C_START

/* Devices a controller query covers. */
#define XINPUT_FLAG_GAMEPAD 0x00000001

/* XamInputGetKeystrokeEx: every device class, and a keystroke of any user. */
#define XINPUT_FLAG_ANYDEVICE 0x000000FF
#define XINPUT_FLAG_ANYUSER   0x40000000

/* Device type and subtype. */
#define XINPUT_DEVTYPE_GAMEPAD    0x01
#define XINPUT_DEVSUBTYPE_GAMEPAD 0x01

/* Pressed buttons of a gamepad. */
#define XINPUT_GAMEPAD_DPAD_UP        0x0001
#define XINPUT_GAMEPAD_DPAD_DOWN      0x0002
#define XINPUT_GAMEPAD_DPAD_LEFT      0x0004
#define XINPUT_GAMEPAD_DPAD_RIGHT     0x0008
#define XINPUT_GAMEPAD_START          0x0010
#define XINPUT_GAMEPAD_BACK           0x0020
#define XINPUT_GAMEPAD_LEFT_THUMB     0x0040
#define XINPUT_GAMEPAD_RIGHT_THUMB    0x0080
#define XINPUT_GAMEPAD_LEFT_SHOULDER  0x0100
#define XINPUT_GAMEPAD_RIGHT_SHOULDER 0x0200
#define XINPUT_GAMEPAD_GUIDE          0x0400
#define XINPUT_GAMEPAD_A              0x1000
#define XINPUT_GAMEPAD_B              0x2000
#define XINPUT_GAMEPAD_X              0x4000
#define XINPUT_GAMEPAD_Y              0x8000

/* Capability flags. */
#define XINPUT_CAPS_FFB_SUPPORTED   0x0001
#define XINPUT_CAPS_WIRELESS        0x0002
#define XINPUT_CAPS_VOICE_SUPPORTED 0x0004
#define XINPUT_CAPS_PMD_SUPPORTED   0x0008
#define XINPUT_CAPS_NO_NAVIGATION   0x0010

/* Keystroke flags. */
#define XINPUT_KEYSTROKE_KEYDOWN 0x0001
#define XINPUT_KEYSTROKE_KEYUP   0x0002
#define XINPUT_KEYSTROKE_REPEAT  0x0004

typedef struct _XINPUT_GAMEPAD {
    WORD wButtons;
    BYTE bLeftTrigger;
    BYTE bRightTrigger;
    SHORT sThumbLX;
    SHORT sThumbLY;
    SHORT sThumbRX;
    SHORT sThumbRY;
} XINPUT_GAMEPAD, *PXINPUT_GAMEPAD;

C_ASSERT(sizeof(XINPUT_GAMEPAD) == 12);

typedef struct _XINPUT_STATE {
    /* Changes whenever the controller state changes. */
    DWORD dwPacketNumber;
    XINPUT_GAMEPAD Gamepad;
} XINPUT_STATE, *PXINPUT_STATE;

C_ASSERT(sizeof(XINPUT_STATE) == 16);

/* Motor speeds, 0 to 65535. */
typedef struct _XINPUT_VIBRATION {
    WORD wLeftMotorSpeed;
    WORD wRightMotorSpeed;
} XINPUT_VIBRATION, *PXINPUT_VIBRATION;

C_ASSERT(sizeof(XINPUT_VIBRATION) == 4);

/* The gamepad and vibration fields hold the ranges the device supports. */
typedef struct _XINPUT_CAPABILITIES {
    BYTE Type;
    BYTE SubType;
    WORD Flags;
    XINPUT_GAMEPAD Gamepad;
    XINPUT_VIBRATION Vibration;
} XINPUT_CAPABILITIES, *PXINPUT_CAPABILITIES;

C_ASSERT(sizeof(XINPUT_CAPABILITIES) == 20);
C_ASSERT(__builtin_offsetof(XINPUT_CAPABILITIES, Vibration) == 16);

/* One key transition. Controller buttons arrive as virtual keys 0x5800 and up. */
typedef struct _XINPUT_KEYSTROKE {
    WORD VirtualKey;
    WCHAR Unicode;
    WORD Flags;
    BYTE UserIndex;
    BYTE HidCode;
} XINPUT_KEYSTROKE, *PXINPUT_KEYSTROKE;

C_ASSERT(sizeof(XINPUT_KEYSTROKE) == 8);

/*
 * Controller keys as XamInputGetKeystrokeEx reports them in VirtualKey.
 * Keyboard keys use Windows virtual key codes, not listed here.
 */
typedef enum _XINPUT_PAD_KEY {
    VK_PAD_A = 0x5800,
    VK_PAD_B = 0x5801,
    VK_PAD_X = 0x5802,
    VK_PAD_Y = 0x5803,
    VK_PAD_RSHOULDER = 0x5804,
    VK_PAD_LSHOULDER = 0x5805,
    VK_PAD_LTRIGGER = 0x5806,
    VK_PAD_RTRIGGER = 0x5807,
    VK_PAD_DPAD_UP = 0x5810,
    VK_PAD_DPAD_DOWN = 0x5811,
    VK_PAD_DPAD_LEFT = 0x5812,
    VK_PAD_DPAD_RIGHT = 0x5813,
    VK_PAD_START = 0x5814,
    VK_PAD_BACK = 0x5815,
    VK_PAD_LTHUMB_UP = 0x5820,
    VK_PAD_LTHUMB_DOWN = 0x5821,
    VK_PAD_LTHUMB_RIGHT = 0x5822,
    VK_PAD_LTHUMB_LEFT = 0x5823,
} XINPUT_PAD_KEY;

/**
 * Reads the capabilities of the controller of a user slot.
 * Returns a Win32 error code: ERROR_DEVICE_NOT_CONNECTED without a controller.
 */
XAM_IMPORT(0x190) DWORD XamInputGetCapabilities(DWORD UserIndex, DWORD Flags, PXINPUT_CAPABILITIES Capabilities);

/**
 * Reads the state of the controller of a user slot.
 * Returns a Win32 error code: ERROR_DEVICE_NOT_CONNECTED without a controller.
 */
XAM_IMPORT(0x191) DWORD XamInputGetState(DWORD UserIndex, DWORD Flags, PXINPUT_STATE State);

/**
 * Sets the motor speeds of the controller of a user slot. Xenia ignores Flags.
 * Returns a Win32 error code.
 */
XAM_IMPORT(0x192) DWORD XamInputSetState(DWORD UserIndex, DWORD Flags, const XINPUT_VIBRATION *Vibration);

/**
 * Takes the next queued key transition. UserIndex points to the user slot, or
 * with XINPUT_FLAG_ANYUSER in Flags receives the slot of the keystroke.
 * Returns a Win32 error code; nonzero when no keystroke is queued.
 */
XAM_IMPORT(0x198) DWORD XamInputGetKeystrokeEx(PDWORD UserIndex, DWORD Flags, PXINPUT_KEYSTROKE Keystroke);

XBOX360_EXTERN_C_END

#endif
