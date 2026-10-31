pub use super::common::Args;

use crate::ffi::OsString;
use crate::sync::atomic::{AtomicU8, Ordering};
use crate::vec::Vec;
use xbox360_sys::xboxkrnl;

const COMMAND_LINE_CAPACITY: usize = 0x200;

pub fn args() -> Args {
    let mut command_line = [0; COMMAND_LINE_CAPACITY];
    let start = xboxkrnl::ExLoadedCommandLine.address() as usize;
    let length = command_line
        .iter_mut()
        .enumerate()
        .find_map(|(index, slot)| {
            // SAFETY: the loader resolves the import to a mapped,
            // NUL-terminated kernel buffer before starting the title.
            let byte = unsafe {
                AtomicU8::from_ptr(core::ptr::with_exposed_provenance_mut(start + index))
            }
            .load(Ordering::Relaxed);
            if byte == 0 {
                Some(index)
            } else {
                *slot = byte;
                None
            }
        })
        .unwrap_or(COMMAND_LINE_CAPACITY);
    Args::new(parse_command_line(&command_line[..length]))
}

/// Parse the command line using the Microsoft C/C++ argument rules used by
/// Xbox executables. The first, executable-name argument has the simpler
/// executable-name quoting rules.
fn parse_command_line(command_line: &[u8]) -> Vec<OsString> {
    let mut arguments = Vec::new();
    let mut at = 0;

    if command_line.is_empty() {
        return arguments;
    }

    let mut executable = Vec::new();
    let mut quoted = false;
    while at < command_line.len() {
        match command_line[at] {
            b'"' => quoted = !quoted,
            b' ' | b'\t' if !quoted => break,
            byte => executable.push(byte),
        }
        at += 1;
    }
    arguments.push(os_string(executable));

    while at < command_line.len() {
        while at < command_line.len() && matches!(command_line[at], b' ' | b'\t') {
            at += 1;
        }
        if at == command_line.len() {
            break;
        }

        let mut argument = Vec::new();
        let mut quoted = false;
        while at < command_line.len() {
            match command_line[at] {
                b' ' | b'\t' if !quoted => break,
                b'\\' => {
                    let start = at;
                    while at < command_line.len() && command_line[at] == b'\\' {
                        at += 1;
                    }
                    let count = at - start;
                    if at < command_line.len() && command_line[at] == b'"' {
                        argument.extend(core::iter::repeat_n(b'\\', count / 2));
                        if count % 2 == 1 {
                            argument.push(b'"');
                            at += 1;
                        }
                    } else {
                        argument.extend(core::iter::repeat_n(b'\\', count));
                    }
                    continue;
                }
                b'"' if quoted && command_line.get(at + 1) == Some(&b'"') => {
                    argument.push(b'"');
                    at += 2;
                    continue;
                }
                b'"' => quoted = !quoted,
                byte => argument.push(byte),
            }
            at += 1;
        }
        arguments.push(os_string(argument));
    }

    arguments
}

fn os_string(bytes: Vec<u8>) -> OsString {
    // SAFETY: Xbox uses std's byte-based OS string representation, for which
    // every byte sequence is valid.
    unsafe { OsString::from_encoded_bytes_unchecked(bytes) }
}
