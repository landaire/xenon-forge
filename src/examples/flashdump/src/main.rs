//! System module that reads every physical sector of the NAND flash through
//! `\Device\Flash` and writes it to `DEVKIT:\flashraw.bin` as 0x210-byte
//! records (512 data bytes, then the 16 spare bytes) in physical order. It
//! writes a summary to `DEVKIT:\flashraw.txt`, then creates
//! `DEVKIT:\flashraw.done`. It issues read requests only.
//!
//! `bin/buck2 run //src/examples/flashdump:flashdump` loads it with XBDM
//! `dbgextld`. It stays loaded until the console restarts, so restart the
//! console before running it again. Progress goes to
//! the debug output. When `flashraw.done` exists, fetch the dump with
//! `xeedee file get 'DEVKIT:\flashraw.bin'`. To continue a dump that
//! stopped, write the first block to read to `DEVKIT:\flashdump.start`;
//! `flashraw.bin` then starts at that block, and the module removes the file.

#![no_main]
#![feature(xbox360_ext)]

mod dma;
mod dump;
mod error;
mod extension;
mod flash;
mod report;

use std::os::xbox360::release_tls_slots;
use std::thread;

use xbox360::sys::xboxkrnl::Boolean;
use xbox360::sys::xboxkrnl::DllEntryReason;

use crate::report::Report;

xbox360::sys::bridged!(#[unsafe(no_mangle)] pub _start, module_main);

const TXT_PATH: &str = "\\Device\\Harddisk0\\Partition1\\DEVKIT\\flashraw.txt";
const DONE_PATH: &str = "\\Device\\Harddisk0\\Partition1\\DEVKIT\\flashraw.done";

fn worker() {
    // The marker of an earlier run would announce this one as done.
    match std::fs::remove_file(DONE_PATH) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            println!("flashdump: {DONE_PATH}: {error}");
            return;
        }
        _ => {}
    }

    let mut report = Report::default();
    let outcome = dump::run(&mut report);
    match &outcome {
        Ok(()) => println!("flashdump: wrote {} sectors to {}", report.sectors_written, dump::BIN_PATH),
        Err(error) => println!("flashdump: failed: {error}"),
    }

    if let Err(error) = std::fs::write(TXT_PATH, report.render(&outcome)) {
        println!("flashdump: {TXT_PATH}: {error}");
    }

    let marker: &[u8] = match outcome {
        Ok(()) => b"ok\n",
        Err(_) => b"error\n",
    };
    if let Err(error) = std::fs::write(DONE_PATH, marker) {
        println!("flashdump: {DONE_PATH}: {error}");
    }
}

extern "C" fn module_main(_module: u32, reason: u32, _reserved: u32) -> Boolean {
    let loaded = match DllEntryReason::from_raw(reason) {
        Some(DllEntryReason::ProcessAttach) => thread::Builder::new().stack_size(64 * 1024).spawn(worker).is_ok(),
        Some(DllEntryReason::ProcessDetach) => {
            // SAFETY: the module is unloading; nothing uses its TLS slots
            // after this.
            unsafe { release_tls_slots() };
            true
        }
        // Thread attach and detach need no work.
        _ => true,
    };

    Boolean::from(loaded)
}
