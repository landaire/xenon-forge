//! Points the crate at the thunk library of src/sdk, which `src/thunks.rs`
//! assembles. Buck sets the same variable itself.

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"));
    let thunks = manifest.join("../../sdk/thunks");
    let thunks = thunks
        .canonicalize()
        .unwrap_or_else(|error| panic!("{}: {error}", thunks.display()));
    println!("cargo::rerun-if-changed={}", thunks.display());
    println!("cargo::rustc-env=XBOX360_SDK_THUNKS={}", thunks.display());
}
