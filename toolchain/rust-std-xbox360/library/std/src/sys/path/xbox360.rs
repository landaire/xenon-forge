//! Paths of the Xbox 360. The kernel separates object name components with
//! `\` and rejects `/`, so `\` is the main separator. `/` is accepted as a
//! separator too, as on Windows, and the file system backend passes it to
//! the kernel as `\`.
//!
//! A path is absolute when it starts with a separator, as kernel object
//! names do (`\Device\Harddisk0\Partition1\file`), or with a drive name
//! (`game:\file`). Drive names are not prefixes: they are the first normal
//! component.

use crate::ffi::OsStr;
use crate::io;
use crate::path::{Component, Path, PathBuf, Prefix};

path_separator_bytes!(b'\\', b'/');

#[inline]
pub const fn is_verbatim_sep(b: u8) -> bool {
    b == b'\\'
}

#[inline]
pub fn parse_prefix(_: &OsStr) -> Option<Prefix<'_>> {
    None
}

pub const HAS_PREFIXES: bool = false;

/// Whether the first component of `path` is a drive name such as `game:`.
pub(crate) fn has_drive(path: &Path) -> bool {
    match path.components().next() {
        Some(Component::Normal(first)) => {
            let first = first.as_encoded_bytes();
            first.len() > 1 && first.ends_with(b":")
        }
        _ => false,
    }
}

pub(crate) fn is_absolute(path: &Path) -> bool {
    path.has_root() || has_drive(path)
}

/// `path` without `.` components and repeated separators. There is no
/// current directory to resolve a relative path against.
pub(crate) fn absolute(path: &Path) -> io::Result<PathBuf> {
    if !is_absolute(path) {
        return Err(io::const_error!(
            io::ErrorKind::Unsupported,
            "relative paths cannot be made absolute: there is no current directory",
        ));
    }
    Ok(path.components().collect())
}
