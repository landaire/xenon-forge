//! The thunk library of `src/sdk/thunks`: the macros of `macros.inc` and
//! the invocations that `src/tools/xbox360-bindgen` generates from the headers,
//! a function under its own name for each function import of
//! [`ffi`](crate::ffi) and the `__imp_<name>` slot of each data import. C
//! titles link the same files through `xbox360//src/sdk:xbox360`.
//!
//! `XBOX360_SDK_THUNKS` is the directory, set by build.rs for Cargo and by
//! the BUCK file for Buck. The generated files include the macros with the
//! C preprocessor, which this assembly does not run; here the `#include`
//! line is a comment, and the macros are put in front.

macro_rules! thunks {
    ($file:literal) => {
        core::arch::global_asm!(
            concat!(
                include_str!(concat!(env!("XBOX360_SDK_THUNKS"), "/macros.inc")),
                include_str!(concat!(env!("XBOX360_SDK_THUNKS"), "/", $file)),
            ),
            options(raw)
        );
    };
}

#[cfg(target_arch = "powerpc")]
thunks!("ppc32.S");

#[cfg(target_arch = "powerpc64")]
thunks!("ppc64.S");
