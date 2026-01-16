/*
 * STARS FFI - C API for STARS Core scheduling library
 *
 * This header provides a C-compatible interface to the STARS Core C++ library,
 * allowing it to be called from Rust or other languages via FFI.
 */

#ifndef STARS_FFI_H
#define STARS_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Error Codes
 * ============================================================================ */

typedef enum {
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

/* Result structure returned by most functions */
typedef struct {
    StarsErrorCode code;
    char* error_message;  /* Owned by caller - free with stars_free_string */
} StarsResult;

/* ============================================================================
 * Opaque Handle Types
 * ============================================================================ */

typedef struct StarsContext StarsContext;
typedef struct StarsBlocksCollection StarsBlocksCollection;
typedef struct StarsPossiblePeriods StarsPossiblePeriods;
typedef struct StarsSchedule StarsSchedule;

typedef StarsContext* StarsContextHandle;
typedef StarsBlocksCollection* StarsBlocksHandle;
typedef StarsPossiblePeriods* StarsPossiblePeriodsHandle;
typedef StarsSchedule* StarsScheduleHandle;

/* ============================================================================
 * Scheduling Parameters
 * ============================================================================ */

typedef enum {
    STARS_SCHEDULER_ACCUMULATIVE = 0,
    STARS_SCHEDULER_HYBRID_ACCUMULATIVE = 1
} StarsSchedulerType;

typedef struct {
    StarsSchedulerType algorithm;
    uint32_t max_iterations;
    double time_limit_seconds;
    int32_t seed;
} StarsSchedulingParams;

/* ============================================================================
 * Error Handling
 * ============================================================================ */

/* Get last error message (static buffer, do not free) */
const char* stars_get_last_error(void);

/* Clear the last error */
void stars_clear_error(void);

/* ============================================================================
 * Memory Management
 * ============================================================================ */

/* Free a string allocated by STARS FFI */
void stars_free_string(char* str);

/* Free a result structure */
void stars_free_result(StarsResult* result);

/* ============================================================================
 * Context Management
 * ============================================================================ */

/* Create context from JSON configuration string */
StarsResult stars_context_create(
    const char* config_json,
    StarsContextHandle* out_handle
);

/* Create context from a JSON file */
StarsResult stars_context_create_from_file(
    const char* file_path,
    StarsContextHandle* out_handle
);

/* Destroy a context */
void stars_context_destroy(StarsContextHandle handle);

/* Get execution period as JSON */
StarsResult stars_context_get_execution_period(
    StarsContextHandle handle,
    char** out_json
);

/* ============================================================================
 * Scheduling Blocks Management
 * ============================================================================ */

/* Load blocks from JSON string */
StarsResult stars_blocks_load_json(
    const char* json,
    StarsBlocksHandle* out_handle
);

/* Load blocks from a file */
StarsResult stars_blocks_load_file(
    const char* file_path,
    StarsBlocksHandle* out_handle
);

/* Serialize blocks to JSON */
StarsResult stars_blocks_to_json(
    StarsBlocksHandle handle,
    char** out_json
);

/* Get number of blocks */
StarsResult stars_blocks_count(
    StarsBlocksHandle handle,
    size_t* out_count
);

/* Get a single block by index as JSON */
StarsResult stars_blocks_get_at(
    StarsBlocksHandle handle,
    size_t index,
    char** out_json
);

/* Destroy blocks collection */
void stars_blocks_destroy(StarsBlocksHandle handle);

/* ============================================================================
 * Prescheduler
 * ============================================================================ */

/* Compute possible observation periods */
StarsResult stars_compute_possible_periods(
    StarsContextHandle ctx,
    StarsBlocksHandle blocks,
    StarsPossiblePeriodsHandle* out_handle
);

/* Export possible periods as JSON */
StarsResult stars_possible_periods_to_json(
    StarsPossiblePeriodsHandle handle,
    char** out_json
);

/* Destroy possible periods */
void stars_possible_periods_destroy(StarsPossiblePeriodsHandle handle);

/* ============================================================================
 * Scheduling Algorithm
 * ============================================================================ */

/* Get default scheduling parameters */
StarsSchedulingParams stars_scheduling_params_default(void);

/* Run the scheduler */
StarsResult stars_run_scheduler(
    StarsContextHandle ctx,
    StarsBlocksHandle blocks,
    StarsPossiblePeriodsHandle possible_periods,  /* Can be NULL */
    StarsSchedulingParams params,
    StarsScheduleHandle* out_handle
);

/* Export schedule as JSON */
StarsResult stars_schedule_to_json(
    StarsScheduleHandle handle,
    char** out_json
);

/* Get schedule statistics as JSON */
StarsResult stars_schedule_get_stats(
    StarsScheduleHandle handle,
    char** out_json
);

/* Destroy schedule */
void stars_schedule_destroy(StarsScheduleHandle handle);

/* ============================================================================
 * Full Pipeline
 * ============================================================================ */

/* Run full pipeline from JSON (config + blocks combined) */
StarsResult stars_run_full_pipeline(
    const char* input_json,
    StarsSchedulingParams params,
    char** out_result_json
);

/* Run full pipeline from a file */
StarsResult stars_run_pipeline_from_file(
    const char* input_file_path,
    StarsSchedulingParams params,
    char** out_result_json
);

/* ============================================================================
 * Version Info
 * ============================================================================ */

/* Get FFI library version */
const char* stars_ffi_version(void);

/* Get STARS Core version */
const char* stars_core_version(void);

#ifdef __cplusplus
}
#endif

#endif /* STARS_FFI_H */
