/*
 * STARS Core FFI - Implementation
 * 
 * C++ implementation of the C ABI wrapper around STARS Core library.
 */

#include "stars_ffi.h"

#include <string>
#include <memory>
#include <optional>
#include <cstring>
#include <exception>
#include <mutex>
#include <thread>
#include <unordered_map>
#include <fstream>
#include <sstream>

// STARS Core includes
#include <stars/builders/json/schedule_json_loader.h>
#include <stars/builders/json/schedule_json_builder.h>
#include <stars/serialization/serializer.h>
#include <stars/serialization/deserializer.h>
#include <stars/serialization/archives/json/input_archive.h>
#include <stars/serialization/archives/json/output_archive.h>
#include <stars/scheduler/prescheduler/prescheduler.h>
#include <stars/scheduler/schedule/schedule.h>
#include <stars/scheduler/scheduling_algorithms/accumulative/accumulative_scheduling_algorithm.h>
#include <stars/scheduler/scheduling_algorithms/hybrid_accumulative/hybrid_accumulative_scheduling_algorithm.h>
#include <stars/scheduling_blocks/scheduling_block.h>
#include <stars/scheduling_blocks/sequence.h>
#include <stars/scheduling_blocks/observation_task.h>
#include <stars/scheduling_blocks/engineering_task.h>

#include <nlohmann/json.hpp>

/* ============================================================================
 * Internal Types and State
 * ============================================================================ */

namespace {

// Thread-local error storage
thread_local std::string g_last_error;

void set_error(const std::string& msg) {
    g_last_error = msg;
}

void clear_error_internal() {
    g_last_error.clear();
}

char* duplicate_string(const std::string& str) {
    char* result = static_cast<char*>(malloc(str.size() + 1));
    if (result) {
        memcpy(result, str.c_str(), str.size() + 1);
    }
    return result;
}

StarsResult make_result(StarsErrorCode code, const std::string& msg = "") {
    StarsResult result;
    result.code = code;
    result.error_message = msg.empty() ? nullptr : duplicate_string(msg);
    if (!msg.empty()) {
        set_error(msg);
    }
    return result;
}

StarsResult make_ok() {
    return make_result(STARS_OK);
}

} // anonymous namespace

/* ============================================================================
 * Internal Handle Structures
 * ============================================================================ */

struct StarsContext {
    stars::scheduler::instruments::Instrument::PointerType instrument;
    stars::time::Period executionPeriod;
    std::string observatoryName;
    nlohmann::json originalConfig;
    
    StarsContext() = default;
};

struct StarsBlocksCollection {
    stars::scheduling_blocks::SchedulingBlock::ConstBlocks blocks;
    
    StarsBlocksCollection() = default;
};

struct StarsPossiblePeriods {
    stars::constraints::PossiblePeriodsMap periods;
    std::unordered_map<std::string, std::string> blockNamesById;
    
    StarsPossiblePeriods() = default;
};

struct StarsSchedule {
    stars::scheduler::schedule::Schedule::ConstPointerType schedule;
    stars::scheduling_blocks::SchedulingBlock::ConstBlocks blocks;
    std::set<stars::scheduling_blocks::SchedulingBlock::ConstPointerType> unscheduled;
    stars::scheduler::instruments::Instrument::ConstPointerType instrument;
    double fitness = 0.0;
    
    StarsSchedule() = default;
};

/* ============================================================================
 * Error Handling Implementation
 * ============================================================================ */

extern "C" {

const char* stars_get_last_error(void) {
    return g_last_error.empty() ? nullptr : g_last_error.c_str();
}

void stars_clear_error(void) {
    clear_error_internal();
}

/* ============================================================================
 * Memory Management Implementation
 * ============================================================================ */

void stars_free_string(char* str) {
    if (str) {
        free(str);
    }
}

void stars_free_result(StarsResult* result) {
    if (result && result->error_message) {
        free(result->error_message);
        result->error_message = nullptr;
    }
}

/* ============================================================================
 * Context Management Implementation
 * ============================================================================ */

StarsResult stars_context_create(const char* config_json, StarsContextHandle* out_handle) {
    clear_error_internal();
    
    if (!config_json || !out_handle) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json config = nlohmann::json::parse(config_json);
        
        auto ctx = std::make_unique<StarsContext>();
        ctx->originalConfig = config;
        
        // Load instrument if present
        if (config.contains("instrument")) {
            stars::serialization::archives::json::InputArchive archive(config["instrument"]);
            stars::serialization::Deserializer deserializer(archive);
            ctx->instrument = std::make_shared<stars::scheduler::instruments::Instrument>(0);
            ctx->instrument->Load(deserializer);
        }
        
        // Load execution period
        if (config.contains("executionPeriod")) {
            auto& ep = config["executionPeriod"];
            // Parse ISO datetime strings to TimeUTC
            std::string beginStr = ep["begin"].get<std::string>();
            std::string endStr = ep["end"].get<std::string>();
            stars::time::TimeUTC begin(beginStr);
            stars::time::TimeUTC end(endStr);
            ctx->executionPeriod = stars::time::Period(begin, end);
        }
        
        // Load observatory name if present
        if (config.contains("observatory")) {
            ctx->observatoryName = config["observatory"].get<std::string>();
        }
        
        *out_handle = ctx.release();
        return make_ok();
        
    } catch (const nlohmann::json::parse_error& e) {
        return make_result(STARS_ERROR_INVALID_JSON, std::string("JSON parse error: ") + e.what());
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_DESERIALIZATION, std::string("Failed to create context: ") + e.what());
    } catch (...) {
        return make_result(STARS_ERROR_UNKNOWN, "Unknown error creating context");
    }
}

StarsResult stars_context_create_from_file(const char* file_path, StarsContextHandle* out_handle) {
    clear_error_internal();
    
    if (!file_path || !out_handle) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        stars::builders::json::ScheduleJsonLoader loader(file_path);
        
        auto ctx = std::make_unique<StarsContext>();
        ctx->originalConfig = loader.GetContent();
        ctx->instrument = loader.LoadInstrument();
        ctx->executionPeriod = loader.LoadExecutionPeriod();
        
        *out_handle = ctx.release();
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_IO, std::string("Failed to load file: ") + e.what());
    } catch (...) {
        return make_result(STARS_ERROR_UNKNOWN, "Unknown error loading file");
    }
}

void stars_context_destroy(StarsContextHandle handle) {
    delete handle;
}

StarsResult stars_context_get_execution_period(StarsContextHandle handle, char** out_json) {
    clear_error_internal();
    
    if (!handle || !out_json) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json result;
        result["begin"] = handle->executionPeriod.BeginTime().ToString();
        result["end"] = handle->executionPeriod.EndTime().ToString();
        result["duration_days"] = handle->executionPeriod.GetDuration().TotalHours() / 24.0;
        
        *out_json = duplicate_string(result.dump());
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_SERIALIZATION, std::string("Failed to serialize: ") + e.what());
    }
}

/* ============================================================================
 * Scheduling Blocks Implementation
 * ============================================================================ */

StarsResult stars_blocks_load_json(const char* json, StarsBlocksHandle* out_handle) {
    clear_error_internal();
    
    if (!json || !out_handle) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json data = nlohmann::json::parse(json);
        
        auto collection = std::make_unique<StarsBlocksCollection>();
        
        // Handle both array of blocks and object with "schedulingBlocks" key
        nlohmann::json blocksArray;
        if (data.is_array()) {
            blocksArray = data;
        } else if (data.contains("schedulingBlocks")) {
            blocksArray = data["schedulingBlocks"];
        } else {
            return make_result(STARS_ERROR_INVALID_JSON, "Expected array or object with 'schedulingBlocks' key");
        }
        
        for (const auto& blockJson : blocksArray) {
            stars::serialization::archives::json::InputArchive archive(blockJson);
            stars::serialization::Deserializer deserializer(archive);
            
            // The deserializer will use type registry to create the correct block type
            stars::scheduling_blocks::SchedulingBlock::PointerType block;
            deserializer >> STARS_DESERIALIZER_DECORATE(block);
            
            if (block) {
                collection->blocks.push_back(block);
            }
        }
        
        *out_handle = collection.release();
        return make_ok();
        
    } catch (const nlohmann::json::parse_error& e) {
        return make_result(STARS_ERROR_INVALID_JSON, std::string("JSON parse error: ") + e.what());
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_DESERIALIZATION, std::string("Failed to load blocks: ") + e.what());
    }
}

StarsResult stars_blocks_load_file(const char* file_path, StarsBlocksHandle* out_handle) {
    clear_error_internal();
    
    if (!file_path || !out_handle) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        stars::builders::json::ScheduleJsonLoader loader(file_path);
        
        auto collection = std::make_unique<StarsBlocksCollection>();
        collection->blocks = loader.LoadBlocks();
        
        *out_handle = collection.release();
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_IO, std::string("Failed to load file: ") + e.what());
    }
}

StarsResult stars_blocks_to_json(StarsBlocksHandle handle, char** out_json) {
    clear_error_internal();
    
    if (!handle || !out_json) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json result = stars::builders::json::ScheduleJsonBuilder::Build(handle->blocks);
        *out_json = duplicate_string(result.dump());
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_SERIALIZATION, std::string("Failed to serialize blocks: ") + e.what());
    }
}

StarsResult stars_blocks_count(StarsBlocksHandle handle, size_t* out_count) {
    clear_error_internal();
    
    if (!handle || !out_count) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    *out_count = handle->blocks.size();
    return make_ok();
}

StarsResult stars_blocks_get_at(StarsBlocksHandle handle, size_t index, char** out_json) {
    clear_error_internal();
    
    if (!handle || !out_json) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    if (index >= handle->blocks.size()) {
        return make_result(STARS_ERROR_INVALID_HANDLE, "Index out of bounds");
    }
    
    try {
        auto it = handle->blocks.begin();
        std::advance(it, index);
        
        stars::serialization::archives::json::OutputArchive archive;
        stars::serialization::Serializer serializer(archive);
        serializer << STARS_SERIALIZER_DECORATE(*it);
        
        nlohmann::json result = archive.GetJSON();
        *out_json = duplicate_string(result.dump());
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_SERIALIZATION, std::string("Failed to serialize block: ") + e.what());
    }
}

void stars_blocks_destroy(StarsBlocksHandle handle) {
    delete handle;
}

/* ============================================================================
 * Prescheduler Implementation
 * ============================================================================ */

StarsResult stars_compute_possible_periods(
    StarsContextHandle ctx,
    StarsBlocksHandle blocks,
    StarsPossiblePeriodsHandle* out_handle
) {
    clear_error_internal();
    
    if (!ctx || !blocks || !out_handle) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    if (!ctx->instrument) {
        return make_result(STARS_ERROR_INVALID_HANDLE, "Context has no instrument configured");
    }
    
    try {
        auto periods = std::make_unique<StarsPossiblePeriods>();
        
        periods->periods = stars::scheduler::prescheduler::ComputePeriods(
            *ctx->instrument,
            blocks->blocks,
            ctx->executionPeriod
        );

        for (const auto& block : blocks->blocks) {
            periods->blockNamesById.emplace(block->GetId(), block->GetName());
        }
        
        *out_handle = periods.release();
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_PRESCHEDULER_FAILED, 
            std::string("Prescheduler failed: ") + e.what());
    }
}

StarsResult stars_possible_periods_to_json(StarsPossiblePeriodsHandle handle, char** out_json) {
    clear_error_internal();
    
    if (!handle || !out_json) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json result = nlohmann::json::array();
        
        for (const auto& [blockId, periods] : handle->periods) {
            nlohmann::json entry;
            entry["block_id"] = blockId;
            auto it = handle->blockNamesById.find(blockId);
            entry["block_name"] = (it == handle->blockNamesById.end()) ? blockId : it->second;
            
            nlohmann::json periodsArray = nlohmann::json::array();
            for (const auto& period : periods) {
                nlohmann::json p;
                p["begin"] = period.BeginTime().ToString();
                p["end"] = period.EndTime().ToString();
                periodsArray.push_back(p);
            }
            entry["periods"] = periodsArray;
            
            result.push_back(entry);
        }
        
        *out_json = duplicate_string(result.dump());
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_SERIALIZATION, 
            std::string("Failed to serialize periods: ") + e.what());
    }
}

void stars_possible_periods_destroy(StarsPossiblePeriodsHandle handle) {
    delete handle;
}

/* ============================================================================
 * Scheduling Algorithm Implementation
 * ============================================================================ */

StarsSchedulingParams stars_scheduling_params_default(void) {
    StarsSchedulingParams params;
    params.algorithm = STARS_SCHEDULER_ACCUMULATIVE;
    params.max_iterations = 0;
    params.time_limit_seconds = 0.0;
    params.seed = -1;
    return params;
}

StarsResult stars_run_scheduler(
    StarsContextHandle ctx,
    StarsBlocksHandle blocks,
    StarsPossiblePeriodsHandle possible_periods,
    StarsSchedulingParams params,
    StarsScheduleHandle* out_handle
) {
    clear_error_internal();
    
    if (!ctx || !blocks || !out_handle) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    if (!ctx->instrument) {
        return make_result(STARS_ERROR_INVALID_HANDLE, "Context has no instrument configured");
    }
    
    try {
        // Create appropriate algorithm
        std::unique_ptr<stars::scheduler::algorithms::SchedulingAlgorithm> algorithm;
        
        switch (params.algorithm) {
            case STARS_SCHEDULER_ACCUMULATIVE:
                algorithm = std::make_unique<
                    stars::scheduler::algorithms::accumulative::AccumulativeSchedulingAlgorithm>(
                    stars::scheduler::algorithms::accumulative::AccumulativeSchedulingAlgorithm::Configuration(
                        params.max_iterations ? params.max_iterations : 50,
                        /*reattempt=*/false,
                        /*range=*/1,
                        stars::scheduler::algorithms::figure_of_merit::TaskPriority,
                        params.seed >= 0 ? std::optional<int>{params.seed} : std::nullopt));
                break;
            case STARS_SCHEDULER_HYBRID_ACCUMULATIVE:
                algorithm = std::make_unique<
                    stars::scheduler::algorithms::hybrid_accumulative::HybridAccumulativeSchedulingAlgorithm>(
                    stars::scheduler::algorithms::hybrid_accumulative::HybridAccumulativeSchedulingAlgorithm::
                        Configuration(
                            std::max(1u, std::thread::hardware_concurrency()),
                            params.max_iterations ? params.max_iterations : 50,
                            /*reattempt=*/false,
                            /*range=*/1,
                            stars::scheduler::algorithms::figure_of_merit::TaskPriority,
                            /*seeds=*/{}));
                break;
            default:
                return make_result(STARS_ERROR_SCHEDULING_FAILED, "Unknown algorithm type");
        }
        
        // Compute possible periods if not provided
        stars::constraints::PossiblePeriodsMap periodsMap;
        if (possible_periods) {
            periodsMap = possible_periods->periods;
        } else {
            periodsMap = stars::scheduler::prescheduler::ComputePeriods(
                *ctx->instrument,
                blocks->blocks,
                ctx->executionPeriod
            );
        }
        
        // Run scheduling
        auto [schedule, unscheduled, failInfo] = algorithm->Schedule(
            blocks->blocks,
            ctx->instrument,
            ctx->executionPeriod
        );
        
        // Build result
        auto result = std::make_unique<StarsSchedule>();
        result->schedule = schedule;
        result->blocks = blocks->blocks;
        result->unscheduled = unscheduled;
        result->instrument = ctx->instrument;
        result->fitness = schedule ? schedule->ComputeFitness() : 0.0;
        
        *out_handle = result.release();
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_SCHEDULING_FAILED, 
            std::string("Scheduling failed: ") + e.what());
    }
}

StarsResult stars_schedule_to_json(StarsScheduleHandle handle, char** out_json) {
    clear_error_internal();
    
    if (!handle || !out_json) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json result;
        
        if (handle->schedule) {
            nlohmann::json units = nlohmann::json::array();
            
            for (const auto& unit : handle->schedule->GetUnits()) {
                nlohmann::json u;
                u["task_id"] = unit->GetTask()->GetId();
                u["task_name"] = unit->GetTask()->GetName();
                u["begin"] = unit->GetPeriod().BeginTime().ToString();
                u["end"] = unit->GetPeriod().EndTime().ToString();
                units.push_back(u);
            }
            
            result["units"] = units;
            result["fitness"] = handle->fitness;
            result["scheduled_count"] = handle->schedule->size();
        } else {
            result["units"] = nlohmann::json::array();
            result["fitness"] = 0.0;
            result["scheduled_count"] = 0;
        }
        
        // Add unscheduled blocks
        nlohmann::json unscheduled = nlohmann::json::array();
        for (const auto& block : handle->unscheduled) {
            nlohmann::json b;
            b["id"] = block->GetId();
            b["name"] = block->GetName();
            unscheduled.push_back(b);
        }
        result["unscheduled"] = unscheduled;
        result["unscheduled_count"] = handle->unscheduled.size();
        
        *out_json = duplicate_string(result.dump());
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_SERIALIZATION, 
            std::string("Failed to serialize schedule: ") + e.what());
    }
}

StarsResult stars_schedule_get_stats(StarsScheduleHandle handle, char** out_json) {
    clear_error_internal();
    
    if (!handle || !out_json) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    try {
        nlohmann::json stats;
        
        size_t scheduledCount = handle->schedule ? handle->schedule->size() : 0;
        size_t unscheduledCount = handle->unscheduled.size();
        size_t totalCount = scheduledCount + unscheduledCount;
        
        stats["scheduled_count"] = scheduledCount;
        stats["unscheduled_count"] = unscheduledCount;
        stats["total_blocks"] = totalCount;
        stats["scheduling_rate"] = totalCount > 0 
            ? static_cast<double>(scheduledCount) / totalCount 
            : 0.0;
        stats["fitness"] = handle->fitness;
        
        *out_json = duplicate_string(stats.dump());
        return make_ok();
        
    } catch (const std::exception& e) {
        return make_result(STARS_ERROR_SERIALIZATION, 
            std::string("Failed to get stats: ") + e.what());
    }
}

void stars_schedule_destroy(StarsScheduleHandle handle) {
    delete handle;
}

/* ============================================================================
 * Full Pipeline Implementation
 * ============================================================================ */

StarsResult stars_run_full_pipeline(
    const char* input_json,
    StarsSchedulingParams params,
    char** out_result_json
) {
    clear_error_internal();
    
    if (!input_json || !out_result_json) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    StarsContextHandle ctx = nullptr;
    StarsBlocksHandle blocks = nullptr;
    StarsScheduleHandle schedule = nullptr;
    
    StarsResult result;
    
    // Create context
    result = stars_context_create(input_json, &ctx);
    if (result.code != STARS_OK) {
        return result;
    }
    
    // Load blocks
    result = stars_blocks_load_json(input_json, &blocks);
    if (result.code != STARS_OK) {
        stars_context_destroy(ctx);
        return result;
    }
    
    // Run scheduler
    result = stars_run_scheduler(ctx, blocks, nullptr, params, &schedule);
    if (result.code != STARS_OK) {
        stars_context_destroy(ctx);
        stars_blocks_destroy(blocks);
        return result;
    }
    
    // Export result
    result = stars_schedule_to_json(schedule, out_result_json);
    
    // Cleanup
    stars_schedule_destroy(schedule);
    stars_blocks_destroy(blocks);
    stars_context_destroy(ctx);
    
    return result;
}

StarsResult stars_run_pipeline_from_file(
    const char* input_file_path,
    StarsSchedulingParams params,
    char** out_result_json
) {
    clear_error_internal();
    
    if (!input_file_path || !out_result_json) {
        return make_result(STARS_ERROR_NULL_POINTER, "Null pointer argument");
    }
    
    StarsContextHandle ctx = nullptr;
    StarsBlocksHandle blocks = nullptr;
    StarsScheduleHandle schedule = nullptr;
    
    StarsResult result;
    
    // Create context from file
    result = stars_context_create_from_file(input_file_path, &ctx);
    if (result.code != STARS_OK) {
        return result;
    }
    
    // Load blocks from file
    result = stars_blocks_load_file(input_file_path, &blocks);
    if (result.code != STARS_OK) {
        stars_context_destroy(ctx);
        return result;
    }
    
    // Run scheduler
    result = stars_run_scheduler(ctx, blocks, nullptr, params, &schedule);
    if (result.code != STARS_OK) {
        stars_context_destroy(ctx);
        stars_blocks_destroy(blocks);
        return result;
    }
    
    // Export result
    result = stars_schedule_to_json(schedule, out_result_json);
    
    // Cleanup
    stars_schedule_destroy(schedule);
    stars_blocks_destroy(blocks);
    stars_context_destroy(ctx);
    
    return result;
}

/* ============================================================================
 * Version Info Implementation
 * ============================================================================ */

const char* stars_ffi_version(void) {
    return "0.1.0";
}

const char* stars_core_version(void) {
    // This would ideally come from STARS Core's version info
    return "1.0.0";
}

} // extern "C"
