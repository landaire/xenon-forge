/*
 * XAM users and profiles: sign-in, XUIDs, profile settings, profiles on
 * storage, achievements and tiles.
 */

#ifndef XBOX360_XAM_USER_H
#define XBOX360_XAM_USER_H

#include <xbox360/xam/xamdef.h>

XBOX360_EXTERN_C_START

/* A user slot, or a choice the export makes. UserIndex arguments take these values. */
typedef enum _XUSER_INDEX {
    XUSER_INDEX_0 = 0,
    XUSER_INDEX_1 = 1,
    XUSER_INDEX_2 = 2,
    XUSER_INDEX_3 = 3,
    /* No user, for operations on behalf of the title itself. */
    XUSER_INDEX_NONE = 0xFE,
    /* Any signed-in user or connected controller, as the export decides. */
    XUSER_INDEX_ANY = 0xFF,
} XUSER_INDEX;

/* Sign-in state of a user slot. */
typedef enum _XUSER_SIGNIN_STATE {
    eXUserSigninState_NotSignedIn = 0,
    eXUserSigninState_SignedInLocally = 1,
    eXUserSigninState_SignedInToLive = 2,
} XUSER_SIGNIN_STATE;

/* Data type of a NUL-terminated UTF-16 string in XUSER_DATA. */
#define XUSER_DATA_TYPE_UNICODE 4

/*
 * A typed setting value. The type byte is at +0 and the value at +8; a
 * string is its byte length at +8 and its address at +0xC.
 */
typedef struct _XUSER_DATA {
    BYTE type;
    BYTE Reserved[7];
    union {
        LONG nData;
        struct {
            DWORD cbData;
            PWSTR pwszData;
        } string;
        DWORD Value[2];
    };
} XUSER_DATA, *PXUSER_DATA;

C_ASSERT(sizeof(XUSER_DATA) == 0x10);
C_ASSERT(__builtin_offsetof(XUSER_DATA, string) == 8);

/*
 * One profile of XamProfileEnumerate. The first 8 bytes are the offline
 * XUID; the account data after it is not decoded here.
 */
typedef struct _XPROFILE_ENUM_RESULT {
    XUID xuidOffline;
    BYTE Account[0x180];
} XPROFILE_ENUM_RESULT, *PXPROFILE_ENUM_RESULT;

C_ASSERT(sizeof(XPROFILE_ENUM_RESULT) == 0x188);

/**
 * Stores the XUID of a user slot at Xuid. Flags selects the XUID kinds accepted
 * (the 2.0.6717 dashboard passes 1, 3, 4 and 7). Fails when the slot has no such XUID.
 */
XAM_IMPORT(0x20A) HRESULT XamUserGetXUID(DWORD UserIndex, DWORD Flags, PXUID Xuid);

/**
 * Signs in the users of the four XUIDs at Xuids, one per user slot; zero XUIDs sign
 * those slots out. XAM copies the XUIDs during the call. Returns ERROR_IO_PENDING (a
 * success value as an HRESULT) with an Overlapped.
 */
XAM_IMPORT(0x20B) HRESULT XamUserLogon(const XUID *Xuids, DWORD Flags, PXOVERLAPPED Overlapped);

/**
 * Writes the gamertag of a signed-in user slot to Buffer as NUL-terminated UTF-16,
 * at most Length units. The 2.0.6717 dashboard passes 16 units.
 */
XAM_IMPORT(0x20C) HRESULT XamUserGetGamerTag(DWORD UserIndex, PWSTR Buffer, DWORD Length);

/** Returns the sign-in state of a user slot. */
XAM_IMPORT(0x210) XUSER_SIGNIN_STATE XamUserGetSigninState(DWORD UserIndex);

/**
 * Stores at UserIndex the user slot the profile Xuid is signed in on. Flags
 * selects the XUID kinds compared, as for XamUserGetXUID; the 2.0.6717 dashboard's
 * Memory area passes 1. Fails when no slot has the profile.
 */
XAM_IMPORT(0x211) HRESULT XamUserGetIndexFromXUID(XUID Xuid, DWORD Flags, PDWORD UserIndex);

/**
 * Reads the profile settings SettingIds (SettingIdCount entries) of TitleId for UserIndex,
 * or of the XuidCount profiles at Xuids. Without Buffer, stores the size needed at
 * BufferSize. Returns a Win32 error code, or ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x219) DWORD XamUserReadProfileSettings(
    DWORD TitleId,
    DWORD UserIndex,
    DWORD XuidCount,
    const XUID *Xuids,
    DWORD SettingIdCount,
    const DWORD *SettingIds,
    PDWORD BufferSize,
    PBYTE Buffer,
    PXOVERLAPPED Overlapped);

/** Returns the Xbox LIVE membership type of UserIndex in the low byte. */
XAM_IMPORT(0x21B) DWORD XamUserGetMembershipType(DWORD UserIndex);

/**
 * Processes the queued Xbox LIVE logons. Returns ERROR_IO_PENDING (a success value as an
 * HRESULT) with an Overlapped.
 */
XAM_IMPORT(0x220) HRESULT XamUserFlushLogonQueue(PXOVERLAPPED Overlapped);

/** Returns nonzero when the profile of a user slot has an Xbox LIVE account. */
XAM_IMPORT(0x221) BOOLEAN XamUserIsOnlineEnabled(DWORD UserIndex);

/**
 * Starts the creation of an offline profile on the device DeviceId points to and stores
 * the creation handle. Account is a zeroed 0x17C-byte XAMACCOUNTINFO with the
 * big-endian UTF-16 gamertag (16 units) at +8; Block is a zeroed 0x8F0-byte block. The
 * 2.0.6717 dashboard passes 1 for Flags and 0 for Reserved0, Reserved1 and Reserved2.
 * XAM uses the buffers until the creation completes.
 */
XAM_IMPORT(0x230) HRESULT XamProfileCreate(
    DWORD Flags,
    const DWORD *DeviceId,
    DWORD Reserved0,
    PBYTE Account,
    PBYTE Block,
    DWORD Reserved1,
    DWORD Reserved2,
    PHANDLE Handle);

/**
 * Opens an enumeration of the profiles on DeviceId (0 for all devices) and stores its handle.
 * Close the handle with NtClose. Returns a Win32 error code.
 */
XAM_IMPORT(0x231) DWORD XamProfileCreateEnumerator(DWORD DeviceId, PHANDLE Handle);

/**
 * Stores the next profile of an enumeration at Result. Returns a Win32 error code;
 * nonzero once no profile remains.
 */
XAM_IMPORT(0x232) DWORD XamProfileEnumerate(HANDLE Handle, DWORD Flags, PXPROFILE_ENUM_RESULT Result, PXOVERLAPPED Overlapped);

/**
 * Deletes the profile Xuid. The 2.0.6717 dashboard passes 0 for Unknown, whose meaning
 * is not known, and calls it without an overlapped block. Returns an HRESULT.
 */
XAM_IMPORT(0x233) HRESULT XamProfileDelete(DWORD Unknown, XUID Xuid);

/**
 * Stores the XUID of the profile a XamProfileCreate handle creates. Returns
 * E_PENDING while the creation runs.
 */
XAM_IMPORT(0x234) HRESULT XamProfileGetCreationStatus(HANDLE Handle, PXUID Xuid);

/**
 * Mounts the dashboard settings of the profile Xuid as the drive MountName
 * (NUL-terminated ASCII, such as DASHUSER). The 2.0.6717 dashboard passes 0 and NULL
 * for Flags and ContentData. Returns a Win32 error code.
 */
XAM_IMPORT(0x237) DWORD XamProfileOpen(XUID Xuid, PCSTR MountName, DWORD Flags, PBYTE ContentData);

/** Unmounts a drive that XamProfileOpen mounted. Returns a Win32 error code. */
XAM_IMPORT(0x238) DWORD XamProfileClose(PCSTR MountName);

/**
 * Reads the 0x17C-byte account record of the profile Xuid from its package on
 * DeviceId into Account; the gamertag is NUL-terminated UTF-16 at +0x08. Returns an
 * HRESULT.
 */
XAM_IMPORT(0x23A) HRESULT XamProfileLoadAccountInfo(DWORD DeviceId, XUID Xuid, PBYTE Account);

/**
 * Opens an enumeration of the achievements of TitleId for UserIndex or Xuid (0 for the
 * user). Stores the buffer size for Count records at BufferSize. Returns a Win32 error code.
 */
XAM_IMPORT(0x2EE) DWORD XamUserCreateAchievementEnumerator(
    DWORD TitleId,
    DWORD UserIndex,
    XUID Xuid,
    DWORD DetailFlags,
    DWORD StartingIndex,
    DWORD Count,
    PDWORD BufferSize,
    PHANDLE Handle);

/**
 * Reads a tile PNG into Buffer. Without Buffer, stores the size needed at BufferSize.
 * Returns a Win32 error code, or ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x2EF) DWORD XamReadTile(
    DWORD TileType,
    DWORD TitleId,
    ULONGLONG TileId,
    DWORD UserIndex,
    PBYTE Buffer,
    PDWORD BufferSize,
    PXOVERLAPPED Overlapped);

/**
 * Writes the gamer picture of TitleId with the key words KeyHigh and KeyLow to the
 * profile signed in on UserIndex. The 2.0.6717 dashboard passes 0 for Reserved.
 * Returns a Win32 error code, ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x2F0) DWORD XamWriteGamerTile(
    DWORD UserIndex,
    DWORD TitleId,
    DWORD KeyHigh,
    DWORD KeyLow,
    DWORD Reserved,
    PXOVERLAPPED Overlapped);

/**
 * Opens an enumeration of the titles UserIndex or Xuid (0 for the user) has played.
 * Stores the buffer size for Count records at BufferSize. Returns a Win32 error code.
 */
XAM_IMPORT(0x2F3) DWORD XamUserCreateTitlesPlayedEnumerator(
    DWORD TitleId,
    DWORD UserIndex,
    XUID Xuid,
    DWORD StartingIndex,
    DWORD Count,
    PDWORD BufferSize,
    PHANDLE Handle);

/**
 * Reads a tile PNG into Buffer of *BufferSize bytes and stores the length at BufferSize.
 * Key is the picture key setting for tile type 0xC; the 2.0.6717 dashboard passes
 * HasKey 0 with it. Returns a Win32 error code, or ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x2FC) DWORD XamReadTileEx(
    DWORD TileType,
    DWORD TitleId,
    ULONGLONG TileId,
    DWORD UserIndex,
    DWORD HasKey,
    const XUSER_DATA *Key,
    PBYTE Buffer,
    PDWORD BufferSize,
    PXOVERLAPPED Overlapped);

/**
 * Writes the time of the local FILETIME Time for UserIndex as NUL-terminated UTF-16,
 * at most Length units. The 2.0.6717 dashboard does not read the result.
 */
XAM_IMPORT(0x1E7) VOID XamFormatTimeString(DWORD UserIndex, ULONGLONG Time, PWSTR Buffer, DWORD Length);

/**
 * Writes the date of the local FILETIME Time for UserIndex as NUL-terminated UTF-16,
 * at most Length units. The 2.0.6717 dashboard does not read the result.
 */
XAM_IMPORT(0x1E8) VOID XamFormatDateString(DWORD UserIndex, ULONGLONG Time, PWSTR Buffer, DWORD Length);

XBOX360_EXTERN_C_END

#endif
