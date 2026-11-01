#ifndef XBOX360_XBOXKRNL_IO_H
#define XBOX360_XBOXKRNL_IO_H

#include <xbox360/xboxkrnl/ntdef.h>

XBOX360_EXTERN_C_START

/* ------------------------------------------------------------------------- */
/* File access, share, attribute and create constants                        */
/* ------------------------------------------------------------------------- */

/* FILE_SHARE_*: what other opens of a file may do. */
#define FILE_SHARE_READ   0x00000001
#define FILE_SHARE_WRITE  0x00000002
#define FILE_SHARE_DELETE 0x00000004

/* FILE_ATTRIBUTE_*. */
#define FILE_ATTRIBUTE_READONLY   0x00000001
#define FILE_ATTRIBUTE_HIDDEN     0x00000002
#define FILE_ATTRIBUTE_SYSTEM     0x00000004
#define FILE_ATTRIBUTE_DIRECTORY  0x00000010
#define FILE_ATTRIBUTE_ARCHIVE    0x00000020
#define FILE_ATTRIBUTE_DEVICE     0x00000040
#define FILE_ATTRIBUTE_NORMAL     0x00000080
#define FILE_ATTRIBUTE_TEMPORARY  0x00000100
#define FILE_ATTRIBUTE_COMPRESSED 0x00000800
#define FILE_ATTRIBUTE_ENCRYPTED  0x00004000

/* Create disposition of NtCreateFile: what to do when the file exists or does not. */
typedef enum _CREATE_DISPOSITION {
    /* Replace the file if it exists, create it otherwise. */
    FILE_SUPERSEDE = 0,
    /* Open the file; fail if it does not exist. */
    FILE_OPEN = 1,
    /* Create the file; fail if it exists. */
    FILE_CREATE = 2,
    /* Open the file, or create it if it does not exist. */
    FILE_OPEN_IF = 3,
    /* Open and truncate the file; fail if it does not exist. */
    FILE_OVERWRITE = 4,
    /* Open and truncate the file, or create it if it does not exist. */
    FILE_OVERWRITE_IF = 5
} CREATE_DISPOSITION;

/* IoStatusBlock->Information of a successful create. */
#define FILE_SUPERSEDED     0x00000000
#define FILE_OPENED         0x00000001
#define FILE_CREATED        0x00000002
#define FILE_OVERWRITTEN    0x00000003
#define FILE_EXISTS         0x00000004
#define FILE_DOES_NOT_EXIST 0x00000005

/* Create options of NtCreateFile and open options of NtOpenFile. */
#define FILE_DIRECTORY_FILE            0x00000001
#define FILE_WRITE_THROUGH             0x00000002
#define FILE_SEQUENTIAL_ONLY           0x00000004
#define FILE_NO_INTERMEDIATE_BUFFERING 0x00000008
#define FILE_SYNCHRONOUS_IO_ALERT      0x00000010
#define FILE_SYNCHRONOUS_IO_NONALERT   0x00000020
#define FILE_NON_DIRECTORY_FILE        0x00000040
#define FILE_RANDOM_ACCESS             0x00000800
#define FILE_DELETE_ON_CLOSE           0x00001000
#define FILE_OPEN_BY_FILE_ID           0x00002000
#define FILE_OPEN_FOR_BACKUP_INTENT    0x00004000

/* ByteOffset values of NtReadFile and NtWriteFile (LowPart, HighPart -1). */
#define FILE_WRITE_TO_END_OF_FILE      0xffffffff
#define FILE_USE_FILE_POINTER_POSITION 0xfffffffe

/* File system flags of FILE_FS_ATTRIBUTE_INFORMATION.FileSystemAttributes. */
#define FILE_CASE_SENSITIVE_SEARCH 0x00000001
#define FILE_CASE_PRESERVED_NAMES  0x00000002
#define FILE_UNICODE_ON_DISK       0x00000004
#define FILE_PERSISTENT_ACLS       0x00000008
#define FILE_FILE_COMPRESSION      0x00000010
#define FILE_VOLUME_IS_COMPRESSED  0x00008000
#define FILE_READ_ONLY_VOLUME      0x00080000

/* ------------------------------------------------------------------------- */
/* File information                                                          */
/* ------------------------------------------------------------------------- */

/* Classes of NtQueryInformationFile and NtSetInformationFile (Xenia's values). */
typedef enum _FILE_INFORMATION_CLASS {
    FileDirectoryInformation = 1,
    FileFullDirectoryInformation = 2,
    FileBothDirectoryInformation = 3,
    FileBasicInformation = 4,
    FileStandardInformation = 5,
    FileInternalInformation = 6,
    FileEaInformation = 7,
    FileAccessInformation = 8,
    FileNameInformation = 9,
    FileRenameInformation = 10,
    FileLinkInformation = 11,
    FileNamesInformation = 12,
    FileDispositionInformation = 13,
    FilePositionInformation = 14,
    FileFullEaInformation = 15,
    FileModeInformation = 16,
    FileAlignmentInformation = 17,
    FileAllInformation = 18,
    FileAllocationInformation = 19,
    FileEndOfFileInformation = 20,
    FileAlternateNameInformation = 21,
    FileStreamInformation = 22,
    /* Mounts a file of the volume as a partition device. */
    FileMountPartitionInformation = 23,
    FileMountPartitionsInformation = 24,
    FilePipeRemoteInformation = 25,
    FileSectorInformation = 26,
    FileXctdCompressionInformation = 27,
    FileCompressionInformation = 28,
    FileObjectIdInformation = 29,
    FileCompletionInformation = 30,
    FileMoveClusterInformation = 31,
    FileIoPriorityInformation = 32,
    FileReparsePointInformation = 33,
    FileNetworkOpenInformation = 34,
    FileAttributeTagInformation = 35,
    FileTrackingInformation = 36,
    FileMaximumInformation = 37
} FILE_INFORMATION_CLASS, *PFILE_INFORMATION_CLASS;

/* Classes of NtQueryVolumeInformationFile. */
typedef enum _FS_INFORMATION_CLASS {
    FileFsVolumeInformation = 1,
    FileFsLabelInformation = 2,
    FileFsSizeInformation = 3,
    FileFsDeviceInformation = 4,
    FileFsAttributeInformation = 5,
    FileFsControlInformation = 6,
    FileFsFullSizeInformation = 7,
    FileFsObjectIdInformation = 8,
    FileFsMaximumInformation = 9
} FS_INFORMATION_CLASS, *PFS_INFORMATION_CLASS;

/*
 * Result of NtQueryFullAttributesFile and of NtQueryInformationFile with
 * FileNetworkOpenInformation. Times are FILETIMEs.
 */
typedef struct _FILE_NETWORK_OPEN_INFORMATION {
    LARGE_INTEGER CreationTime;
    LARGE_INTEGER LastAccessTime;
    LARGE_INTEGER LastWriteTime;
    LARGE_INTEGER ChangeTime;
    LARGE_INTEGER AllocationSize;
    /* File size in bytes. */
    LARGE_INTEGER EndOfFile;
    ULONG FileAttributes;
    ULONG Reserved;
} FILE_NETWORK_OPEN_INFORMATION, *PFILE_NETWORK_OPEN_INFORMATION;

C_ASSERT(__builtin_offsetof(FILE_NETWORK_OPEN_INFORMATION, EndOfFile) == 0x28);
C_ASSERT(__builtin_offsetof(FILE_NETWORK_OPEN_INFORMATION, FileAttributes) == 0x30);
C_ASSERT(sizeof(FILE_NETWORK_OPEN_INFORMATION) == 0x38);

/*
 * An entry of NtQueryDirectoryFile. The name, FileNameLength bytes, follows
 * at 0x40. Times are FILETIMEs.
 */
typedef struct _FILE_DIRECTORY_INFORMATION {
    /* Offset of the next entry, or 0 for the last one. */
    ULONG NextEntryOffset;
    ULONG FileIndex;
    LARGE_INTEGER CreationTime;
    LARGE_INTEGER LastAccessTime;
    LARGE_INTEGER LastWriteTime;
    LARGE_INTEGER ChangeTime;
    /* File size in bytes. */
    LARGE_INTEGER EndOfFile;
    LARGE_INTEGER AllocationSize;
    ULONG FileAttributes;
    ULONG FileNameLength;
    CHAR FileName[];
} FILE_DIRECTORY_INFORMATION, *PFILE_DIRECTORY_INFORMATION;

C_ASSERT(__builtin_offsetof(FILE_DIRECTORY_INFORMATION, CreationTime) == 0x08);
C_ASSERT(__builtin_offsetof(FILE_DIRECTORY_INFORMATION, EndOfFile) == 0x28);
C_ASSERT(__builtin_offsetof(FILE_DIRECTORY_INFORMATION, FileAttributes) == 0x38);
C_ASSERT(__builtin_offsetof(FILE_DIRECTORY_INFORMATION, FileNameLength) == 0x3c);
C_ASSERT(__builtin_offsetof(FILE_DIRECTORY_INFORMATION, FileName) == 0x40);
C_ASSERT(sizeof(FILE_DIRECTORY_INFORMATION) == 0x40);

/* Information of FileBasicInformation. Times are FILETIMEs. */
typedef struct _FILE_BASIC_INFORMATION {
    LARGE_INTEGER CreationTime;
    LARGE_INTEGER LastAccessTime;
    LARGE_INTEGER LastWriteTime;
    LARGE_INTEGER ChangeTime;
    ULONG FileAttributes;
    ULONG Reserved;
} FILE_BASIC_INFORMATION, *PFILE_BASIC_INFORMATION;

C_ASSERT(__builtin_offsetof(FILE_BASIC_INFORMATION, FileAttributes) == 0x20);
C_ASSERT(sizeof(FILE_BASIC_INFORMATION) == 0x28);

/* Information of FileDispositionInformation. */
typedef struct _FILE_DISPOSITION_INFORMATION {
    /* Delete the file when its last handle is closed. */
    BOOLEAN DeleteFile;
} FILE_DISPOSITION_INFORMATION, *PFILE_DISPOSITION_INFORMATION;

C_ASSERT(sizeof(FILE_DISPOSITION_INFORMATION) == 1);

/* Information of FilePositionInformation. */
typedef struct _FILE_POSITION_INFORMATION {
    LARGE_INTEGER CurrentByteOffset;
} FILE_POSITION_INFORMATION, *PFILE_POSITION_INFORMATION;

C_ASSERT(sizeof(FILE_POSITION_INFORMATION) == 8);

/* Information of FileEndOfFileInformation. */
typedef struct _FILE_END_OF_FILE_INFORMATION {
    LARGE_INTEGER EndOfFile;
} FILE_END_OF_FILE_INFORMATION, *PFILE_END_OF_FILE_INFORMATION;

C_ASSERT(sizeof(FILE_END_OF_FILE_INFORMATION) == 8);

/* Information of FileRenameInformation. */
typedef struct _FILE_RENAME_INFORMATION {
    BOOLEAN ReplaceIfExists;
    /* Directory the name is relative to, or NULL. */
    HANDLE RootDirectory;
    OBJECT_STRING FileName;
} FILE_RENAME_INFORMATION, *PFILE_RENAME_INFORMATION;

C_ASSERT(__builtin_offsetof(FILE_RENAME_INFORMATION, RootDirectory) == 0x04);
C_ASSERT(__builtin_offsetof(FILE_RENAME_INFORMATION, FileName) == 0x08);
C_ASSERT(sizeof(FILE_RENAME_INFORMATION) == 0x10);

/* Information of FileFsSizeInformation. */
typedef struct _FILE_FS_SIZE_INFORMATION {
    LARGE_INTEGER TotalAllocationUnits;
    LARGE_INTEGER AvailableAllocationUnits;
    ULONG SectorsPerAllocationUnit;
    ULONG BytesPerSector;
} FILE_FS_SIZE_INFORMATION, *PFILE_FS_SIZE_INFORMATION;

C_ASSERT(sizeof(FILE_FS_SIZE_INFORMATION) == 0x18);

/* Information of FileFsDeviceInformation. */
typedef struct _FILE_FS_DEVICE_INFORMATION {
    /* A FILE_DEVICE_* value. */
    ULONG DeviceType;
    ULONG Characteristics;
} FILE_FS_DEVICE_INFORMATION, *PFILE_FS_DEVICE_INFORMATION;

C_ASSERT(sizeof(FILE_FS_DEVICE_INFORMATION) == 8);

/*
 * Information of FileFsAttributeInformation. The file system name,
 * FileSystemNameLength bytes, follows at 0xc.
 */
typedef struct _FILE_FS_ATTRIBUTE_INFORMATION {
    /* FILE_* file system flags. */
    ULONG FileSystemAttributes;
    LONG MaximumComponentNameLength;
    ULONG FileSystemNameLength;
    CHAR FileSystemName[];
} FILE_FS_ATTRIBUTE_INFORMATION, *PFILE_FS_ATTRIBUTE_INFORMATION;

C_ASSERT(__builtin_offsetof(FILE_FS_ATTRIBUTE_INFORMATION, FileSystemName) == 0xc);
C_ASSERT(sizeof(FILE_FS_ATTRIBUTE_INFORMATION) == 0xc);

/* Element of the segment array of NtReadFileScatter and NtWriteFileGather: a page address. */
typedef PVOID FILE_SEGMENT_ELEMENT, *PFILE_SEGMENT_ELEMENT;

/* ------------------------------------------------------------------------- */
/* Devices and device control                                                */
/* ------------------------------------------------------------------------- */

/* FILE_DEVICE_*: the type of a device. */
#define FILE_DEVICE_CD_ROM                0x02
#define FILE_DEVICE_CD_ROM_FILE_SYSTEM    0x03
#define FILE_DEVICE_CONTROLLER            0x04
#define FILE_DEVICE_DISK                  0x07
#define FILE_DEVICE_DISK_FILE_SYSTEM      0x08
#define FILE_DEVICE_FILE_SYSTEM           0x09
#define FILE_DEVICE_NETWORK_FILE_SYSTEM   0x14
#define FILE_DEVICE_MASS_STORAGE          0x2d
#define FILE_DEVICE_FLASH                 0x3c
#define FILE_DEVICE_STFS                  0x3d
#define FILE_DEVICE_SVOD                  0x40
#define FILE_DEVICE_MMC_FLASH             0x48
#define FILE_DEVICE_MMC_FLASH_PARTITION   0x49

/* Device control codes: device type, required access, function and buffering method. */
#define CTL_CODE(DeviceType, Function, Method, Access)                                  \
    (((ULONG)(DeviceType) << 16) | ((ULONG)(Access) << 14) | ((ULONG)(Function) << 2) | \
     (ULONG)(Method))

#define METHOD_BUFFERED   0
#define METHOD_IN_DIRECT  1
#define METHOD_OUT_DIRECT 2
#define METHOD_NEITHER    3

#define FILE_ANY_ACCESS   0
#define FILE_READ_ACCESS  0x0001
#define FILE_WRITE_ACCESS 0x0002

/* The sector count and the sector size: a DISK_GEOMETRY. */
#define IOCTL_DISK_GET_DRIVE_GEOMETRY \
    CTL_CODE(FILE_DEVICE_DISK, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)
/* The start offset and the length in bytes: a PARTITION_INFORMATION. */
#define IOCTL_DISK_GET_PARTITION_INFO \
    CTL_CODE(FILE_DEVICE_DISK, 1, METHOD_BUFFERED, FILE_READ_ACCESS)
/* A SCSI command described by a SCSI_PASS_THROUGH_DIRECT input. */
#define IOCTL_SCSI_PASS_THROUGH_DIRECT 0x0004D014
/* The 0x14-byte flash geometry; words 0 and 2 multiply to the write unit of the device. */
#define IOCTL_FLASH_GEOMETRY 0x003C0028
/* The authentication state of the optical drive. */
#define IOCTL_ODD_AUTH_STATE 0x000240CC

/* Output of IOCTL_DISK_GET_DRIVE_GEOMETRY. */
typedef struct _DISK_GEOMETRY {
    ULONG Sectors;
    ULONG BytesPerSector;
} DISK_GEOMETRY, *PDISK_GEOMETRY;

C_ASSERT(sizeof(DISK_GEOMETRY) == 8);

/* Output of IOCTL_DISK_GET_PARTITION_INFO. */
typedef struct _PARTITION_INFORMATION {
    /* Byte offset of the partition on the disk. */
    LARGE_INTEGER StartingOffset;
    /* Length of the partition in bytes. */
    LARGE_INTEGER PartitionLength;
} PARTITION_INFORMATION, *PPARTITION_INFORMATION;

C_ASSERT(sizeof(PARTITION_INFORMATION) == 0x10);

/*
 * Input of IOCTL_SCSI_PASS_THROUGH_DIRECT. The device transfers data
 * directly to or from DataBuffer.
 */
typedef struct _SCSI_PASS_THROUGH_DIRECT {
    /* Size of the structure in bytes. */
    USHORT Length;
    UCHAR Reserved02[5];
    /* Meaning unknown. */
    UCHAR Unknown07;
    /* Bytes to transfer. */
    ULONG DataTransferLength;
    ULONG Reserved0C;
    /* Address of the transfer buffer. */
    PVOID DataBuffer;
    /* Command descriptor block. */
    UCHAR Cdb[16];
} SCSI_PASS_THROUGH_DIRECT, *PSCSI_PASS_THROUGH_DIRECT;

C_ASSERT(__builtin_offsetof(SCSI_PASS_THROUGH_DIRECT, Unknown07) == 0x07);
C_ASSERT(__builtin_offsetof(SCSI_PASS_THROUGH_DIRECT, DataTransferLength) == 0x08);
C_ASSERT(__builtin_offsetof(SCSI_PASS_THROUGH_DIRECT, DataBuffer) == 0x10);
C_ASSERT(__builtin_offsetof(SCSI_PASS_THROUGH_DIRECT, Cdb) == 0x14);
C_ASSERT(sizeof(SCSI_PASS_THROUGH_DIRECT) == 0x24);

/* The standard data of a SCSI INQUIRY command. */
typedef struct _SCSI_INQUIRY_DATA {
    /* Peripheral qualifier and device type. */
    UCHAR DeviceType;
    /* Bit 7: removable medium. */
    UCHAR Removable;
    UCHAR Version;
    /* Response data format in the low four bits. */
    UCHAR ResponseFormat;
    /* Bytes that follow this one. */
    UCHAR AdditionalLength;
    UCHAR Flags[3];
    /* ASCII vendor identification. */
    CHAR VendorId[8];
    /* ASCII product identification. */
    CHAR ProductId[16];
    /* ASCII product revision level. */
    CHAR ProductRevision[4];
} SCSI_INQUIRY_DATA, *PSCSI_INQUIRY_DATA;

C_ASSERT(__builtin_offsetof(SCSI_INQUIRY_DATA, VendorId) == 0x08);
C_ASSERT(__builtin_offsetof(SCSI_INQUIRY_DATA, ProductId) == 0x10);
C_ASSERT(__builtin_offsetof(SCSI_INQUIRY_DATA, ProductRevision) == 0x20);
C_ASSERT(sizeof(SCSI_INQUIRY_DATA) == 0x24);

/* ------------------------------------------------------------------------- */
/* I/O request packets, devices and drivers                                  */
/* ------------------------------------------------------------------------- */

/* IRP_MJ_*: the major function of an I/O request. */
#define IRP_MJ_CREATE                   0x00
#define IRP_MJ_CLOSE                    0x01
#define IRP_MJ_READ                     0x02
#define IRP_MJ_WRITE                    0x03
#define IRP_MJ_QUERY_INFORMATION        0x04
#define IRP_MJ_SET_INFORMATION          0x05
#define IRP_MJ_FLUSH_BUFFERS            0x06
#define IRP_MJ_QUERY_VOLUME_INFORMATION 0x07
#define IRP_MJ_DIRECTORY_CONTROL        0x08
#define IRP_MJ_DEVICE_CONTROL           0x09
#define IRP_MJ_CLEANUP                  0x0a
#define IRP_MJ_MAXIMUM_FUNCTION         0x0a

/* Flags of IO_STACK_LOCATION. */
/* The buffer of a read or write is the transfer address, not an offset into Irp->UserBuffer. */
#define SL_FSCACHE_REQUEST 0x01

/* SL_* control bits of IO_STACK_LOCATION. */
#define SL_PENDING_RETURNED  0x01
#define SL_INVOKE_ON_CANCEL  0x20
#define SL_INVOKE_ON_SUCCESS 0x40
#define SL_INVOKE_ON_ERROR   0x80

/* DO_* flags of DEVICE_OBJECT. */
#define DO_DIRECT_IO           0x00000004
#define DO_DEVICE_HAS_NAME     0x00000008
/* Set at creation. The driver clears it once the device is ready. */
#define DO_DEVICE_INITIALIZING 0x00000010

/* Priority boosts of IoCompleteRequest. */
#define IO_NO_INCREMENT     0
#define IO_CD_ROM_INCREMENT 1
#define IO_DISK_INCREMENT   1

struct _DEVICE_OBJECT;
struct _IRP;

typedef NTSTATUS (*PIO_COMPLETION_ROUTINE)(
    struct _DEVICE_OBJECT *DeviceObject,
    struct _IRP *Irp,
    PVOID Context);

/*
 * The part of a request for one driver. The locations of a request are
 * contiguous; the next driver's location is the one before the current one.
 * The parameters are 4-byte aligned: ByteOffset is at offset 12.
 */
#pragma pack(push, 4)
typedef struct _IO_STACK_LOCATION {
    /* An IRP_MJ_* value. */
    UCHAR MajorFunction;
    UCHAR MinorFunction;
    UCHAR Flags;
    UCHAR Control;
    union {
        struct {
            ULONG Length;
            union {
                /* Offset into Irp->UserBuffer, without SL_FSCACHE_REQUEST. */
                ULONG BufferOffset;
                /* With SL_FSCACHE_REQUEST. */
                PVOID CacheBuffer;
            };
            LARGE_INTEGER ByteOffset;
        } Read;
        struct {
            ULONG Length;
            union {
                ULONG BufferOffset;
                PVOID CacheBuffer;
            };
            LARGE_INTEGER ByteOffset;
        } Write;
        struct {
            ULONG OutputBufferLength;
            PVOID InputBuffer;
            ULONG InputBufferLength;
            ULONG IoControlCode;
        } DeviceIoControl;
        struct {
            ULONG Length;
            FILE_INFORMATION_CLASS FileInformationClass;
        } QueryFile;
        struct {
            ULONG Length;
            FS_INFORMATION_CLASS FsInformationClass;
        } QueryVolume;
        struct {
            PVOID Argument1;
            PVOID Argument2;
            PVOID Argument3;
            PVOID Argument4;
        } Others;
    } Parameters;
    struct _DEVICE_OBJECT *DeviceObject;
    struct _FILE_OBJECT *FileObject;
    PIO_COMPLETION_ROUTINE CompletionRoutine;
    PVOID Context;
} IO_STACK_LOCATION, *PIO_STACK_LOCATION;
#pragma pack(pop)

C_ASSERT(__builtin_offsetof(IO_STACK_LOCATION, Control) == 3);
C_ASSERT(__builtin_offsetof(IO_STACK_LOCATION, Parameters) == 4);
C_ASSERT(__builtin_offsetof(IO_STACK_LOCATION, Parameters.Read.ByteOffset) == 12);
C_ASSERT(__builtin_offsetof(IO_STACK_LOCATION, DeviceObject) == 0x14);
C_ASSERT(__builtin_offsetof(IO_STACK_LOCATION, CompletionRoutine) == 0x1c);
C_ASSERT(__builtin_offsetof(IO_STACK_LOCATION, Context) == 0x20);
C_ASSERT(sizeof(IO_STACK_LOCATION) == 0x24);

/* An I/O request packet. The kernel allocates it with its stack locations. */
typedef struct _IRP {
    SHORT Type;
    USHORT Size;
    ULONG Flags;
    LIST_ENTRY ThreadListEntry;
    IO_STATUS_BLOCK IoStatus;
    CHAR StackCount;
    /* Index of the current stack location. */
    CHAR CurrentLocation;
    BOOLEAN PendingReturned;
    BOOLEAN Cancel;
    PVOID UserBuffer;
    PIO_STATUS_BLOCK UserIosb;
    PKEVENT UserEvent;
    union {
        struct {
            PIO_APC_ROUTINE UserApcRoutine;
            PVOID UserApcContext;
        } AsynchronousParameters;
        LARGE_INTEGER AllocationSize;
    } Overlay;
    union {
        struct {
            /* KDEVICE_QUEUE_ENTRY (ke.h), or the key of an I/O completion packet. */
            ULONG DeviceQueueEntry[4];
            /* Length of the buffer locked by IoLockUserBuffer. */
            ULONG LockedBufferLength;
            struct _ETHREAD *Thread;
            LIST_ENTRY ListEntry;
            PIO_STACK_LOCATION CurrentStackLocation;
            struct _FILE_OBJECT *OriginalFileObject;
        } Overlay;
        /* KAPC (ke.h). */
        ULONG Apc[10];
    } Tail;
    /* Called with the cancel spin lock held; it releases the lock. */
    VOID (*CancelRoutine)(struct _DEVICE_OBJECT *DeviceObject, struct _IRP *Irp);
    ULONG Reserved1;
} IRP, *PIRP;

C_ASSERT(__builtin_offsetof(IRP, IoStatus) == 0x10);
C_ASSERT(__builtin_offsetof(IRP, CurrentLocation) == 0x19);
C_ASSERT(__builtin_offsetof(IRP, UserBuffer) == 0x1c);
C_ASSERT(__builtin_offsetof(IRP, UserIosb) == 0x20);
C_ASSERT(__builtin_offsetof(IRP, Overlay) == 0x28);
C_ASSERT(__builtin_offsetof(IRP, Tail.Overlay.LockedBufferLength) == 0x40);
C_ASSERT(__builtin_offsetof(IRP, Tail.Overlay.CurrentStackLocation) == 0x50);
C_ASSERT(__builtin_offsetof(IRP, Tail.Overlay.OriginalFileObject) == 0x54);
C_ASSERT(sizeof(IRP) == 0x60);

/* A driver routine for a request. */
typedef NTSTATUS (*PDRIVER_DISPATCH)(struct _DEVICE_OBJECT *DeviceObject, PIRP Irp);
/* The start I/O routine of a driver. */
typedef VOID (*PDRIVER_STARTIO)(struct _DEVICE_OBJECT *DeviceObject, PIRP Irp);
typedef VOID (*PDRIVER_DELETEDEVICE)(struct _DEVICE_OBJECT *DeviceObject);
typedef NTSTATUS (*PDRIVER_DISMOUNTVOLUME)(struct _DEVICE_OBJECT *DeviceObject);

/* The routines of a driver. */
typedef struct _DRIVER_OBJECT {
    PDRIVER_STARTIO DriverStartIo;
    PDRIVER_DELETEDEVICE DriverDeleteDevice;
    PDRIVER_DISMOUNTVOLUME DriverDismountVolume;
    /* The routine of each IRP_MJ_* function. */
    PDRIVER_DISPATCH MajorFunction[IRP_MJ_MAXIMUM_FUNCTION + 1];
} DRIVER_OBJECT, *PDRIVER_OBJECT;

C_ASSERT(__builtin_offsetof(DRIVER_OBJECT, MajorFunction) == 0xc);
C_ASSERT(sizeof(DRIVER_OBJECT) == 0x38);

/* A device object. The kernel allocates it and its extension. */
typedef struct _DEVICE_OBJECT {
    SHORT Type;
    USHORT Size;
    LONG ReferenceCount;
    PDRIVER_OBJECT DriverObject;
    /* File system device mounted on this device, or the device itself. */
    struct _DEVICE_OBJECT *MountedOrSelfDevice;
    PIRP CurrentIrp;
    /* DO_* flags. */
    ULONG Flags;
    PVOID DeviceExtension;
    UCHAR DeviceType;
    UCHAR StartIoFlags;
    /* Stack locations a request to the device needs. */
    CHAR StackSize;
    BOOLEAN DeletePending;
    ULONG SectorSize;
    /* Mask of the address bits a transfer buffer must have clear. */
    ULONG AlignmentRequirement;
    /* KDEVICE_QUEUE (ke.h). */
    ULONG DeviceQueue[4];
    KEVENT DeviceLock;
    ULONG StartIoCount;
    ULONG StartIoKey;
} DEVICE_OBJECT, *PDEVICE_OBJECT;

C_ASSERT(__builtin_offsetof(DEVICE_OBJECT, DriverObject) == 8);
C_ASSERT(__builtin_offsetof(DEVICE_OBJECT, CurrentIrp) == 0x10);
C_ASSERT(__builtin_offsetof(DEVICE_OBJECT, Flags) == 0x14);
C_ASSERT(__builtin_offsetof(DEVICE_OBJECT, DeviceExtension) == 0x18);
C_ASSERT(__builtin_offsetof(DEVICE_OBJECT, StackSize) == 0x1e);
C_ASSERT(__builtin_offsetof(DEVICE_OBJECT, AlignmentRequirement) == 0x24);
C_ASSERT(__builtin_offsetof(DEVICE_OBJECT, DeviceQueue) == 0x28);
C_ASSERT(__builtin_offsetof(DEVICE_OBJECT, DeviceLock) == 0x38);
C_ASSERT(sizeof(DEVICE_OBJECT) == 0x50);

/* ------------------------------------------------------------------------- */
/* File system cache                                                         */
/* ------------------------------------------------------------------------- */

/** Returns the element count of a file system cache. ProcessType is a KPROCESS_TYPE value. */
XBOXKRNL_IMPORT(0x020) ULONG FscGetCacheElementCount(ULONG ProcessType);

/** Sets the element count of a file system cache. ProcessType is a KPROCESS_TYPE value. */
XBOXKRNL_IMPORT(0x021) NTSTATUS FscSetCacheElementCount(ULONG ProcessType, ULONG ElementCount);

/* ------------------------------------------------------------------------- */
/* I/O manager                                                               */
/* ------------------------------------------------------------------------- */

/** Acquires the device object lock. Returns the IRQL that IoReleaseDeviceObjectLock restores. */
XBOXKRNL_IMPORT(0x02E) KIRQL IoAcquireDeviceObjectLock(VOID);

/** Allocates an I/O request packet with StackSize stack locations. Returns it or NULL. */
XBOXKRNL_IMPORT(0x02F) PIRP IoAllocateIrp(CHAR StackSize);

/** Completes an I/O request packet. */
XBOXKRNL_IMPORT(0x035) VOID IoCompleteRequest(PIRP Irp, CHAR PriorityBoost);

/** Creates a device object for a driver. The device object address is stored at DeviceObject. */
XBOXKRNL_IMPORT(0x037) NTSTATUS IoCreateDevice(
    PDRIVER_OBJECT DriverObject,
    ULONG DeviceExtensionSize,
    POBJECT_STRING DeviceName,
    ULONG DeviceType,
    ULONG Attributes,
    PDEVICE_OBJECT *DeviceObject);

/** Object type of device objects. */
XBOXKRNL_IMPORT(0x03A) extern struct _OBJECT_TYPE *__imp_IoDeviceObjectType;
#define IoDeviceObjectType (*__imp_IoDeviceObjectType)

/** Dismounts the file system volume on a device object. */
XBOXKRNL_IMPORT(0x03B) NTSTATUS IoDismountVolume(PDEVICE_OBJECT DeviceObject);

/** Dismounts the file system volume that an open file belongs to. */
XBOXKRNL_IMPORT(0x03C) NTSTATUS IoDismountVolumeByFileHandle(HANDLE FileHandle);

/** Completes an I/O request packet with an invalid device request status. */
XBOXKRNL_IMPORT(0x041) NTSTATUS IoInvalidDeviceRequest(PDEVICE_OBJECT DeviceObject, PIRP Irp);

/** Releases the device object lock and restores OldIrql. */
XBOXKRNL_IMPORT(0x044) VOID IoReleaseDeviceObjectLock(KIRQL OldIrql);

/** Starts the next queued I/O request packet of a device. */
XBOXKRNL_IMPORT(0x048) VOID IoStartNextPacket(PDEVICE_OBJECT DeviceObject);

/** Starts or queues an I/O request packet on a device. */
XBOXKRNL_IMPORT(0x04A) VOID IoStartPacket(PDEVICE_OBJECT DeviceObject, PIRP Irp, PULONG Key);

/** Sends a device I/O control request to a device object and waits for it. */
XBOXKRNL_IMPORT(0x04B) NTSTATUS IoSynchronousDeviceIoControlRequest(
    ULONG IoControlCode,
    PDEVICE_OBJECT DeviceObject,
    PVOID InputBuffer,
    ULONG InputBufferLength,
    PVOID OutputBuffer,
    ULONG OutputBufferLength,
    PULONG ReturnedOutputBufferLength);

/** Sends a read or write request (an IRP_MJ_* value) to a device object and waits for it. */
XBOXKRNL_IMPORT(0x04C) NTSTATUS IoSynchronousFsdRequest(
    ULONG MajorFunction,
    PDEVICE_OBJECT DeviceObject,
    PVOID Buffer,
    ULONG Length,
    PLARGE_INTEGER StartingOffset);

/* ------------------------------------------------------------------------- */
/* File system calls                                                         */
/* ------------------------------------------------------------------------- */

/** Creates or opens a file. */
XBOXKRNL_IMPORT(0x0D2) NTSTATUS NtCreateFile(
    PHANDLE FileHandle,
    ACCESS_MASK DesiredAccess,
    POBJECT_ATTRIBUTES ObjectAttributes,
    PIO_STATUS_BLOCK IoStatusBlock,
    PLARGE_INTEGER AllocationSize,
    ULONG FileAttributes,
    ULONG ShareAccess,
    CREATE_DISPOSITION CreateDisposition,
    ULONG CreateOptions);

/** Deletes a file. */
XBOXKRNL_IMPORT(0x0D8) NTSTATUS NtDeleteFile(POBJECT_ATTRIBUTES ObjectAttributes);

/** Sends a device I/O control request through a file handle. */
XBOXKRNL_IMPORT(0x0D9) NTSTATUS NtDeviceIoControlFile(
    HANDLE FileHandle,
    HANDLE Event,
    PIO_APC_ROUTINE ApcRoutine,
    PVOID ApcContext,
    PIO_STATUS_BLOCK IoStatusBlock,
    ULONG IoControlCode,
    PVOID InputBuffer,
    ULONG InputBufferLength,
    PVOID OutputBuffer,
    ULONG OutputBufferLength);

/** Writes buffered data of a file to the device. */
XBOXKRNL_IMPORT(0x0DB) NTSTATUS NtFlushBuffersFile(
    HANDLE FileHandle,
    PIO_STATUS_BLOCK IoStatusBlock);

/** Opens a file. */
XBOXKRNL_IMPORT(0x0DF) NTSTATUS NtOpenFile(
    PHANDLE FileHandle,
    ACCESS_MASK DesiredAccess,
    POBJECT_ATTRIBUTES ObjectAttributes,
    PIO_STATUS_BLOCK IoStatusBlock,
    ULONG ShareAccess,
    ULONG OpenOptions);

/** Reads directory entries. FileName is an optional wildcard pattern. */
XBOXKRNL_IMPORT(0x0E4) NTSTATUS NtQueryDirectoryFile(
    HANDLE FileHandle,
    HANDLE Event,
    PIO_APC_ROUTINE ApcRoutine,
    PVOID ApcContext,
    PIO_STATUS_BLOCK IoStatusBlock,
    PVOID FileInformation,
    ULONG Length,
    POBJECT_STRING FileName,
    BOOLEAN RestartScan);

/** Reads the network open information of a file by name. */
XBOXKRNL_IMPORT(0x0E7) NTSTATUS NtQueryFullAttributesFile(
    POBJECT_ATTRIBUTES ObjectAttributes,
    PFILE_NETWORK_OPEN_INFORMATION FileInformation);

/** Reads information of class FileInformationClass about an open file. */
XBOXKRNL_IMPORT(0x0E8) NTSTATUS NtQueryInformationFile(
    HANDLE FileHandle,
    PIO_STATUS_BLOCK IoStatusBlock,
    PVOID FileInformation,
    ULONG Length,
    FILE_INFORMATION_CLASS FileInformationClass);

/** Reads information of class FsInformationClass about a volume. */
XBOXKRNL_IMPORT(0x0EF) NTSTATUS NtQueryVolumeInformationFile(
    HANDLE FileHandle,
    PIO_STATUS_BLOCK IoStatusBlock,
    PVOID FsInformation,
    ULONG Length,
    FS_INFORMATION_CLASS FsInformationClass);

/**
 * Reads from a file. A NULL ByteOffset reads at the current position. A NULL
 * Event signals the file handle.
 */
XBOXKRNL_IMPORT(0x0F0) NTSTATUS NtReadFile(
    HANDLE FileHandle,
    HANDLE Event,
    PIO_APC_ROUTINE ApcRoutine,
    PVOID ApcContext,
    PIO_STATUS_BLOCK IoStatusBlock,
    PVOID Buffer,
    ULONG Length,
    PLARGE_INTEGER ByteOffset);

/** Reads from a file into a list of page-sized segments. */
XBOXKRNL_IMPORT(0x0F1) NTSTATUS NtReadFileScatter(
    HANDLE FileHandle,
    HANDLE Event,
    PIO_APC_ROUTINE ApcRoutine,
    PVOID ApcContext,
    PIO_STATUS_BLOCK IoStatusBlock,
    PFILE_SEGMENT_ELEMENT SegmentArray,
    ULONG Length,
    PLARGE_INTEGER ByteOffset);

/** Writes information of class FileInformationClass about an open file. */
XBOXKRNL_IMPORT(0x0F7) NTSTATUS NtSetInformationFile(
    HANDLE FileHandle,
    PIO_STATUS_BLOCK IoStatusBlock,
    PVOID FileInformation,
    ULONG Length,
    FILE_INFORMATION_CLASS FileInformationClass);

/**
 * Writes to a file. A NULL ByteOffset writes at the current position. A NULL
 * Event signals the file handle.
 */
XBOXKRNL_IMPORT(0x0FF) NTSTATUS NtWriteFile(
    HANDLE FileHandle,
    HANDLE Event,
    PIO_APC_ROUTINE ApcRoutine,
    PVOID ApcContext,
    PIO_STATUS_BLOCK IoStatusBlock,
    PVOID Buffer,
    ULONG Length,
    PLARGE_INTEGER ByteOffset);

/** Writes to a file from a list of page-sized segments. */
XBOXKRNL_IMPORT(0x100) NTSTATUS NtWriteFileGather(
    HANDLE FileHandle,
    HANDLE Event,
    PIO_APC_ROUTINE ApcRoutine,
    PVOID ApcContext,
    PIO_STATUS_BLOCK IoStatusBlock,
    PFILE_SEGMENT_ELEMENT SegmentArray,
    ULONG Length,
    PLARGE_INTEGER ByteOffset);

XBOX360_EXTERN_C_END

#endif
