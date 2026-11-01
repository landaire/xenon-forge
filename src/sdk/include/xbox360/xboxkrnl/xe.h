/*
 * Kernel cryptography (XeCrypt) and security key (XeKeys) routines.
 */

#ifndef XBOX360_XBOXKRNL_XE_H
#define XBOX360_XBOXKRNL_XE_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* DES key schedule of XeCryptDesKey: 16 rounds of two words. */
typedef struct _XECRYPT_DES_STATE {
    ULONG KeyTable[32];
} XECRYPT_DES_STATE, *PXECRYPT_DES_STATE;

C_ASSERT(sizeof(XECRYPT_DES_STATE) == 0x80);

/* Key id of XeKeysGetKey: the 12-digit console serial number. */
#define XEKEYS_KEY_CONSOLE_SERIAL_NUMBER 0x14

/* Console identifier: 5 bytes. */
#define XE_CONSOLE_ID_SIZE 5

typedef struct _XE_CONSOLE_ID {
    UCHAR Bytes[XE_CONSOLE_ID_SIZE];
} XE_CONSOLE_ID, *PXE_CONSOLE_ID;

C_ASSERT(sizeof(XE_CONSOLE_ID) == 5);

/** Sets the parity bit of each byte of a DES key of InputSize bytes. */
XBOXKRNL_IMPORT(0x174) VOID XeCryptDesParity(PCBYTE Input, ULONG InputSize, PBYTE Output);

/** Expands an 8-byte DES key into DesState. */
XBOXKRNL_IMPORT(0x175) VOID XeCryptDesKey(PXECRYPT_DES_STATE DesState, PCBYTE Key);

/** Encrypts (Encrypt nonzero) or decrypts one 8-byte block in ECB mode. */
XBOXKRNL_IMPORT(0x176) VOID XeCryptDesEcb(const XECRYPT_DES_STATE *DesState, PCBYTE Input, PBYTE Output, BOOL Encrypt);

/**
 * HMAC-SHA1 of the concatenation of up to three buffers under Key; the
 * first OutputSize bytes of the digest go to Output. The dashboard passes
 * null buffers of size 0 for the unused ones.
 */
XBOXKRNL_IMPORT(0x182) VOID XeCryptHmacSha(
    PCBYTE Key,
    ULONG KeySize,
    PCBYTE Input1,
    ULONG Input1Size,
    PCBYTE Input2,
    ULONG Input2Size,
    PCBYTE Input3,
    ULONG Input3Size,
    PBYTE Output,
    ULONG OutputSize
);

/**
 * MD5 of the concatenation of up to three buffers; the first OutputSize
 * bytes of the digest go to Output. The dashboard passes null buffers of
 * size 0 for the unused ones.
 */
XBOXKRNL_IMPORT(0x187) VOID XeCryptMd5(
    PCBYTE Input1,
    ULONG Input1Size,
    PCBYTE Input2,
    ULONG Input2Size,
    PCBYTE Input3,
    ULONG Input3Size,
    PBYTE Output,
    ULONG OutputSize
);

/** Fills a buffer with random bytes. */
XBOXKRNL_IMPORT(0x18A) VOID XeCryptRandom(PBYTE Buffer, ULONG Length);

/**
 * Writes a boot loader image to flash. A nonzero StartOffset value on
 * return also indicates failure.
 */
XBOXKRNL_IMPORT(0x23D) NTSTATUS XeKeysSaveBootLoader(PULONG StartOffset, PULONG Length, PVOID Buffer);

/** Stores the security key status flags. */
XBOXKRNL_IMPORT(0x23F) VOID XeKeysGetStatus(PULONG Status);

/**
 * Copies the key KeyId into Buffer; Length holds the buffer size on entry
 * and the key size on return. Key 0x14 is the 12-digit console serial number.
 */
XBOXKRNL_IMPORT(0x244) NTSTATUS XeKeysGetKey(ULONG KeyId, PVOID Buffer, PULONG Length);

/**
 * Stores the 5-byte console ID and, when ConsoleIdString is not null, its
 * 12-character text form.
 */
XBOXKRNL_IMPORT(0x246) NTSTATUS XeKeysGetConsoleID(PXE_CONSOLE_ID ConsoleId, PCHAR ConsoleIdString);

XBOX360_EXTERN_C_END

#endif
