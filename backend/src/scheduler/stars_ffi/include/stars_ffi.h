/*
 * STARS Core FFI - C API for Rust bindings
 * 
 * This header defines a stable C ABI wrapper around the STARS Core C++ scheduling library,
 * allowing dynamic modeling of scheduling blocks and running scheduling simulations from Rust.
 * 
 * Design principles:
 * - All functions use C types only (no C++ types across boundary)
 * - JSON strings for data interchange (scheduling blocks, results, errors)
 * - Opaque handles for stateful objects (context, blocks collection)
 * - All memory allocated by this library must be freed via stars_free_*
 * - No exceptions cross the FFI boundary (all caught and converted to error codes)
 */

#ifndef STARS_FFI_H
#define STARS_FFI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Error Handling
 * ============================================================================ */

/**
 * Error codes returned by FFI functions
 */
typedef enum StarsErrorCode {
    STARS_OK = 0,
    STARS_ERROR_NULL_POINTER = 1,
    STARS_ERROR_INVALID_JSON = 2,
    STARS_ERROR_SERIALIZATION = 3,
    STARS_ERROR_DESERIALIZATION = 4,
    STARS_ERROR_INVALID_HANDLE = 5,
    STARS_ERROR_SCHEDULING_FAILED = 6,
    STARS_ERROR_PRESCHEDULER_FAILED = 7,
    STARS_ERROR_IO = 8,
    STARS_ERROR_UNKNOWN = 99
} StarsErrorCode;

/**
 * Result structure containing error code and optional message
 */
typedef struct StarsResult {
    StarsErrorCode code;
    char* error_message;  /* Null if no error, must be freed with stars_free_string */
} StarsResult;

/**
 * Get the last error message (thread-local)
 * @return Error message string, or NULL if no error. Caller must NOT free.
 */
const char* stars_get_last_error(void);

/**
 * Clear the last error (thread-local)
 */
void stars_clear_error(void);

/* ============================================================================
 * Memory Management
 * ============================================================================ */

/**
 * Free a string allocated by this library
 * @param str String to free (safe to pass NULL)
 */
void stars_free_string(char* str);

/**
 * Free a result structure's error message
 * @param result Result to clean up (safe to pass NULL or result with NULL message)
 */
void stars_free_result(StarsResult* result);

/* ============================================================================
 * Opaque Handle Types
 * ============================================================================ */

/** Opaque handle to a STARS context (holds instrument, execution period, etc.) */
typedef struct StarsContext* StarsContextHandle;

/** Opaque handle to a collection of scheduling blocks */
typedef struct StarsBlocksCollection* StarsBlocksHandle;

/** Opaque handle to computed possible periods map */
typedef struct StarsPossiblePeriods* StarsPossiblePeriodsHandle;

/** Opaque handle to a schedule result */
typedef struct StarsSchedule* StarsScheduleHandle;

/* ============================================================================
 * Context Management
 * ============================================================================ */

/**
 * Create a new STARS context from JSON configuration
 * 
 * The JSON should contain:
 * - "instrument": instrument configuration
 * - "executionPeriod": { "begin": "ISO datetime", "end": "ISO datetime" }
 * - "observatory": observatory name (optional)
 * 
 * @param config_json JSON string with configuration
 * @param out_handle Output handle (set on success)
 * @return Result with error code
 */
StarsResult stars_context_create(const char* config_json, StarsContextHandle* out_handle);

/**
 * Create a context from a schedule JSON file path
 * 
 * @param file_path Path to schedule JSON file
 * @param out_handle Output handle (set on success)
 * @return Result with error code
 */
StarsResult stars_context_create_from_file(const char* file_path, StarsContextHandle* out_handle);

/**
 * Destroy a STARS context and free all resources
 * @param handle Context handle (safe to pass NULL)
 */
void stars_context_destroy(StarsContextHandle handle);

/**
 * Get the execution period from context as JSON
 * @param handle Context handle
 * @param out_json Output JSON string (caller must free with stars_free_string)
 * @return Result with error code
 */
StarsResult stars_context_get_execution_period(StarsContextHandle handle, char** out_json);

/* ============================================================================
 * Scheduling Blocks Management
 * ============================================================================ */

/**
 * Load scheduling blocks from JSON string
 * 
 * The JSON should be an array of scheduling block objects, where each block
 * has a type identifier that maps to a registered STARS block type.
 * 
 * @param json JSON string containing scheduling blocks array
 * @param out_handle Output handle to blocks collection
 * @return Result with error code
 */
StarsResult stars_blocks_load_json(const char* json, StarsBlocksHandle* out_handle);

/**
 * Load scheduling blocks from a schedule JSON file
 * 
 * @param file_path Path to schedule JSON file
 * @param out_handle Output handle to blocks collection
 * @return Result with error code
 */
StarsResult stars_blocks_load_file(const char* file_path, StarsBlocksHandle* out_handle);

/**
 * Serialize scheduling blocks to JSON string
 * 
 * @param handle Blocks collection handle
 * @param out_json Output JSON string (caller must free with stars_free_string)
 * @return Result with error code
 */
StarsResult stars_blocks_to_json(StarsBlocksHandle handle, char** out_json);

/**
 * Get the number of blocks in collection
 * 
 * @param handle Blocks collection handle
 * @param out_count Output count
 * @return Result with error code
 */
StarsResult stars_blocks_count(StarsBlocksHandle handle, size_t* out_count);

/**
 * Get a single block by index as JSON
 * 
 * @param handle Blocks collection handle
 * @param index Block index (0-based)
 * @param out_json Output JSON string (caller must free with stars_free_string)
 * @return Result with error code
 */
StarsResult stars_blocks_get_at(StarsBlocksHandle handle, size_t index, char** out_json);

/**
 * Destroy a blocks collection and free all resources
 * @param handle Blocks handle (safe to pass NULL)
 */
void stars_blocks_destroy(StarsBlocksHandle handle);

/* ============================================================================
 * Prescheduler (Possible Periods Computation)
 * ============================================================================ */

/**
 * Compute possible observation periods for scheduling blocks
 * 
 * This runs the STARS prescheduler which computes when each task can be observed
 * given instrument constraints, astronomical conditions, and time windows.
 * 
 * @param ctx Context handle (provides instrument and execution period)
 * @param blocks Blocks collection handle
 * @param out_handle Output handle to possible periods map
 * @return Result with error code
 */
StarsResult stars_compute_possible_periods(
    StarsContextHandle ctx,
    StarsBlocksHandle blocks,
    StarsPossiblePeriodsHandle* out_handle
);

/**
 * Export possible periods to JSON string
 * 
 * @param handle Possible periods handle
 * @param out_json Output JSON string (caller must free with stars_free_string)
 * @return Result with error code
 */
StarsResult stars_possible_periods_to_json(StarsPossiblePeriodsHandle handle, char** out_json);

/**
 * Destroy possible periods and free resources
 * @param handle Possible periods handle (safe to pass NULL)
 */
void stars_possible_periods_destroy(StarsPossiblePeriodsHandle handle);

/* ============================================================================
 * Scheduling Algorithm
 * ============================================================================ */

/**
 * Scheduling algorithm type
 */
typedef enum StarsSchedulerType {
    STARS_SCHEDULER_ACCUMULATIVE = 0,
    STARS_SCHEDULER_HYBRID_ACCUMULATIVE = 1
} StarsSchedulerType;

/**
 * Scheduling parameters
 */
typedef struct StarsSchedulingParams {
    StarsSchedulerType algorithm;
    uint32_t max_iterations;      /* 0 = default */
    double time_limit_seconds;    /* 0 = no limit */
    int32_t seed;                 /* Random seed, -1 = random */
} StarsSchedulingParams;

/**
 * Default scheduling parameters
 */
StarsSchedulingParams stars_scheduling_params_default(void);

/**
 * Run the scheduling algorithm
 * 
 * @param ctx Context handle
 * @param blocks Blocks collection handle
 * @param possible_periods Pre-computed possible periods (can be NULL to compute internally)
 * @param params Scheduling parameters
 * @param out_handle Output schedule handle
 * @return Result with error code
 */
StarsResult stars_run_scheduler(
    StarsContextHandle ctx,
    StarsBlocksHandle blocks,
    StarsPossiblePeriodsHandle possible_periods,
    StarsSchedulingParams params,
    StarsScheduleHandle* out_handle
);

/**
 * Export schedule to JSON string
 * 
 * @param handle Schedule handle
 * @param out_json Output JSON string (caller must free with stars_free_string)
 * @return Result with error code
 */
StarsResult stars_schedule_to_json(StarsScheduleHandle handle, char** out_json);

/**
 * Get schedule statistics as JSON
 * 
 * Returns: { "scheduled_count": N, "unscheduled_count": M, "fitness": F, ... }
 * 
 * @param handle Schedule handle
 * @param out_json Output JSON string (caller must free with stars_free_string)
 * @return Result with error code
 */
StarsResult stars_schedule_get_stats(StarsScheduleHandle handle, char** out_json);

/**
 * Destroy schedule and free resources
 * @param handle Schedule handle (safe to pass NULL)
 */
void stars_schedule_destroy(StarsScheduleHandle handle);

/* ============================================================================
 * Full Pipeline (Convenience)
 * ============================================================================ */

/**
 * Run the full scheduling pipeline: load → prescheduler → scheduler → export
 * 
 * This is a convenience function that runs the entire pipeline in one call.
 * 
 * @param input_json Input JSON containing blocks, instrument, execution period
 * @param params Scheduling parameters
 * @param out_result_json Output JSON with full results (caller must free)
 * @return Result with error code
 */
StarsResult stars_run_full_pipeline(
    const char* input_json,
    StarsSchedulingParams params,
    char** out_result_json
);

/**
 * Run scheduling pipeline from file
 * 
 * @param input_file_path Path to input schedule JSON file
 * @param params Scheduling parameters
 * @param out_result_json Output JSON with full results (caller must free)
 * @return Result with error code
 */
StarsResult stars_run_pipeline_from_file(
    const char* input_file_path,
    StarsSchedulingParams params,
    char** out_result_json
);

/* ============================================================================
 * Version Info
 * ============================================================================ */

/**
 * Get the version string of the STARS FFI library
 * @return Static version string (do not free)
 */
const char* stars_ffi_version(void);

/**
 * Get the version of the underlying STARS Core library
 * @return Static version string (do not free)
 */
const char* stars_core_version(void);

#ifdef __cplusplus
}
#endif

#endif /* STARS_FFI_H */
