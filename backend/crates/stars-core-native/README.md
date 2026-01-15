# stars-core-native

This helper crate exposes a workspace-local location for the STARS Core native C++ sources.

It is intended to make a stable path available for other build scripts (for example, the
`stars-core-sys` crate) that need to build or link the STARS Core C++ project from the
workspace. It does not contain Rust sources by design.

Usage: other crates can locate the CMake project at `../crates/stars-core-native` or use
the `cargo:root` metadata emitted by this crate's build script.
