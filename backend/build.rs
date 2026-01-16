//! Build script for tsi-rust
//!
//! This script handles finding or building the stars_ffi C++ library
//! and its dependencies (STARS Core) when the `stars-core` feature is enabled.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Only process stars_ffi linking when the stars-core feature is enabled
    #[cfg(feature = "stars-core")]
    {
        println!("cargo:rerun-if-env-changed=STARS_FFI_LIB_DIR");
        println!("cargo:rerun-if-env-changed=STARS_CORE_DIR");
        
        link_stars_ffi();
    }
}

#[cfg(feature = "stars-core")]
fn link_stars_ffi() {
    use std::env;
    use std::path::PathBuf;
    
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
        build_from_source(&_out_dir);
        return;
    }

    // Strategy 4: Look for library in common locations
    #[cfg(not(feature = "build-native"))]
    {
        let search_paths = [
            "/usr/local/lib",
            "/usr/lib",
            "/usr/lib/x86_64-linux-gnu",
            "/opt/stars/lib",
        ];

        for path in &search_paths {
            let lib_path = PathBuf::from(path).join("libstars_ffi.so");
            if lib_path.exists() {
                println!("cargo:rustc-link-search=native={}", path);
                println!("cargo:rustc-link-lib=dylib=stars_ffi");
                link_stars_core_deps();
                return;
            }
            // Also check for .dylib on macOS
            let lib_path = PathBuf::from(path).join("libstars_ffi.dylib");
            if lib_path.exists() {
                println!("cargo:rustc-link-search=native={}", path);
                println!("cargo:rustc-link-lib=dylib=stars_ffi");
                link_stars_core_deps();
                return;
            }
        }
    }

    // If we get here, we couldn't find the library
    #[cfg(not(feature = "build-native"))]
    {
        eprintln!("Could not find stars_ffi library.");
        eprintln!("Options:");
        eprintln!("  1. Set STARS_FFI_LIB_DIR environment variable");
        eprintln!("  2. Install stars_ffi system-wide");
        eprintln!("  3. Enable 'build-native' feature to build from source");
        panic!("stars_ffi library not found");
    }
}

#[cfg(feature = "stars-core")]
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

#[cfg(all(feature = "stars-core", feature = "build-native"))]
fn build_from_source(out_dir: &std::path::PathBuf) {
    use cmake::Config;
    use std::env;
    use std::path::PathBuf;

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Look for stars_ffi sources
    // Option 1: In a native subdirectory
    let stars_ffi_dir = manifest.join("native/ffi");
    let stars_core_root = manifest.join("native/core");

    if !stars_ffi_dir.exists() {
        // Option 2: From environment
        let _stars_ffi_dir = env::var("STARS_FFI_SRC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                panic!(
                    "stars_ffi source directory not found.\n\
                     Expected at: {}\n\
                     Or set STARS_FFI_SRC_DIR environment variable.",
                    stars_ffi_dir.display()
                );
            });
    }

    let stars_core_root = env::var("STARS_CORE_DIR")
        .map(PathBuf::from)
        .unwrap_or(stars_core_root);

    if !stars_core_root.exists() {
        panic!(
            "STARS Core source directory not found at: {}\n\
             Set STARS_CORE_DIR environment variable.",
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

    // Copy header for downstream use
    let header_src = stars_ffi_dir.join("include/stars_ffi.h");
    let header_dst = out_dir.join("stars_ffi.h");
    if header_src.exists() {
        std::fs::copy(&header_src, &header_dst).expect("Failed to copy header");
        println!("cargo:include={}", out_dir.display());
    }

    link_stars_core_deps();
}

#[cfg(feature = "stars-core")]
fn link_stars_core_deps() {
    // Link C++ standard library
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=c++");

    // Note: stars_ffi library transitively provides all STARS Core dependencies
    // No need to explicitly link individual STARS Core components

    // Try to find boost libraries via pkg-config
    if pkg_config::probe_library("boost_log").is_err() {
        // Fallback: link boost libraries directly
        for boost_lib in &[
            "boost_log",
            "boost_log_setup",
            "boost_thread",
            "boost_system",
            "boost_filesystem",
            "boost_date_time",
            "boost_serialization",
            "boost_regex",
        ] {
            println!("cargo:rustc-link-lib=dylib={}", boost_lib);
        }
    }
}
