//! Build script: places the shared workspace `memory.x` where the linker can
//! find it, and requests a rebuild whenever it changes.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Copy memory.x into the Cargo output directory and add it to the linker
    // search path, exactly as the crate root used to do.
    let out = &PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR unset"));
    File::create(out.join("memory.x"))
        .expect("unable to create memory.x")
        .write_all(include_bytes!("../../memory.x"))
        .expect("unable to write memory.x");
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=../../memory.x");
}
