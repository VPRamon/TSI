# STARS FFI - C API for STARS Core

This library provides a stable C ABI wrapper around the STARS Core C++ scheduling library, enabling integration with other languages (primarily Rust).

## Overview

The STARS Core library is a C++ library for astronomical observation scheduling. This FFI layer:

- Exposes a C-compatible API (no C++ types cross the boundary)
- Uses JSON for data interchange (scheduling blocks, results, configuration)
- Provides opaque handles for stateful objects
- Catches all C++ exceptions and converts them to error codes
- Is designed for Rust integration via the `stars-core-sys` and `stars-core` crates

## Building

### Prerequisites

- CMake 3.14+
- C++17 compiler
- STARS Core library (built from source or installed)
- nlohmann_json library

### Build from source

```bash
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make
sudo make install
```

### Build options

| Option | Default | Description |
|--------|---------|-------------|
| `STARS_FFI_BUILD_STATIC` | OFF | Build static library instead of shared |
| `STARS_FFI_INSTALL_HEADERS` | ON | Install header files |
| `STARS_FFI_BUILD_CORE` | ON | Build STARS Core from source if found |
| `STARS_CORE_ROOT` | `../core` | Path to STARS Core source |

## API Overview

### Error Handling

All functions return a `StarsResult` structure with an error code and optional message:

```c
StarsResult result = stars_context_create(json, &handle);
if (result.code != STARS_OK) {
    printf("Error: %s\n", result.error_message);
    stars_free_result(&result);
}
```

### Context Management

```c
// Create context from JSON
StarsContextHandle ctx;
stars_context_create(config_json, &ctx);

// Or from file
stars_context_create_from_file("schedule.json", &ctx);

// Get execution period
char* period_json;
stars_context_get_execution_period(ctx, &period_json);
// ... use period_json ...
stars_free_string(period_json);

// Cleanup
stars_context_destroy(ctx);
```

### Scheduling Blocks

```c
// Load blocks from JSON
StarsBlocksHandle blocks;
stars_blocks_load_json(blocks_json, &blocks);

// Get count
size_t count;
stars_blocks_count(blocks, &count);

// Serialize back to JSON
char* json;
stars_blocks_to_json(blocks, &json);
stars_free_string(json);

// Cleanup
stars_blocks_destroy(blocks);
```

### Prescheduler (Possible Periods)

```c
StarsPossiblePeriodsHandle periods;
stars_compute_possible_periods(ctx, blocks, &periods);

char* periods_json;
stars_possible_periods_to_json(periods, &periods_json);
// ... use periods_json ...
stars_free_string(periods_json);

stars_possible_periods_destroy(periods);
```

### Scheduling

```c
StarsSchedulingParams params = stars_scheduling_params_default();
params.algorithm = STARS_SCHEDULER_HYBRID_ACCUMULATIVE;

StarsScheduleHandle schedule;
stars_run_scheduler(ctx, blocks, periods, params, &schedule);

// Get results
char* schedule_json;
stars_schedule_to_json(schedule, &schedule_json);

// Get statistics
char* stats_json;
stars_schedule_get_stats(schedule, &stats_json);

stars_free_string(schedule_json);
stars_free_string(stats_json);
stars_schedule_destroy(schedule);
```

### Full Pipeline

For convenience, run the entire pipeline in one call:

```c
StarsSchedulingParams params = stars_scheduling_params_default();
char* result_json;

stars_run_full_pipeline(input_json, params, &result_json);
// or
stars_run_pipeline_from_file("schedule.json", params, &result_json);

stars_free_string(result_json);
```

## Memory Management

- All strings returned by this library must be freed with `stars_free_string()`
- All handles must be destroyed with their corresponding `stars_*_destroy()` function
- Passing NULL to destroy functions is safe (no-op)
- Result structures should be cleaned up with `stars_free_result()` if they contain error messages

## Thread Safety

- Each handle should only be used from one thread at a time
- Different handles can be used concurrently from different threads
- Error messages are stored thread-locally

## Integration with Rust

This library is designed to be used via the Rust crates:

- `stars-core-sys`: Raw FFI bindings
- `stars-core`: Safe, ergonomic Rust API

See the crates in `/backend/crates/` for details.

## JSON Formats

### Configuration

```json
{
    "instrument": { /* STARS instrument config */ },
    "executionPeriod": {
        "begin": "2024-01-01T00:00:00",
        "end": "2024-12-31T23:59:59"
    },
    "observatory": "Optional observatory name"
}
```

### Scheduling Blocks

```json
{
    "schedulingBlocks": [
        {
            "stars::scheduling_blocks::ObservationTask": {
                "name": "Target1",
                "priority": 1.0,
                "duration": { "days": 0, "hours": 1, "minutes": 0, "seconds": 0 },
                "targetCoordinates": { /* coordinates */ }
            }
        }
    ]
}
```

### Schedule Result

```json
{
    "units": [
        {
            "task_id": "uuid",
            "task_name": "Target1",
            "begin": "2024-01-15T20:00:00",
            "end": "2024-01-15T21:00:00"
        }
    ],
    "fitness": 0.95,
    "scheduled_count": 42,
    "unscheduled": [
        { "id": "uuid", "name": "Target99" }
    ],
    "unscheduled_count": 3
}
```

## License

MIT License - see LICENSE file.
