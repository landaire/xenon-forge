/*
 * XAM system UI: sign-in, keyboard, message box, device selector, gamer
 * cards, marketplace and other system screens.
 */

#ifndef XBOX360_XAM_UI_H
#define XBOX360_XAM_UI_H

#include <xbox360/xam/xamdef.h>

XBOX360_EXTERN_C_START

/* Icon of a message box, in the low four bits of the XamShowMessageBoxUI flags. */
typedef enum _XMB_ICON {
    XMB_NOICON = 0,
    XMB_ERRORICON = 1,
    XMB_WARNINGICON = 2,
    XMB_ALERTICON = 3,
} XMB_ICON;

/* The user enters a four-key passcode, returned in MESSAGEBOX_RESULT. The box takes no buttons. */
#define XMB_PASSCODEMODE       0x00010000
/* XAM compares the entered passcode with the one the result block holds. */
#define XMB_VERIFYPASSCODEMODE 0x00020000

/* The index of the chosen button, or the four virtual key codes of a passcode. */
typedef union _MESSAGEBOX_RESULT {
    DWORD dwButtonPressed;
    WORD rgwPasscode[4];
} MESSAGEBOX_RESULT, *PMESSAGEBOX_RESULT;

C_ASSERT(sizeof(MESSAGEBOX_RESULT) == 8);

/**
 * Opens the sign-in UI with Panes user panes. Returns a Win32 error code; the UI does
 * not report when it closes.
 */
XAM_IMPORT(0x2BC) DWORD XamShowSigninUI(DWORD Panes, DWORD Flags);

/**
 * XamShowSigninUI for the controller UserIndex. Returns a Win32 error code; the UI
 * does not report when it closes.
 */
XAM_IMPORT(0x2BE) DWORD XamShowSigninUIp(DWORD UserIndex, DWORD Panes, DWORD Flags);

/**
 * Shows the virtual keyboard. DefaultText, Title and Description are NUL-terminated
 * UTF-16 strings. Completes through Overlapped and writes the NUL-terminated text to
 * Buffer, at most Length units. Returns ERROR_IO_PENDING when the keyboard is shown,
 * or another Win32 error code.
 */
XAM_IMPORT(0x2C1) DWORD XamShowKeyboardUI(
    DWORD UserIndex,
    DWORD Flags,
    PCWSTR DefaultText,
    PCWSTR Title,
    PCWSTR Description,
    PWSTR Buffer,
    DWORD Length,
    PXOVERLAPPED Overlapped);

/**
 * Opens the gamer card of the profile signed in on UserIndex. Returns a Win32 error code.
 * An index above 3 reaches RtlRip.
 */
XAM_IMPORT(0x2C4) DWORD XamShowGamerCardUI(DWORD UserIndex);

/**
 * Opens the achievements of TitleId for UserIndex. Returns a Win32 error code. An index
 * above 3 reaches RtlRip.
 */
XAM_IMPORT(0x2C5) DWORD XamShowAchievementsUI(DWORD UserIndex, DWORD TitleId);

/**
 * Shows a message box with up to three buttons. Title, Text and each entry of Buttons are
 * NUL-terminated UTF-16 strings. Completes through Overlapped and stores the result at
 * Result; dwButtonPressed is the chosen button index. Returns ERROR_IO_PENDING when the
 * box is shown, or another Win32 error code.
 */
XAM_IMPORT(0x2CA) DWORD XamShowMessageBoxUI(
    DWORD UserIndex,
    PCWSTR Title,
    PCWSTR Text,
    DWORD ButtonCount,
    const PCWSTR *Buttons,
    DWORD FocusButton,
    DWORD Flags,
    PMESSAGEBOX_RESULT Result,
    PXOVERLAPPED Overlapped);

/**
 * Shows the storage device selector for content of type ContentType (0 for any) that
 * needs BytesRequested free bytes (0 for any). DeviceId holds the device selected
 * first and receives the chosen device id. Completes through Overlapped. Returns
 * ERROR_IO_PENDING when the selector is shown, or another Win32 error code.
 */
XAM_IMPORT(0x2CB) DWORD XamShowDeviceSelectorUI(
    DWORD UserIndex,
    DWORD ContentType,
    DWORD Flags,
    ULONGLONG BytesRequested,
    PDWORD DeviceId,
    PXOVERLAPPED Overlapped);

/**
 * Opens the profile creation UI for UserIndex. In system software 2.0.21256.0 the export
 * passes Unknown on to XamShowCreateProfileUIEx, which stores it in the request and
 * selects one of two modes by it (1, 0xB and 0xE against any other value); the 2.0.6717.0
 * export takes only UserIndex and stores 0 there. Returns a Win32 error code. An index
 * above 3 reaches RtlRip.
 */
XAM_IMPORT(0x2CF) DWORD XamShowCreateProfileUI(DWORD UserIndex, DWORD Unknown);

/**
 * Opens the played games list of UserIndex. The 2.0.6717 dashboard passes 0 for
 * Unknown. Returns a Win32 error code. An index above 3 reaches RtlRip.
 */
XAM_IMPORT(0x2D0) DWORD XamShowGamesUI(DWORD UserIndex, DWORD Unknown);

/**
 * Opens the Xbox LIVE signup UI for UserIndex. The 2.0.6717 dashboard passes 1 to
 * recover an existing account and 0 to join. Returns a Win32 error code.
 */
XAM_IMPORT(0x2D1) DWORD XamShowLiveSignupUI(DWORD UserIndex, DWORD Mode);

/**
 * Opens the gamer card of Xuid for UserIndex. A nonzero Flags sets bit 0x80 of the
 * request; the 2.0.6717 dashboard passes 0. Returns a Win32 error code. An index above 3
 * reaches RtlRip.
 */
XAM_IMPORT(0x2D5) DWORD XamShowGamerCardUIForXUID(DWORD UserIndex, XUID Xuid, DWORD Flags);

/**
 * Opens the details of AchievementId of TitleId for UserIndex. Returns a Win32 error
 * code. An index above 3 reaches RtlRip.
 */
XAM_IMPORT(0x2E0) DWORD XamShowAchievementDetailsUI(DWORD UserIndex, DWORD TitleId, DWORD AchievementId);

/**
 * In system software 2.0.6717.0 opens the theme picker for UserIndex, copying 0x200 bytes
 * from InitialSelection when it is not NULL. In 2.0.21256.0 it opens no UI: it writes
 * 0x144 bytes of the selection to <drive>:\ThematicSkin and sends notification
 * 0x8000000F, and a NULL InitialSelection reaches RtlRip. The purpose of Unknown is
 * not known.
 */
XAM_IMPORT(0x2E1) DWORD XamShowPersonalizationUI(DWORD UserIndex, PCBYTE InitialSelection, DWORD Unknown);

/**
 * Opens the marketplace at Entry for UserIndex, with the offer OfferId and the title
 * TitleId. The 2.0.6717 dashboard passes 0xFFFFFFFF, 0xFFFFFFFF, 0 and 0 for the four
 * unknown arguments. Returns a Win32 error code.
 */
XAM_IMPORT(0x2E8) DWORD XamShowMarketplaceUIEx(
    DWORD UserIndex,
    DWORD Entry,
    ULONGLONG OfferId,
    DWORD Unknown1,
    DWORD Unknown2,
    DWORD Unknown3,
    DWORD Unknown4,
    DWORD TitleId);

/**
 * Opens the Xbox LIVE upsell UI for UserIndex. XamShowLiveUpsellUI (0x2D7) takes no
 * arguments and calls this export with 0. Returns a Win32 error code.
 */
XAM_IMPORT(0x300) DWORD XamShowLiveUpsellUIEx(DWORD UserIndex);

/**
 * Opens the join session in progress UI for the player Xuid for UserIndex. Returns a
 * Win32 error code. An index above 3 reaches RtlRip.
 */
XAM_IMPORT(0x301) DWORD XamShowJoinSessionInProgressUI(DWORD UserIndex, XUID Xuid);

XBOX360_EXTERN_C_END

#endif
