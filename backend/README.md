# Backend Rust Workspace Organization

This directory contains the Telescope Scheduling Intelligence (TSI) Rust backend and native C++ integrations.

## Directory Structure

```
backend/
├── Cargo.toml                    # Package manifest
├── build.rs                      # Build script for native library linkage
├── src/                          # Main Rust backend sources
│   ├── lib.rs                    # TSI Rust library entry point
│   ├── api/                      # Python bindings (PyO3)
│   ├── db/                       # Database and repository layer
│   ├── models/                   # Domain models
│   ├── services/                 # Business logic
│   └── scheduler/                # Scheduling integration module
│       ├── mod.rs                # Scheduler module exports
│       ├── stars.rs              # STARS Core Rust API (FFI declarations + safe wrappers)
│       └── tests.rs              # Integration tests for STARS Core
│
└── native/                       # Native C++ FFI implementation
    └── ffi/                      # C FFI shim wrapping STARS Core C++
        ├── include/stars_ffi.h   # C header (API for Rust FFI)
        ├── src/stars_ffi.cpp     # C++ implementation
        ├── CMakeLists.txt        # CMake build configuration
        └── stars_ffi.pc.in       # pkg-config template
```

## Dependency Graph

```
tsi-rust (main backend)
    └─> stars.rs (feature-gated via "stars-core")
            └─> libstars_ffi.so (C FFI shim)
                    └─> STARS Core C++ library (docker/deps/stars-core)
```

## Features

- `local-repo` (default): In-memory repository backend for testing
- `postgres-repo`: PostgreSQL repository backend with Diesel ORM
- `stars-core`: Enable STARS Core scheduling library integration
- `build-native`: Build stars_ffi from source (includes `stars-core`)

## Building

### Without STARS Core (default)

```bash
cargo build
cargo test
```

### With STARS Core (requires libstars_ffi installed)

```bash
# If stars_ffi is installed system-wide or via Docker:
cargo test --features stars-core

# Or specify library location:
STARS_FFI_LIB_DIR=/usr/local/lib cargo test --features stars-core
```

### Docker Build (recommended)

The Docker image builds everything:

```bash
cd docker
docker compose build
docker compose run --rm dev cargo test --features stars-core
```

## Design Principles

### Separation of Concerns

1. **Native C++ code** (`crates/stars-core-native/`)
   - Contains only C++ sources and minimal build helper
   - `core/` is a git submodule pointing to STARS Core upstream
   - `ffi/` contains the C ABI wrapper library (`stars_ffi`)

2. **Low-level FFI** (`crates/stars-core-sys/`)
   - Raw `extern "C"` function declarations
   - Build script that finds or builds the native library
   - No safe abstractions, minimal dependencies

3. **Safe Rust wrapper** (`crates/stars-core/`)
   - RAII handles for C++ objects
   - `Result`-based error handling
   - Serde-compatible types for JSON interchange
   - Zero unsafe code in user-facing API

4. **Backend integration** (`src/scheduler/`)
   - Re-exports `stars-core` API
   - Optional feature (`stars-core`) so backend can build without native libs
   - Extends API with TSI-specific types if needed

### Build Flexibility

The `stars-core-sys` crate supports multiple build strategies:

1. **Pre-built library** (fastest for development)
   ```bash
   export STARS_FFI_LIB_DIR=/path/to/installed/lib
   cargo build
   ```

2. **System-installed library**
   ```bash
   # If stars_ffi is installed system-wide
   cargo build  # pkg-config will find it
   ```

3. **Build from source** (for CI or first-time setup)
   ```bash
   cargo build -p stars-core-sys --features build-native
   ```

4. **Optional feature** (build backend without STARS Core)
   ```bash
   cargo build  # Excludes stars-core by default
   cargo build --features stars-core  # Include STARS scheduling
   ```

## Usage

### From Rust

```rust
use tsi_rust::scheduler::stars::{Context, Blocks, SchedulingParams, run_scheduler};

let ctx = Context::from_file("schedule.json")?;
let blocks = Blocks::from_file("schedule.json")?;
let params = SchedulingParams::default();
let schedule = run_scheduler(&ctx, &blocks, None, params)?;

let stats = schedule.stats()?;
println!("Scheduled: {}/{}", stats.scheduled_count, stats.total_blocks);
```

### Building the Native Library

If you need to manually build the C++ FFI library:

```bash
# Option 1: Via Cargo (recommended)
cd backend
cargo build -p stars-core-sys --features build-native

# Option 2: Direct CMake build
cd backend/crates/stars-core-native/ffi
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DSTARS_CORE_ROOT=../core
make
sudo make install
```

## Development Workflow

1. **Working on Rust code only**: Just run `cargo build` (without `stars-core` feature)

2. **Working with STARS integration**: 
   - First time: `cargo build --features stars-core,build-native`
   - Subsequent builds: `cargo build --features stars-core`

3. **Updating STARS Core**:
   ```bash
   cd backend/crates/stars-core-native/core
   git pull
   cd ../../..
   cargo clean -p stars-core-sys
   cargo build --features stars-core,build-native
   ```

## Testing

```bash
# Run all tests except those requiring native libs
cargo test

# Run STARS integration tests and build the native C++ libs from source
cargo test --features build-native

# If you already have a system-installed stars_ffi (advanced), you can also run:
# cargo test --features stars-core
```

## CI/CD Considerations

- CI can build without native libs by default (faster)
- Separate job for STARS integration tests (with `build-native` feature)
- Docker images should either:
  - Pre-install `stars_ffi` system-wide, or
  - Build with `build-native` feature enabled

## Why This Structure?

**✅ Pros:**
- Clean separation between native C++ and Rust code
- Rust crates follow standard conventions (`crates/` for workspace members)
- Native sources bundled together (easier to build/maintain)
- Optional feature keeps main backend fast to build
- Build scripts can reliably locate native sources
- Works with Cargo's workspace model

**Compared to alternatives:**
- ❌ All native in `src/`: Mixes concerns, confuses Cargo
- ❌ Native scattered across folders: Hard to build, maintain
- ❌ Required feature: Forces everyone to build C++ even if not used
