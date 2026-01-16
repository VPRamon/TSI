/*
 * STARS FFI - C API implementation for STARS Core scheduling library
 */

#include "stars_ffi.h"

// STARS Core headers - use paths relative to src/
#include <stars/scheduling_blocks/observation_task.h>
#include <stars/scheduling_blocks/sequence.h>
#include <stars/scheduling_blocks/scheduling_block.h>
#include <stars/scheduling_blocks/task.h>
#include <stars/coordinates/multidimensional/const_equatorial.h>
#include <stars/coordinates/const_geographic.h>
#include <stars/coordinates/unidimensional/latitude.h>
#include <stars/coordinates/unidimensional/longitude.h>
#include <stars/scheduler/scheduling_algorithms/accumulative/accumulative_scheduling_algorithm.h>
#include <stars/scheduler/scheduling_algorithms/hybrid_accumulative/hybrid_accumulative_scheduling_algorithm.h>
#include <stars/scheduler/prescheduler/prescheduler.h>
#include <stars/scheduler/instrument/instrument.h>
#include <stars/scheduler/schedule/schedule.h>
#include <stars/scheduler/schedule/schedule_unit.h>
#include <stars/time/period.h>
#include <stars/time/duration.h>
#include <stars/time/time_utc.h>

#include <nlohmann/json.hpp>

#include <cstring>
#include <fstream>
#include <memory>
#include <sstream>
#include <string>
#include <vector>
#include <set>
#include <iomanip>

/* ============================================================================
 * Version Info
 * ============================================================================ */

#define STARS_FFI_VERSION "0.1.0"

/* Namespace aliases */
namespace sb = stars::scheduling_blocks;
namespace sched = stars::scheduler;
namespace algo = stars::scheduler::algorithms;
namespace instr = stars::scheduler::instruments;
namespace coord = stars::coordinates;

/* ============================================================================
 * Internal State
 * ============================================================================ */

namespace {
    thread_local std::string g_last_error;
    
    void set_error(const std::string& msg) {
        g_last_error = msg;
    }
    
    void clear_error() {
        g_last_error.clear();
    }
    
    char* duplicate_string(const std::string& s) {
        char* result = static_cast<char*>(malloc(s.size() + 1));
        if (result) {
            std::memcpy(result, s.c_str(), s.size() + 1);
        }
        return result;
    }
    
    StarsResult make_ok() {
        return StarsResult{STARS_OK, nullptr};
    }
    
    StarsResult make_error(StarsErrorCode code, const std::string& msg) {
        set_error(msg);
        return StarsResult{code, duplicate_string(msg)};
    }
    
    // Format TimeUTC to ISO string
    std::string format_datetime(const stars::time::TimeUTC& time) {
        auto date = time.GetDate();
        auto duration = time.GetDuration();
        
        std::ostringstream oss;
        oss << std::setfill('0') 
            << std::setw(4) << date.Year() << "-"
            << std::setw(2) << date.Month() << "-"
            << std::setw(2) << date.Day() << "T"
            << std::setw(2) << duration.Hours() << ":"
            << std::setw(2) << duration.Minutes() << ":"
            << std::setw(2) << duration.Seconds() << "Z";
        return oss.str();
    }
}

/* ============================================================================
 * Internal Handle Wrappers
 * ============================================================================ */

struct StarsContext {
    std::shared_ptr<instr::Instrument> instrument;
    stars::time::Period executionPeriod;
    std::string config_json;
};

struct StarsBlocksCollection {
    sb::SchedulingBlock::ConstBlocks blocks;
    std::string source_json;
};

struct StarsPossiblePeriods {
    // After prescheduler runs, the PossiblePeriodsMap is stored in the instrument
    // This handle just marks that computation was done and stores the map reference
    bool computed = false;
};

struct StarsSchedule {
    sched::schedule::Schedule::ConstPointerType schedule;
    algo::SchedulingAlgorithm::ConstBlocksSet unscheduled;
    sched::FailInformation failInfo;
    size_t total_blocks;
    double fitness;
};

/* ============================================================================
 * Error Handling
 * ============================================================================ */

extern "C" {

const char* stars_get_last_error(void) {
    return g_last_error.empty() ? nullptr : g_last_error.c_str();
}

void stars_clear_error(void) {
    clear_error();
}

/* ============================================================================
 * Memory Management
 * ============================================================================ */

void stars_free_string(char* str) {
    free(str);
}

void stars_free_result(StarsResult* result) {
    if (result && result->error_message) {
        free(result->error_message);
        result->error_message = nullptr;
    }
}

/* ============================================================================
 * Context Management
 * ============================================================================ */

StarsResult stars_context_create(
    const char* config_json,
    StarsContextHandle* out_handle
) {
    if (!config_json || !out_handle) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        auto ctx = new StarsContext();
        ctx->config_json = config_json;
        
        nlohmann::json j = nlohmann::json::parse(config_json);
        
        // Parse instrument
        if (!j.contains("instrument")) {
            delete ctx;
            return make_error(STARS_ERROR_INVALID_JSON, "Missing 'instrument' in config");
        }
        
        auto& instr_json = j["instrument"];
        double lat = instr_json.value("location", nlohmann::json::object()).value("latitude", 0.0);
        double lon = instr_json.value("location", nlohmann::json::object()).value("longitude", 0.0);
        double alt = instr_json.value("location", nlohmann::json::object()).value("altitude", 0.0);
        
        coord::ConstGeographic location(
            coord::Latitude(lat),
            coord::Longitude(lon),
            alt
        );
        ctx->instrument = std::make_shared<instr::Instrument>(1, location);
        
        // Parse execution period
        if (!j.contains("executionPeriod")) {
            delete ctx;
            return make_error(STARS_ERROR_INVALID_JSON, "Missing 'executionPeriod' in config");
        }
        
        auto& period_json = j["executionPeriod"];
        std::string begin_str = period_json.value("begin", "");
        std::string end_str = period_json.value("end", "");
        
        if (begin_str.empty() || end_str.empty()) {
            delete ctx;
            return make_error(STARS_ERROR_INVALID_JSON, "Invalid executionPeriod: missing begin or end");
        }
        
        stars::time::TimeUTC begin(begin_str);
        stars::time::TimeUTC end(end_str);
        ctx->executionPeriod = stars::time::Period(begin, end);
        
        *out_handle = ctx;
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_INVALID_JSON, e.what());
    }
}

StarsResult stars_context_create_from_file(
    const char* file_path,
    StarsContextHandle* out_handle
) {
    if (!file_path || !out_handle) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        std::ifstream ifs(file_path);
        if (!ifs) {
            return make_error(STARS_ERROR_IO, std::string("Cannot open file: ") + file_path);
        }
        
        std::stringstream buffer;
        buffer << ifs.rdbuf();
        return stars_context_create(buffer.str().c_str(), out_handle);
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_IO, e.what());
    }
}

void stars_context_destroy(StarsContextHandle handle) {
    delete handle;
}

StarsResult stars_context_get_execution_period(
    StarsContextHandle handle,
    char** out_json
) {
    if (!handle || !out_json) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        const auto& period = handle->executionPeriod;
        
        nlohmann::json j;
        j["begin"] = format_datetime(period.BeginTime());
        j["end"] = format_datetime(period.EndTime());
        
        // Calculate duration in days
        auto duration = period.GetDuration();
        double days = static_cast<double>(duration.Seconds()) / 86400.0;
        j["duration_days"] = days;
        
        *out_json = duplicate_string(j.dump());
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_SERIALIZATION, e.what());
    }
}

/* ============================================================================
 * Scheduling Blocks Management
 * ============================================================================ */

StarsResult stars_blocks_load_json(
    const char* json,
    StarsBlocksHandle* out_handle
) {
    if (!json || !out_handle) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        auto blocks = new StarsBlocksCollection();
        blocks->source_json = json;
        
        nlohmann::json j = nlohmann::json::parse(json);
        
        // Find scheduling blocks array
        nlohmann::json blocks_array;
        if (j.contains("schedulingBlocks")) {
            blocks_array = j["schedulingBlocks"];
        } else if (j.is_array()) {
            blocks_array = j;
        } else {
            delete blocks;
            return make_error(STARS_ERROR_INVALID_JSON, "No scheduling blocks found in JSON");
        }
        
        if (!blocks_array.is_array()) {
            delete blocks;
            return make_error(STARS_ERROR_INVALID_JSON, "schedulingBlocks must be an array");
        }
        
        for (const auto& block_json : blocks_array) {
            // Parse each block - look for ObservationTask type
            for (auto& [key, value] : block_json.items()) {
                if (key.find("ObservationTask") != std::string::npos) {
                    std::string name = value.value("name", "unnamed");
                    double priority = value.value("priority", 1.0);
                    
                    // Parse duration
                    int hours = 0, minutes = 0, seconds = 0;
                    if (value.contains("duration")) {
                        auto& dur = value["duration"];
                        hours = dur.value("hours", 0);
                        minutes = dur.value("minutes", 0);
                        seconds = dur.value("seconds", 0);
                    }
                    stars::time::Duration duration(hours, minutes, seconds, 0);
                    
                    // Parse target coordinates if available
                    coord::Coordinates::PointerType target;
                    if (value.contains("targetCoordinates")) {
                        auto& tc = value["targetCoordinates"];
                        double ra_deg = tc.value("ra", 0.0);
                        double dec_deg = tc.value("dec", 0.0);
                        
                        target = std::make_shared<coord::ConstEquatorial>(
                            coord::RightAscension(ra_deg),
                            coord::Declination(dec_deg)
                        );
                    } else {
                        // Default target at origin
                        target = std::make_shared<coord::ConstEquatorial>(
                            coord::RightAscension(0.0),
                            coord::Declination(0.0)
                        );
                    }
                    
                    auto task = sb::ObservationTask::Create(name, priority, duration, target);
                    blocks->blocks.push_back(task);
                }
            }
        }
        
        *out_handle = blocks;
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_INVALID_JSON, e.what());
    }
}

StarsResult stars_blocks_load_file(
    const char* file_path,
    StarsBlocksHandle* out_handle
) {
    if (!file_path || !out_handle) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        std::ifstream ifs(file_path);
        if (!ifs) {
            return make_error(STARS_ERROR_IO, std::string("Cannot open file: ") + file_path);
        }
        
        std::stringstream buffer;
        buffer << ifs.rdbuf();
        return stars_blocks_load_json(buffer.str().c_str(), out_handle);
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_IO, e.what());
    }
}

StarsResult stars_blocks_to_json(
    StarsBlocksHandle handle,
    char** out_json
) {
    if (!handle || !out_json) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json j = nlohmann::json::array();
        for (const auto& block : handle->blocks) {
            nlohmann::json block_json;
            block_json["name"] = block->GetName();
            
            auto task = std::dynamic_pointer_cast<const stars::scheduling_blocks::Task>(block);
            if (task) {
                block_json["priority"] = task->GetPriority();
            } else {
                block_json["priority"] = 0.0;
            }
            j.push_back(block_json);
        }
        
        *out_json = duplicate_string(j.dump());
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_SERIALIZATION, e.what());
    }
}

StarsResult stars_blocks_count(
    StarsBlocksHandle handle,
    size_t* out_count
) {
    if (!handle || !out_count) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    *out_count = handle->blocks.size();
    return make_ok();
}

StarsResult stars_blocks_get_at(
    StarsBlocksHandle handle,
    size_t index,
    char** out_json
) {
    if (!handle || !out_json) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    if (index >= handle->blocks.size()) {
        return make_error(STARS_ERROR_INVALID_HANDLE, "Index out of bounds");
    }
    
    try {
        auto it = handle->blocks.begin();
        std::advance(it, index);
        const auto& block = *it;

        nlohmann::json j;
        j["name"] = block->GetName();
        
        auto task = std::dynamic_pointer_cast<const stars::scheduling_blocks::Task>(block);
        if (task) {
            j["priority"] = task->GetPriority();
        } else {
            j["priority"] = 0.0;
        }
        
        *out_json = duplicate_string(j.dump());
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_SERIALIZATION, e.what());
    }
}

void stars_blocks_destroy(StarsBlocksHandle handle) {
    delete handle;
}

/* ============================================================================
 * Prescheduler
 * ============================================================================ */

StarsResult stars_compute_possible_periods(
    StarsContextHandle ctx,
    StarsBlocksHandle blocks,
    StarsPossiblePeriodsHandle* out_handle
) {
    if (!ctx || !blocks || !out_handle) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        // Run prescheduler - this modifies blocks in place
        sched::prescheduler::ComputePeriods(
            *ctx->instrument,
            blocks->blocks,
            ctx->executionPeriod
        );
        
        auto periods = new StarsPossiblePeriods();
        periods->computed = true;
        
        *out_handle = periods;
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_PRESCHEDULER_FAILED, e.what());
    }
}

StarsResult stars_possible_periods_to_json(
    StarsPossiblePeriodsHandle handle,
    char** out_json
) {
    if (!handle || !out_json) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        // Return empty array - periods are stored in blocks
        nlohmann::json j = nlohmann::json::array();
        *out_json = duplicate_string(j.dump());
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_SERIALIZATION, e.what());
    }
}

void stars_possible_periods_destroy(StarsPossiblePeriodsHandle handle) {
    delete handle;
}

/* ============================================================================
 * Scheduling Algorithm
 * ============================================================================ */

StarsSchedulingParams stars_scheduling_params_default(void) {
    return StarsSchedulingParams{
        STARS_SCHEDULER_ACCUMULATIVE,
        0,      // max_iterations (0 = default)
        0.0,    // time_limit_seconds (0 = no limit)
        -1      // seed (-1 = random)
    };
}

StarsResult stars_run_scheduler(
    StarsContextHandle ctx,
    StarsBlocksHandle blocks,
    StarsPossiblePeriodsHandle possible_periods,
    StarsSchedulingParams params,
    StarsScheduleHandle* out_handle
) {
    if (!ctx || !blocks || !out_handle) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        // If periods weren't computed, compute them now
        if (!possible_periods || !possible_periods->computed) {
            sched::prescheduler::ComputePeriods(
                *ctx->instrument,
                blocks->blocks,
                ctx->executionPeriod
            );
        }
        
        // Create scheduler based on algorithm type
        sched::schedule::Schedule::ConstPointerType result_schedule;
        algo::SchedulingAlgorithm::ConstBlocksSet unscheduled;
        sched::FailInformation failInfo;
        
        if (params.algorithm == STARS_SCHEDULER_HYBRID_ACCUMULATIVE) {
            // Hybrid uses multiple threads
            size_t numThreads = 4;  // Default number of threads
            algo::hybrid_accumulative::HybridAccumulativeSchedulingAlgorithm::Configuration config(
                numThreads,
                50,   // settlerIterations
                false // canReattemptUnplanned
            );
            algo::hybrid_accumulative::HybridAccumulativeSchedulingAlgorithm scheduler(config);
            
            auto [schedule, unplanned, fail] = scheduler.Schedule(
                blocks->blocks,
                ctx->instrument,
                ctx->executionPeriod
            );
            
            result_schedule = schedule;
            unscheduled = unplanned;
            failInfo = fail;
        } else {
            algo::accumulative::AccumulativeSchedulingAlgorithm::Configuration config;
            algo::accumulative::AccumulativeSchedulingAlgorithm scheduler(config);
            
            auto [schedule, unplanned, fail] = scheduler.Schedule(
                blocks->blocks,
                ctx->instrument,
                ctx->executionPeriod
            );
            
            result_schedule = schedule;
            unscheduled = unplanned;
            failInfo = fail;
        }
        
        auto schedule = new StarsSchedule();
        schedule->schedule = result_schedule;
        schedule->unscheduled = unscheduled;
        schedule->failInfo = failInfo;
        schedule->total_blocks = blocks->blocks.size();
        
        // Calculate fitness as scheduling rate
        size_t scheduled = blocks->blocks.size() - unscheduled.size();
        schedule->fitness = blocks->blocks.size() > 0 
            ? static_cast<double>(scheduled) / blocks->blocks.size() 
            : 0.0;
        
        *out_handle = schedule;
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_SCHEDULING_FAILED, e.what());
    }
}

StarsResult stars_schedule_to_json(
    StarsScheduleHandle handle,
    char** out_json
) {
    if (!handle || !out_json) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json j;
        
        // Scheduled units
        nlohmann::json units = nlohmann::json::array();
        if (handle->schedule) {
            for (const auto& unit : handle->schedule->GetUnits()) {
                nlohmann::json u;
                auto task = unit->GetTask();
                auto period = unit->GetPeriod();
                u["task_id"] = task->GetName();
                u["task_name"] = task->GetName();
                u["begin"] = format_datetime(period.BeginTime());
                u["end"] = format_datetime(period.EndTime());
                units.push_back(u);
            }
        }
        j["units"] = units;
        
        // Unscheduled blocks
        nlohmann::json unscheduled = nlohmann::json::array();
        for (const auto& block : handle->unscheduled) {
            nlohmann::json b;
            b["id"] = block->GetName();
            b["name"] = block->GetName();
            unscheduled.push_back(b);
        }
        j["unscheduled"] = unscheduled;
        
        j["fitness"] = handle->fitness;
        j["scheduled_count"] = units.size();
        j["unscheduled_count"] = handle->unscheduled.size();
        
        *out_json = duplicate_string(j.dump());
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_SERIALIZATION, e.what());
    }
}

StarsResult stars_schedule_get_stats(
    StarsScheduleHandle handle,
    char** out_json
) {
    if (!handle || !out_json) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json j;
        
        size_t scheduled = handle->total_blocks - handle->unscheduled.size();
        size_t unscheduled = handle->unscheduled.size();
        size_t total = handle->total_blocks;
        
        j["scheduled_count"] = scheduled;
        j["unscheduled_count"] = unscheduled;
        j["total_blocks"] = total;
        j["scheduling_rate"] = total > 0 ? static_cast<double>(scheduled) / total : 0.0;
        j["fitness"] = handle->fitness;
        
        *out_json = duplicate_string(j.dump());
        return make_ok();
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_SERIALIZATION, e.what());
    }
}

void stars_schedule_destroy(StarsScheduleHandle handle) {
    delete handle;
}

/* ============================================================================
 * Full Pipeline
 * ============================================================================ */

StarsResult stars_run_full_pipeline(
    const char* input_json,
    StarsSchedulingParams params,
    char** out_result_json
) {
    if (!input_json || !out_result_json) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    StarsContextHandle ctx = nullptr;
    StarsBlocksHandle blocks = nullptr;
    StarsPossiblePeriodsHandle periods = nullptr;
    StarsScheduleHandle schedule = nullptr;
    
    // Create context
    auto result = stars_context_create(input_json, &ctx);
    if (result.code != STARS_OK) {
        return result;
    }
    
    // Load blocks
    result = stars_blocks_load_json(input_json, &blocks);
    if (result.code != STARS_OK) {
        stars_context_destroy(ctx);
        return result;
    }
    
    // Compute possible periods
    result = stars_compute_possible_periods(ctx, blocks, &periods);
    if (result.code != STARS_OK) {
        stars_blocks_destroy(blocks);
        stars_context_destroy(ctx);
        return result;
    }
    
    // Run scheduler
    result = stars_run_scheduler(ctx, blocks, periods, params, &schedule);
    if (result.code != STARS_OK) {
        stars_possible_periods_destroy(periods);
        stars_blocks_destroy(blocks);
        stars_context_destroy(ctx);
        return result;
    }
    
    // Get result JSON
    result = stars_schedule_to_json(schedule, out_result_json);
    
    // Cleanup
    stars_schedule_destroy(schedule);
    stars_possible_periods_destroy(periods);
    stars_blocks_destroy(blocks);
    stars_context_destroy(ctx);
    
    return result;
}

StarsResult stars_run_pipeline_from_file(
    const char* input_file_path,
    StarsSchedulingParams params,
    char** out_result_json
) {
    if (!input_file_path || !out_result_json) {
        return make_error(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        std::ifstream ifs(input_file_path);
        if (!ifs) {
            return make_error(STARS_ERROR_IO, std::string("Cannot open file: ") + input_file_path);
        }
        
        std::stringstream buffer;
        buffer << ifs.rdbuf();
        return stars_run_full_pipeline(buffer.str().c_str(), params, out_result_json);
    } catch (const std::exception& e) {
        return make_error(STARS_ERROR_IO, e.what());
    }
}

/* ============================================================================
 * Version Info
 * ============================================================================ */

const char* stars_ffi_version(void) {
    return STARS_FFI_VERSION;
}

const char* stars_core_version(void) {
    // TODO: Get actual version from STARS Core when available
    return "1.0.0";
}

} // extern "C"
