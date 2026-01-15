use std::env;
use std::path::PathBuf;

fn main() {
    // This crate hosts both the STARS Core C++ sources and the FFI shim.
    // Emit metadata so build scripts can locate them.

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let core_path = manifest.join("core");
    let ffi_path = manifest.join("ffi");

    if core_path.exists() {
        println!("cargo:warning=STARS Core located at: {}", core_path.display());
        println!("cargo:core_root={}", core_path.display());
    } else {
        println!("cargo:warning=STARS Core not found in expected location: {}", core_path.display());
    }

    if ffi_path.exists() {
        println!("cargo:warning=stars_ffi located at: {}", ffi_path.display());
        println!("cargo:ffi_root={}", ffi_path.display());
    } else {
        println!("cargo:warning=stars_ffi not found in expected location: {}", ffi_path.display());
    }
}
