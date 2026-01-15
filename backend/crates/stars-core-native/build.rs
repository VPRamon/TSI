use std::env;
use std::path::PathBuf;

fn main() {
    // This crate only acts as a marker/location for the native STARS Core sources.
    // During native builds other crates may inspect this crate's path to find the
    // STARS Core CMake project. We simply emit the path as cargo metadata.

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let core_path = manifest.join("../src/scheduler/core");

    // Prefer a sibling `src/scheduler/core` if present; if the user moved the core
    // sources into this crate, change the path accordingly.
    let crate_core_path = manifest.join("src").join("core");
    let path_to_use = if crate_core_path.exists() { crate_core_path } else { core_path };

    println!("cargo:warning=stars-core-native located at: {}", path_to_use.display());
    println!("cargo:root={}", path_to_use.display());
}
