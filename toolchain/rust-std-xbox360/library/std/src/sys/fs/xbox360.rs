//! Filesystem for the Xbox 360, over the kernel NT file API.
//!
//! Paths are kernel object names, such as
//! `\Device\Harddisk0\Partition1\DEVKIT\file.txt`, or drive paths, such as
//! `game:\file.txt`, which name the object `\??\game:\file.txt`. The kernel
//! rejects `/` in names, so `/` separators are passed as `\`. Names are ANSI.
//!
//! Files are opened for synchronous I/O, so the kernel keeps the file
//! position. Writes of a file opened for appending go to the end of the file.
//!
//! The kernel has no file locks or links. FATX keeps no read-only attribute:
//! a request to set it fails.

use crate::ffi::OsString;
use crate::fmt;
use crate::fs::TryLockError;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::path::{Path, PathBuf};
use crate::sync::Arc;
pub use crate::sys::fs::common::{Dir, copy, exists, remove_dir_all};
use crate::sys::path::has_drive;
use crate::sys::time::SystemTime;
use crate::sys::{unsupported, unsupported_err};

use xbox360_sys::status::KernelError;
use xbox360_sys::xam::{self, DuplicateOptions};
use xbox360_sys::xboxkrnl::{
    self, AccessMask, AnsiString, CreateDisposition, CreateOptions, FileAttributes,
    FileInformationClass, Handle, IoStatusBlock, NtStatus, ObjectAttributeFlags, ObjectAttributes,
    ShareAccess,
};

/// `byte_offset` of `NtWriteFile` that writes at the end of the file
/// (`FILE_WRITE_TO_END_OF_FILE` in both halves).
const END_OF_FILE: u64 = u64::MAX;

/// Other opens may read, write and delete a file that is open, as on Windows.
const SHARE_ALL: ShareAccess = ShareAccess::READ.union(ShareAccess::WRITE).union(ShareAccess::DELETE);

/// Room for one directory entry: the header and a name. FATX names have at
/// most 42 bytes.
const DIRECTORY_BUFFER: usize = 0x200;

fn cvt(status: NtStatus) -> io::Result<()> {
    if status == NtStatus::SUCCESS { Ok(()) } else { Err(io::Error::from_raw_os_error(status.0 as i32)) }
}

/// The kernel object name of `path`: `/` becomes `\`, and a drive path gets
/// the `\??\` prefix.
fn object_name(path: &Path) -> io::Result<Vec<u8>> {
    let bytes = path.as_os_str().as_encoded_bytes();
    if bytes.contains(&0) {
        return Err(io::const_error!(io::ErrorKind::InvalidFilename, "path contains a NUL byte"));
    }
    let mut name = Vec::with_capacity(bytes.len() + 4);
    if has_drive(path) {
        name.extend_from_slice(b"\\??\\");
    }
    name.extend(bytes.iter().map(|&b| if b == b'/' { b'\\' } else { b }));
    if name.len() > usize::from(u16::MAX) {
        return Err(io::const_error!(io::ErrorKind::InvalidFilename, "path is too long"));
    }
    Ok(name)
}

/// Calls `f` with the object attributes of `path`.
fn with_object_attributes<T>(
    path: &Path,
    f: impl FnOnce(&ObjectAttributes) -> io::Result<T>,
) -> io::Result<T> {
    let name = object_name(path)?;
    // `object_name` limits the length to `u16::MAX`.
    let name = AnsiString::new(&name).unwrap();
    let attributes = ObjectAttributes {
        root_directory: Handle::NULL,
        object_name: &name,
        attributes: ObjectAttributeFlags::CASE_INSENSITIVE,
    };
    f(&attributes)
}

/// `FILE_NETWORK_OPEN_INFORMATION`. FATX rejects `FileStandardInformation`
/// and `FileBasicInformation` queries; this class has the same fields.
#[repr(C)]
#[derive(Default)]
struct NetworkOpenInformation {
    creation_time: u64,
    last_access_time: u64,
    last_write_time: u64,
    change_time: u64,
    allocation_size: u64,
    end_of_file: u64,
    attributes: u32,
    _pad: u32,
}

/// `FILE_BASIC_INFORMATION`. Zero leaves a time or the attributes unchanged.
#[repr(C)]
#[derive(Default)]
struct BasicInformation {
    creation_time: u64,
    last_access_time: u64,
    last_write_time: u64,
    change_time: u64,
    attributes: u32,
    _pad: u32,
}

/// `FILE_RENAME_INFORMATION`.
#[repr(C)]
struct RenameInformation {
    replace_if_exists: u8,
    root_directory: Handle,
    file_name: AnsiString,
}

/// The fixed part of `FILE_DIRECTORY_INFORMATION`; the name follows it.
#[repr(C)]
struct DirectoryInformation {
    next_entry_offset: u32,
    file_index: u32,
    creation_time: u64,
    last_access_time: u64,
    last_write_time: u64,
    change_time: u64,
    end_of_file: u64,
    allocation_size: u64,
    attributes: u32,
    file_name_length: u32,
}

const _: () = {
    assert!(size_of::<NetworkOpenInformation>() == 0x38);
    assert!(size_of::<BasicInformation>() == 0x28);
    assert!(size_of::<RenameInformation>() == 0x10);
    assert!(size_of::<DirectoryInformation>() == 0x40);
};

pub struct File {
    handle: Handle,
    append: bool,
}

// The handle is a kernel object id that any thread may use.
unsafe impl Send for File {}
unsafe impl Sync for File {}

#[derive(Clone)]
pub struct FileAttr {
    attributes: FileAttributes,
    creation_time: u64,
    last_access_time: u64,
    last_write_time: u64,
    size: u64,
}

pub struct ReadDir {
    handle: File,
    root: Arc<PathBuf>,
    buffer: Box<[u64; DIRECTORY_BUFFER / 8]>,
    /// Offset of the next entry in `buffer`, or `None` when the buffer has
    /// no more entries.
    next: Option<usize>,
    done: bool,
}

pub struct DirEntry {
    root: Arc<PathBuf>,
    name: OsString,
    attr: FileAttr,
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FileTimes {
    accessed: Option<SystemTime>,
    modified: Option<SystemTime>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FilePermissions {
    attributes: FileAttributes,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FileType {
    is_dir: bool,
}

#[derive(Debug)]
pub struct DirBuilder {}

/// A FILETIME the kernel reported, or an error for 0, which the file system
/// reports for a time it does not keep.
fn filetime(time: u64) -> io::Result<SystemTime> {
    if time == 0 {
        Err(io::const_error!(io::ErrorKind::Unsupported, "the file system does not keep this time"))
    } else {
        Ok(SystemTime::from_filetime(time))
    }
}

impl FileAttr {
    fn from_network_open(info: &NetworkOpenInformation) -> FileAttr {
        FileAttr {
            attributes: FileAttributes::from_bits_retain(info.attributes),
            creation_time: info.creation_time,
            last_access_time: info.last_access_time,
            last_write_time: info.last_write_time,
            size: info.end_of_file,
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn perm(&self) -> FilePermissions {
        FilePermissions { attributes: self.attributes }
    }
    pub fn file_type(&self) -> FileType {
        FileType { is_dir: self.attributes.contains(FileAttributes::DIRECTORY) }
    }
    pub fn modified(&self) -> io::Result<SystemTime> {
        filetime(self.last_write_time)
    }
    pub fn accessed(&self) -> io::Result<SystemTime> {
        filetime(self.last_access_time)
    }
    pub fn created(&self) -> io::Result<SystemTime> {
        filetime(self.creation_time)
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.attributes.contains(FileAttributes::READONLY)
    }
    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            self.attributes.insert(FileAttributes::READONLY);
        } else {
            self.attributes.remove(FileAttributes::READONLY);
        }
    }

    /// The attributes to set. Zero would leave them unchanged, so a file
    /// without attributes gets `NORMAL`, as Windows gives it.
    fn to_set(&self) -> FileAttributes {
        if self.attributes.is_empty() { FileAttributes::NORMAL } else { self.attributes }
    }
}

impl FileTimes {
    pub fn set_accessed(&mut self, t: SystemTime) {
        self.accessed = Some(t);
    }
    pub fn set_modified(&mut self, t: SystemTime) {
        self.modified = Some(t);
    }
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    pub fn is_file(&self) -> bool {
        !self.is_dir
    }
    pub fn is_symlink(&self) -> bool {
        false
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ReadDir").field(&*self.root).finish()
    }
}

impl ReadDir {
    /// Fills the buffer with the next entries. `false` at the end.
    fn fill(&mut self) -> io::Result<bool> {
        let mut iosb = IoStatusBlock::default();
        // SAFETY: `self.handle` is an open directory handle and the buffer is
        // writable for its length.
        let status = unsafe {
            xboxkrnl::NtQueryDirectoryFile(
                self.handle.handle,
                Handle::NULL,
                0,
                0,
                &mut iosb,
                self.buffer.as_mut_ptr().cast(),
                DIRECTORY_BUFFER as u32,
                crate::ptr::null(),
                false,
            )
        };
        // A directory without entries answers the first query with
        // STATUS_NO_SUCH_FILE.
        if status == KernelError::NoMoreFiles.status() || status == KernelError::NoSuchFile.status() {
            return Ok(false);
        }
        cvt(status)?;
        self.next = Some(0);
        Ok(true)
    }

    /// The entry at `offset` in the buffer, and the offset of the next one.
    /// `offset` leaves room for an entry header in the buffer.
    fn entry_at(&self, offset: usize) -> (DirEntry, Option<usize>) {
        let bytes: &[u8; DIRECTORY_BUFFER] =
            // SAFETY: the buffer is DIRECTORY_BUFFER initialized bytes.
            unsafe { &*self.buffer.as_ptr().cast() };
        let header = size_of::<DirectoryInformation>();
        // SAFETY: the header at `offset` is in the buffer.
        let info = unsafe {
            bytes.as_ptr().add(offset).cast::<DirectoryInformation>().read_unaligned()
        };
        let name_start = offset + header;
        let name_end = (name_start + info.file_name_length as usize).min(DIRECTORY_BUFFER);
        let name = bytes[name_start..name_end].to_vec();
        let entry = DirEntry {
            root: self.root.clone(),
            // SAFETY: the OS string of this target is bytes in any encoding.
            name: unsafe { OsString::from_encoded_bytes_unchecked(name) },
            attr: FileAttr {
                attributes: FileAttributes::from_bits_retain(info.attributes),
                creation_time: info.creation_time,
                last_access_time: info.last_access_time,
                last_write_time: info.last_write_time,
                size: info.end_of_file,
            },
        };
        let next = match info.next_entry_offset {
            0 => None,
            n => Some(offset + n as usize).filter(|&next| next + header <= DIRECTORY_BUFFER),
        };
        (entry, next)
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;
    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        while !self.done {
            let Some(offset) = self.next else {
                match self.fill() {
                    Ok(true) => continue,
                    Ok(false) => self.done = true,
                    Err(error) => {
                        self.done = true;
                        return Some(Err(error));
                    }
                }
                continue;
            };
            let (entry, next) = self.entry_at(offset);
            self.next = next;
            let name = entry.name.as_encoded_bytes();
            if name != b"." && name != b".." {
                return Some(Ok(entry));
            }
        }
        None
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.root.join(&self.name)
    }
    pub fn file_name(&self) -> OsString {
        self.name.clone()
    }
    pub fn metadata(&self) -> io::Result<FileAttr> {
        Ok(self.attr.clone())
    }
    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(self.attr.file_type())
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }
    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }

    // The checks of `access` and `disposition` are those of the Windows
    // backend.
    // Metadata of a file opened only for writing needs READ_ATTRIBUTES.
    fn access(&self) -> io::Result<AccessMask> {
        let mut access = AccessMask::SYNCHRONIZE | AccessMask::READ_ATTRIBUTES;
        if !self.read && !self.write && !self.append {
            return Err(io::const_error!(
                io::ErrorKind::InvalidInput,
                "the file must be opened for reading, writing or appending",
            ));
        }
        if self.read {
            access |= AccessMask::GENERIC_READ;
        }
        if self.write || self.append {
            access |= AccessMask::GENERIC_WRITE;
        }
        Ok(access)
    }

    fn disposition(&self) -> io::Result<CreateDisposition> {
        if !self.write && !self.append {
            if self.truncate || self.create || self.create_new {
                return Err(io::const_error!(
                    io::ErrorKind::InvalidInput,
                    "creating or truncating a file requires write or append access",
                ));
            }
        } else if self.append && self.truncate && !self.create_new {
            return Err(io::const_error!(
                io::ErrorKind::InvalidInput,
                "a file cannot be truncated and appended to",
            ));
        }
        Ok(match (self.create, self.truncate, self.create_new) {
            (_, _, true) => CreateDisposition::Create,
            (true, true, false) => CreateDisposition::OverwriteIf,
            (true, false, false) => CreateDisposition::OpenIf,
            (false, true, false) => CreateDisposition::Overwrite,
            (false, false, false) => CreateDisposition::Open,
        })
    }
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        let mut file = File::open_object(
            path,
            opts.access()?,
            opts.disposition()?,
            CreateOptions::NON_DIRECTORY_FILE,
        )?;
        file.append = opts.append;
        Ok(file)
    }

    /// Opens or creates the object `path` for synchronous I/O. `options`
    /// may require a directory or a file.
    fn open_object(
        path: &Path,
        access: AccessMask,
        disposition: CreateDisposition,
        options: CreateOptions,
    ) -> io::Result<File> {
        with_object_attributes(path, |attributes| {
            let mut iosb = IoStatusBlock::default();
            let mut handle = Handle::NULL;
            // SAFETY: every pointer refers to a live local or to `attributes`,
            // which outlives the call.
            cvt(unsafe {
                xboxkrnl::NtCreateFile(
                    &mut handle,
                    access | AccessMask::SYNCHRONIZE,
                    attributes,
                    &mut iosb,
                    crate::ptr::null(),
                    FileAttributes::NORMAL,
                    SHARE_ALL,
                    disposition,
                    options | CreateOptions::SYNCHRONOUS_IO_NONALERT,
                )
            })?;
            Ok(File { handle, append: false })
        })
    }

    fn query<T>(&self, class: FileInformationClass, info: &mut T) -> io::Result<()> {
        let mut iosb = IoStatusBlock::default();
        // SAFETY: `self.handle` is open and `info` is writable for its size.
        cvt(unsafe {
            xboxkrnl::NtQueryInformationFile(
                self.handle,
                &mut iosb,
                (info as *mut T).cast(),
                size_of::<T>() as u32,
                class,
            )
        })
    }

    fn set<T>(&self, class: FileInformationClass, info: &T) -> io::Result<()> {
        let mut iosb = IoStatusBlock::default();
        // SAFETY: `self.handle` is open and `info` is readable for its size.
        cvt(unsafe {
            xboxkrnl::NtSetInformationFile(
                self.handle,
                &mut iosb,
                (info as *const T).cast(),
                size_of::<T>() as u32,
                class,
            )
        })
    }

    /// Deletes the file or directory when its last handle is closed.
    fn delete_on_close(&self) -> io::Result<()> {
        self.set(FileInformationClass::Disposition, &1u8)
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        let mut info = NetworkOpenInformation::default();
        self.query(FileInformationClass::NetworkOpen, &mut info)?;
        Ok(FileAttr::from_network_open(&info))
    }

    pub fn fsync(&self) -> io::Result<()> {
        let mut iosb = IoStatusBlock::default();
        // SAFETY: `self.handle` is open and `iosb` is a live local.
        cvt(unsafe { xboxkrnl::NtFlushBuffersFile(self.handle, &mut iosb) })
    }
    pub fn datasync(&self) -> io::Result<()> {
        self.fsync()
    }
    pub fn lock(&self) -> io::Result<()> {
        unsupported()
    }
    pub fn lock_shared(&self) -> io::Result<()> {
        unsupported()
    }
    pub fn try_lock(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(unsupported_err()))
    }
    pub fn try_lock_shared(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(unsupported_err()))
    }
    pub fn unlock(&self) -> io::Result<()> {
        unsupported()
    }
    pub fn truncate(&self, size: u64) -> io::Result<()> {
        let old_size = self.file_attr()?.size;
        self.set(FileInformationClass::EndOfFile, &size)?;
        if size > old_size {
            // FATX leaves the old contents of the clusters in the grown part.
            self.write_zeros(old_size, size)?;
        }
        Ok(())
    }

    /// Writes zeros from `start` to `end`. The file position is kept.
    fn write_zeros(&self, start: u64, end: u64) -> io::Result<()> {
        const ZEROS: [u8; 4096] = [0; 4096];
        let position = self.tell()?;
        let mut offset = start;
        while offset < end {
            let length = (end - offset).min(ZEROS.len() as u64) as usize;
            match self.write_at(&ZEROS[..length], &offset)? {
                0 => return Err(io::const_error!(io::ErrorKind::WriteZero, "failed to write zeros")),
                n => offset += n as u64,
            }
        }
        self.set(FileInformationClass::Position, &position)
    }

    /// Reads at the file position into `length` bytes at `buffer`.
    fn read_raw(&self, buffer: *mut u8, length: usize) -> io::Result<usize> {
        if length == 0 {
            return Ok(0);
        }
        let mut iosb = IoStatusBlock::default();
        // SAFETY: `self.handle` is open for synchronous I/O, the caller passes
        // a buffer writable for `length` bytes, and a null offset reads at
        // the file position.
        let status = unsafe {
            xboxkrnl::NtReadFile(
                self.handle,
                Handle::NULL,
                0,
                0,
                &mut iosb,
                buffer,
                u32::try_from(length).unwrap_or(u32::MAX),
                crate::ptr::null(),
            )
        };
        if status == KernelError::EndOfFile.status() {
            return Ok(0);
        }
        cvt(status)?;
        Ok(iosb.information as usize)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_raw(buf.as_mut_ptr(), buf.len())
    }

    pub fn read_buf(&self, mut cursor: BorrowedCursor<'_, u8>) -> io::Result<()> {
        // SAFETY: the kernel only writes to the buffer.
        let buffer = unsafe { cursor.as_mut() };
        let n = self.read_raw(buffer.as_mut_ptr().cast(), buffer.len())?;
        // SAFETY: the kernel initialized `n` bytes.
        unsafe { cursor.advance(n) };
        Ok(())
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        crate::io::default_read_vectored(|b| self.read(b), bufs)
    }
    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let offset: *const u64 = if self.append { &END_OF_FILE } else { crate::ptr::null() };
        self.write_at(buf, offset)
    }

    /// Writes at `offset`, or at the file position for a null `offset`.
    fn write_at(&self, buf: &[u8], offset: *const u64) -> io::Result<usize> {
        let mut iosb = IoStatusBlock::default();
        // SAFETY: `self.handle` is open for synchronous I/O, `buf` is readable
        // for its length, and `offset` is null or valid for the call.
        cvt(unsafe {
            xboxkrnl::NtWriteFile(
                self.handle,
                Handle::NULL,
                0,
                0,
                &mut iosb,
                buf.as_ptr(),
                u32::try_from(buf.len()).unwrap_or(u32::MAX),
                offset,
            )
        })?;
        Ok(iosb.information as usize)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        crate::io::default_write_vectored(|b| self.write(b), bufs)
    }
    pub fn is_write_vectored(&self) -> bool {
        false
    }
    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let (base, offset) = match pos {
            SeekFrom::Start(n) => (0, i64::try_from(n).ok()),
            SeekFrom::End(n) => (self.file_attr()?.size, Some(n)),
            SeekFrom::Current(n) => (self.tell()?, Some(n)),
        };
        let Some(target) = offset.and_then(|offset| base.checked_add_signed(offset)) else {
            return Err(io::const_error!(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            ));
        };
        self.set(FileInformationClass::Position, &target)?;
        Ok(target)
    }

    pub fn size(&self) -> Option<io::Result<u64>> {
        Some(self.file_attr().map(|a| a.size))
    }

    pub fn tell(&self) -> io::Result<u64> {
        let mut position = 0u64;
        self.query(FileInformationClass::Position, &mut position)?;
        Ok(position)
    }

    pub fn duplicate(&self) -> io::Result<File> {
        let mut handle = Handle::NULL;
        // SAFETY: `self.handle` is open and `handle` is a live local.
        let duplicated = unsafe {
            xam::DuplicateHandle(
                Handle::CURRENT_PROCESS,
                self.handle,
                Handle::CURRENT_PROCESS,
                &mut handle,
                AccessMask::empty(),
                false,
                DuplicateOptions::SAME_ACCESS,
            )
        };
        if duplicated.0 == 0 {
            return Err(io::const_error!(io::ErrorKind::Uncategorized, "DuplicateHandle failed"));
        }
        Ok(File { handle, append: self.append })
    }

    pub fn set_permissions(&self, perm: FilePermissions) -> io::Result<()> {
        let attributes = perm.to_set();
        self.set(
            FileInformationClass::Basic,
            &BasicInformation { attributes: attributes.bits(), ..Default::default() },
        )?;
        // FATX accepts and drops the read-only attribute.
        if perm.readonly() && !self.file_attr()?.perm().readonly() {
            return Err(io::const_error!(
                io::ErrorKind::Unsupported,
                "the file system does not keep the read-only attribute",
            ));
        }
        Ok(())
    }

    pub fn set_times(&self, times: FileTimes) -> io::Result<()> {
        let to_filetime = |time: Option<SystemTime>| match time {
            None => Ok(0),
            Some(time) => time.to_filetime().ok_or(io::const_error!(
                io::ErrorKind::InvalidInput,
                "the time is out of range for the file system",
            )),
        };
        self.set(
            FileInformationClass::Basic,
            &BasicInformation {
                last_access_time: to_filetime(times.accessed)?,
                last_write_time: to_filetime(times.modified)?,
                ..Default::default()
            },
        )
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // SAFETY: `self.handle` is owned by this `File` and is closed only here.
        unsafe { xboxkrnl::NtClose(self.handle) };
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }
    pub fn mkdir(&self, p: &Path) -> io::Result<()> {
        File::open_object(
            p,
            AccessMask::LIST_DIRECTORY,
            CreateDisposition::Create,
            CreateOptions::DIRECTORY_FILE,
        )
        .map(drop)
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File(handle={:#x})", self.handle.raw())
    }
}

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    let handle = File::open_object(
        p,
        AccessMask::LIST_DIRECTORY,
        CreateDisposition::Open,
        CreateOptions::DIRECTORY_FILE,
    )?;
    Ok(ReadDir {
        handle,
        root: Arc::new(p.to_path_buf()),
        buffer: Box::new([0; DIRECTORY_BUFFER / 8]),
        next: None,
        done: false,
    })
}

pub fn unlink(p: &Path) -> io::Result<()> {
    File::open_object(p, AccessMask::DELETE, CreateDisposition::Open, CreateOptions::NON_DIRECTORY_FILE)?
        .delete_on_close()
}

pub fn rename(old: &Path, new: &Path) -> io::Result<()> {
    let file = File::open_object(old, AccessMask::DELETE, CreateDisposition::Open, CreateOptions::empty())?;
    let name = object_name(new)?;
    let info = RenameInformation {
        replace_if_exists: 1,
        root_directory: Handle::NULL,
        // `object_name` limits the length to `u16::MAX`.
        file_name: AnsiString::new(&name).unwrap(),
    };
    file.set(FileInformationClass::Rename, &info)
}

pub fn set_perm(p: &Path, perm: FilePermissions) -> io::Result<()> {
    File::open_object(
        p,
        AccessMask::READ_ATTRIBUTES | AccessMask::WRITE_ATTRIBUTES,
        CreateDisposition::Open,
        CreateOptions::empty(),
    )?
    .set_permissions(perm)
}

pub fn set_times(p: &Path, times: FileTimes) -> io::Result<()> {
    File::open_object(p, AccessMask::WRITE_ATTRIBUTES, CreateDisposition::Open, CreateOptions::empty())?
        .set_times(times)
}

/// The kernel has no links, so this is [`set_times`].
pub fn set_times_nofollow(p: &Path, times: FileTimes) -> io::Result<()> {
    set_times(p, times)
}

pub fn rmdir(p: &Path) -> io::Result<()> {
    File::open_object(p, AccessMask::DELETE, CreateDisposition::Open, CreateOptions::DIRECTORY_FILE)?
        .delete_on_close()
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}
pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}
pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(p: &Path) -> io::Result<FileAttr> {
    with_object_attributes(p, |attributes| {
        let mut info = NetworkOpenInformation::default();
        // SAFETY: `attributes` and `info` are live and `info` has the size of
        // the information the call writes.
        cvt(unsafe {
            xboxkrnl::NtQueryFullAttributesFile(attributes, (&raw mut info).cast())
        })?;
        Ok(FileAttr::from_network_open(&info))
    })
}

/// The kernel has no links, so this is [`stat`].
pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    stat(p)
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}
