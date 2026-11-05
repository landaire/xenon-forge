//! Console information, DashLaunch state, debug output, directory listing,
//! executable cache synchronization, the title and module loaders, kernel
//! expansions, the system management controller, and a global allocator over
//! the kernel pool.

#![no_std]
#![cfg(any(target_os = "xbox360", xbox360_docs))]

pub mod console;
pub mod dashlaunch;
pub mod debug;
pub mod directory;
pub mod expansion;
pub mod memory;
pub mod module;
pub mod pool;
pub mod smc;
pub mod title;
