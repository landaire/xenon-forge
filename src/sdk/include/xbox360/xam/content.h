/*
 * XAM content packages: create, open, enumerate, copy and delete packages
 * on storage devices.
 */

#ifndef XBOX360_XAM_CONTENT_H
#define XBOX360_XAM_CONTENT_H

#include <xbox360/xam/xamdef.h>

XBOX360_EXTERN_C_START

/* Kind of a content package, with Xenia's values. */
typedef enum _XCONTENT_TYPE {
    XCONTENTTYPE_SAVEDGAME = 0x00000001,
    XCONTENTTYPE_MARKETPLACE = 0x00000002,
    XCONTENTTYPE_PUBLISHER = 0x00000003,
    XCONTENTTYPE_XBOX360TITLE = 0x00001000,
    XCONTENTTYPE_IPTVPAUSEBUFFER = 0x00002000,
    XCONTENTTYPE_XNACOMMUNITY = 0x00003000,
    XCONTENTTYPE_INSTALLEDGAME = 0x00004000,
    XCONTENTTYPE_XBOXTITLE = 0x00005000,
    XCONTENTTYPE_SOCIALTITLE = 0x00006000,
    XCONTENTTYPE_GAMESONDEMAND = 0x00007000,
    XCONTENTTYPE_STORAGEPACK = 0x00008000,
    XCONTENTTYPE_AVATARITEM = 0x00009000,
    XCONTENTTYPE_PROFILE = 0x00010000,
    XCONTENTTYPE_GAMERPICTURE = 0x00020000,
    XCONTENTTYPE_THEME = 0x00030000,
    XCONTENTTYPE_CACHEFILE = 0x00040000,
    XCONTENTTYPE_STORAGEDOWNLOAD = 0x00050000,
    XCONTENTTYPE_XBOXSAVEDGAME = 0x00060000,
    XCONTENTTYPE_XBOXDOWNLOAD = 0x00070000,
    XCONTENTTYPE_GAMEDEMO = 0x00080000,
    XCONTENTTYPE_VIDEO = 0x00090000,
    XCONTENTTYPE_GAMETITLE = 0x000A0000,
    XCONTENTTYPE_INSTALLER = 0x000B0000,
    XCONTENTTYPE_GAMETRAILER = 0x000C0000,
    XCONTENTTYPE_ARCADE = 0x000D0000,
    XCONTENTTYPE_XNA = 0x000E0000,
    XCONTENTTYPE_LICENSESTORE = 0x000F0000,
    XCONTENTTYPE_MOVIE = 0x00100000,
    XCONTENTTYPE_TV = 0x00200000,
    XCONTENTTYPE_MUSICVIDEO = 0x00300000,
    XCONTENTTYPE_GAMEVIDEO = 0x00400000,
    XCONTENTTYPE_PODCASTVIDEO = 0x00500000,
    XCONTENTTYPE_VIRALVIDEO = 0x00600000,
    XCONTENTTYPE_COMMUNITYGAME = 0x02000000,
} XCONTENT_TYPE;

/* Length of the display name, in UTF-16 units including the NUL. */
#define XCONTENT_MAX_DISPLAYNAME_LENGTH 128
/* Length of the file name in bytes. A name of this length has no NUL. */
#define XCONTENT_MAX_FILENAME_LENGTH 42

/* What XamContentCreate did. */
#define XCONTENT_CREATED_NEW      1
#define XCONTENT_OPENED_EXISTING  2

/* The device, kind and names of a package. */
typedef struct _XCONTENT_DATA {
    DWORD DeviceID;
    DWORD dwContentType;
    WCHAR szDisplayName[XCONTENT_MAX_DISPLAYNAME_LENGTH];
    CHAR szFileName[XCONTENT_MAX_FILENAME_LENGTH];
    BYTE Padding[2];
} XCONTENT_DATA, *PXCONTENT_DATA;

C_ASSERT(sizeof(XCONTENT_DATA) == 0x134);
C_ASSERT(__builtin_offsetof(XCONTENT_DATA, szFileName) == 0x108);

/**
 * Creates or opens a content package and maps it to the drive RootName.
 * Flags holds the creation disposition in its low four bits. The disposition taken
 * (XCONTENT_CREATED_NEW or XCONTENT_OPENED_EXISTING) is stored at Disposition when not
 * NULL. Returns a Win32 error code, or ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x258) DWORD XamContentCreate(
    DWORD UserIndex,
    PCSTR RootName,
    const XCONTENT_DATA *ContentData,
    DWORD Flags,
    PDWORD Disposition,
    PDWORD LicenseMask,
    PXOVERLAPPED Overlapped);

/** Closes the content drive RootName. Returns a Win32 error code, or ERROR_IO_PENDING with an Overlapped. */
XAM_IMPORT(0x25A) DWORD XamContentClose(PCSTR RootName, PXOVERLAPPED Overlapped);

/**
 * Deletes a content package.
 * Returns a Win32 error code, or ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x25B) DWORD XamContentDelete(DWORD UserIndex, const XCONTENT_DATA *ContentData, PXOVERLAPPED Overlapped);

/**
 * Creates an enumerator of the storage devices accepting ContentTypes, returning
 * ItemsPerCall 0x50-byte device records per XamEnumerate. Returns a Win32 error code.
 */
XAM_IMPORT(0x25D) DWORD XamContentCreateDeviceEnumerator(
    DWORD ContentTypes,
    DWORD Flags,
    DWORD ItemsPerCall,
    PDWORD BufferSize,
    PHANDLE Handle);

/** Fills the 0x50-byte device record of DeviceId. Returns a Win32 error code. */
XAM_IMPORT(0x25E) DWORD XamContentGetDeviceData(DWORD DeviceId, PBYTE DeviceData);

/**
 * Commits the writes to the content drive RootName. The 2.0.6717 dashboard flushes
 * DASHUSER after it writes DashPreferences. Returns a Win32 error code.
 */
XAM_IMPORT(0x267) DWORD XamContentFlush(PCSTR RootName, PXOVERLAPPED Overlapped);

/**
 * Copies the package of the 0x200-byte aggregate record Content to the device DeviceId
 * points to, owned by the XUID NewOwner points to. The 2.0.6717 dashboard passes NULL
 * for DeviceId when the destination is the package's device and NULL for NewOwner
 * when the owner does not change.
 * XAM calls Progress with a 64-bit byte count in r4, a reason in r8 and Context in
 * the stack slot at 0x50 of its caller. The dashboard's routine (0x9225BF00) adds the
 * growth of the count to its total, takes the count to start again at 0 on reason 1, and
 * returns 0. The dashboard passes 0 for Unknown, whose meaning is not known. Returns a
 * Win32 error code, or ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x26C) DWORD XamContentCopyInternal(
    const XUID *NewOwner,
    const DWORD *DeviceId,
    PCBYTE Content,
    PCVOID Progress,
    PVOID Context,
    DWORD Unknown,
    PXOVERLAPPED Overlapped);

/** Moves a package; the arguments are those of XamContentCopyInternal. */
XAM_IMPORT(0x26D) DWORD XamContentMoveInternal(
    const XUID *NewOwner,
    const DWORD *DeviceId,
    PCBYTE Content,
    PCVOID Progress,
    PVOID Context,
    DWORD Unknown,
    PXOVERLAPPED Overlapped);

/**
 * Reads the header of the package of the 0x200-byte aggregate record Content into
 * Buffer, 0x93D6 bytes in the 2.0.6717 dashboard. Returns a Win32 error code.
 */
XAM_IMPORT(0x26E) DWORD XamContentGetMetaDataInternal(PCBYTE Content, PBYTE Buffer, PXOVERLAPPED Overlapped);

/**
 * Deletes the package of the 0x200-byte aggregate record Content. Returns a Win32 error
 * code, or ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x270) DWORD XamContentDeleteInternal(PCBYTE Content, PXOVERLAPPED Overlapped);

/**
 * Opens the package of the 0x200-byte aggregate record Content and maps it to the drive
 * RootName, as XamContentCreate. The 2.0.6717 dashboard opens existing packages
 * (Flags 3) and passes 0 for Reserved and Size. Returns a Win32 error code.
 */
XAM_IMPORT(0x271) DWORD XamContentCreateInternal(
    PCSTR RootName,
    PCBYTE Content,
    DWORD Flags,
    PDWORD Disposition,
    PDWORD LicenseMask,
    DWORD Reserved,
    ULONGLONG Size,
    PXOVERLAPPED Overlapped);

/**
 * Launches the NUL-terminated executable path Image inside the package of the 0x200-byte
 * aggregate record Content. Returns 0 on success; the system then ends the caller.
 */
XAM_IMPORT(0x273) DWORD XamContentLaunchImageInternal(PCBYTE Content, PCSTR Image);

/**
 * Writes the path of the package of the 0x200-byte aggregate record Content to Path as a
 * NUL-terminated string of at most Capacity bytes. The 2.0.6717 dashboard passes 0x104
 * and 0 for the reserved arguments. Returns a Win32 error code.
 */
XAM_IMPORT(0x276) DWORD XamContentResolveInternal(
    PCBYTE Content,
    PSTR Path,
    DWORD Capacity,
    DWORD Reserved1,
    DWORD Reserved2,
    DWORD Reserved3);

/**
 * Writes the 0x24-byte attribute block of the package of the 0x200-byte aggregate record
 * Content to Attributes; +0x4 holds the creation FILETIME. Returns a Win32 error code,
 * or ERROR_IO_PENDING with an Overlapped.
 */
XAM_IMPORT(0x277) DWORD XamContentGetAttributesInternal(PCBYTE Content, PBYTE Attributes, PXOVERLAPPED Overlapped);

/**
 * Opens an enumeration of the packages of ContentType and TitleId on DeviceId (0: every
 * device). The 2.0.6717 dashboard passes 0 as the first argument; its Memory area passes a
 * device id, other callers 0. Returns a Win32 error code.
 */
XAM_IMPORT(0x279) DWORD XamContentAggregateCreateEnumerator(
    DWORD Reserved,
    DWORD DeviceId,
    DWORD ContentType,
    DWORD TitleId,
    PHANDLE Handle);

/**
 * Stores the id of the default content device at DeviceId. Returns a Win32 error code,
 * ERROR_FUNCTION_FAILED when no device is the default. The 2.0.6717 dashboard passes the
 * device word of a 0x200-byte content record and does not read the result.
 */
XAM_IMPORT(0x284) DWORD XamContentGetDefaultDevice(PDWORD DeviceId);

XBOX360_EXTERN_C_END

#endif
