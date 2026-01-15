//! Build script for stars-core-sys
//!
//! This script handles finding or building the stars_ffi C++ library
//! and its dependencies (STARS Core).

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=STARS_FFI_LIB_DIR");
    println!("cargo:rerun-if-env-changed=STARS_CORE_DIR");

    let _out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Strategy 1: Check for pre-built library via environment variable
    if let Ok(lib_dir) = env::var("STARS_FFI_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
        println!("cargo:rustc-link-lib=dylib=stars_ffi");
        link_stars_core_deps();
        return;
    }

    // Strategy 2: Try pkg-config
    if try_pkg_config() {
        return;
    }

    // Strategy 3: Build from source if feature enabled
    #[cfg(feature = "build-native")]
    {
        build_from_source(&out_dir);
        return;
    }

    // Strategy 4: Look for library in common locations
    let search_paths = [
        "/usr/local/lib",
        "/usr/lib",
        "/usr/lib/x86_64-linux-gnu",
        "../../src/scheduler/stars_ffi/build",
    ];

    for path in &search_paths {
        let lib_path = PathBuf::from(path).join("libstars_ffi.so");
        if lib_path.exists() {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=dylib=stars_ffi");
            link_stars_core_deps();
            return;
        }
    }

    // If we get here, we couldn't find the library
    eprintln!("Could not find stars_ffi library.");
    eprintln!("Options:");
    eprintln!("  1. Set STARS_FFI_LIB_DIR environment variable");
    eprintln!("  2. Install stars_ffi system-wide");
    eprintln!("  3. Enable 'build-native' feature to build from source");
    panic!("stars_ffi library not found");
}

fn try_pkg_config() -> bool {
    match pkg_config::Config::new()
        .atleast_version("0.1.0")
        .probe("stars_ffi")
    {
        Ok(_) => {
            println!("cargo:info=Found stars_ffi via pkg-config");
            true
        }
        Err(_) => false,
    }
}

#[cfg(feature = "build-native")]
fn build_from_source(out_dir: &PathBuf) {
    use cmake::Config;

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Prefer consolidated layout: ffi and core in stars-core-native crate
    let stars_core_native = manifest.join("../stars-core-native");
    let stars_ffi_dir = stars_core_native.join("ffi");
    let stars_core_root = stars_core_native.join("core");

    if !stars_ffi_dir.exists() {
        panic!(
            "stars_ffi source directory not found at: {}\nExpected layout: crates/stars-core-native/ffi/",
            stars_ffi_dir.display()
        );
    }

    if !stars_core_root.exists() {
        panic!(
            "STARS Core source directory not found at: {}\nExpected layout: crates/stars-core-native/core/",
            stars_core_root.display()
        );
    }

    let dst = Config::new(&stars_ffi_dir)
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("STARS_FFI_BUILD_STATIC", "OFF")
        .define("STARS_CORE_ROOT", stars_core_root.to_str().unwrap())
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=dylib=stars_ffi");

    // Copy header for downstream crates
    let header_src = stars_ffi_dir.join("include/stars_ffi.h");
    let header_dst = out_dir.join("stars_ffi.h");
    std::fs::copy(&header_src, &header_dst).expect("Failed to copy header");

    println!("cargo:include={}", out_dir.display());
    link_stars_core_deps();
}

fn link_stars_core_deps() {
    // Link C++ standard library
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=c++");

    // STARS Core dependencies that may be needed at runtime
    // These are typically linked transitively through stars_ffi,
    // but we hint them for RPATH/loader purposes
    #[allow(unused_variables)]
    let stars_libs = [
        "stars-serialization",
        "stars-scheduling-blocks",
        "stars-scheduler",
        "stars-json-builder",
        "stars-constraints",
        "stars-time",
        "stars-coordinates",
        "stars-astro",
    ];

    // Only add library hints if system-libs feature is enabled
    #[cfg(feature = "system-libs")]
    for lib in &stars_libs {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }
}
