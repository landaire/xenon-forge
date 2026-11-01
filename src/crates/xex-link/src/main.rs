//! Link a fully-linked PPC ELF executable into an Xbox 360 XEX.
//!
//! This is the single post-link step of the toolchain: it takes the ELF that
//! rustc/clang + rust-lld produce and emits a devkit-signed XEX that Xenia (and
//! the devkit) load directly. The intermediate PE image is built in memory; no
//! separate ELF-to-EXE or EXE-to-XEX tool is required. `link-script` writes
//! the GNU ld script that the ELF is linked with, from the same manifest.

mod args;
mod elf;
mod error;
mod exports;
mod imports;
mod layout;
mod link;
mod manifest;
mod pe;
mod resources;
mod script;

use std::process::ExitCode;

use crate::args::Command;
use crate::args::USAGE;
use crate::error::Result;
use crate::error::io;
use crate::link::link;
use crate::script::link_script;

fn run(command: Command) -> Result<()> {
    match command {
        Command::Link(config) => link(config),
        Command::LinkScript { output, manifest } => {
            io(&output, std::fs::write(&output, link_script(&manifest)))
        }
    }
}

fn main() -> ExitCode {
    let result = Command::from_args().and_then(run);
    let Err(report) = result else {
        return ExitCode::SUCCESS;
    };

    eprintln!("xex-link: {report:?}\n\n{USAGE}");

    ExitCode::FAILURE
}
