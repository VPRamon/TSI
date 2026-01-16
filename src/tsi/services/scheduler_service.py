"""Scheduler service for running scheduling simulations.

This module provides the interface between the creative workspace
and the STARS Core scheduling backend.
"""

from __future__ import annotations

import json
import logging
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Callable

logger = logging.getLogger(__name__)

# MJD conversion constants
UNIX_EPOCH_MJD = 40587.0
SECONDS_PER_DAY = 86400.0


@dataclass
class SchedulingResult:
    """Result of a scheduling simulation."""
    success: bool
    schedule_id: int | None = None
    scheduled_count: int = 0
    unscheduled_count: int = 0
    error: str | None = None
    execution_time_seconds: float = 0.0


def datetime_to_mjd(dt: datetime) -> float:
    """Convert datetime to Modified Julian Date."""
    unix_timestamp = dt.timestamp()
    return (unix_timestamp / SECONDS_PER_DAY) + UNIX_EPOCH_MJD


def date_to_mjd(d: Any) -> float:
    """Convert date to MJD (at midnight UTC)."""
    dt = datetime.combine(d, datetime.min.time(), tzinfo=timezone.utc)
    return datetime_to_mjd(dt)


def run_scheduling_simulation(
    config: Any,
    log_callback: Callable[[str], None] | None = None,
) -> SchedulingResult:
    """
    Run a scheduling simulation with the given configuration.
    
    Args:
        config: SchedulerConfig with algorithm, period, and location settings.
        log_callback: Optional callback for streaming log messages.
    
    Returns:
        SchedulingResult with success status and schedule details.
    """
    from tsi.components.creative.task_builder import (
        export_tasks_to_schedule_json,
        get_tasks,
    )
    
    def log(msg: str) -> None:
        """Log a message via callback if provided."""
        if log_callback:
            log_callback(msg)
        logger.info(msg)
    
    start_time = time.time()
    
    try:
        # Get tasks
        tasks = get_tasks()
        if not tasks:
            return SchedulingResult(
                success=False,
                error="No tasks to schedule",
            )
        
        log(f"📋 Found {len(tasks)} tasks to schedule")
        
        # Export to schedule JSON format
        schedule_data = export_tasks_to_schedule_json()
        log(f"📦 Prepared schedule with {len(schedule_data['blocks'])} blocks")
        
        # Build schedule period
        start_mjd = date_to_mjd(config.start_date)
        end_mjd = date_to_mjd(config.end_date)
        
        log(f"📅 Schedule period: MJD {start_mjd:.2f} to {end_mjd:.2f}")
        
        # Build context for scheduler
        context = _build_scheduler_context(config, start_mjd, end_mjd)
        log(f"📍 Location: {config.location.name}")
        log(f"🔧 Algorithm: {config.algorithm.display_name}")
        
        # Try to use the Rust backend
        try:
            result = _run_with_backend(
                schedule_data=schedule_data,
                context=context,
                config=config,
                log_callback=log,
            )
        except ImportError:
            log("⚠️ Rust backend not available, using simulation mode")
            result = _run_simulation_mode(
                schedule_data=schedule_data,
                config=config,
                log_callback=log,
            )
        
        execution_time = time.time() - start_time
        result.execution_time_seconds = execution_time
        
        log(f"⏱️ Execution time: {execution_time:.2f}s")
        
        return result
    
    except Exception as e:
        logger.exception("Scheduling simulation failed")
        return SchedulingResult(
            success=False,
            error=str(e),
            execution_time_seconds=time.time() - start_time,
        )


def _build_scheduler_context(
    config: Any,
    start_mjd: float,
    end_mjd: float,
) -> dict[str, Any]:
    """Build the scheduler context dictionary."""
    return {
        "instrument": {
            "name": config.location.name,
            "location": {
                "name": config.location.name,
                "latitude": config.location.latitude,
                "longitude": config.location.longitude,
                "altitude": config.location.altitude_m,
            },
            "capabilities": {
                "min_elevation": config.location.min_elevation,
                "max_elevation": config.location.max_elevation,
            },
        },
        "executionPeriod": {
            "begin_mjd": start_mjd,
            "end_mjd": end_mjd,
        },
        "algorithm": {
            "type": config.algorithm.value,
            "max_iterations": config.max_iterations,
            "time_limit_seconds": config.time_limit_seconds,
            "seed": config.seed,
        },
    }


def _run_with_backend(
    schedule_data: dict[str, Any],
    context: dict[str, Any],
    config: Any,
    log_callback: Callable[[str], None],
) -> SchedulingResult:
    """
    Run scheduling using the Rust backend.
    
    This integrates with the STARS Core scheduling engine.
    """
    import tsi_rust as api
    from tsi.services import backend
    
    log_callback("🔗 Connecting to Rust backend...")
    
    # Convert to JSON strings for API
    schedule_json = json.dumps(schedule_data)
    
    log_callback("📤 Uploading schedule to backend...")
    
    # Create a unique name for this simulation
    schedule_name = f"creative_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
    
    # Upload the schedule
    # Note: The backend will process and store the schedule
    schedule_summary = backend.upload_schedule(
        schedule_name=schedule_name,
        schedule_json=schedule_json,
        visibility_json=None,  # Backend computes visibility
    )
    
    log_callback(f"✅ Schedule stored with ID: {schedule_summary.id}")
    
    # Get schedule statistics
    log_callback("📊 Retrieving schedule statistics...")
    
    try:
        # Fetch validation report to get counts
        validation = backend.get_validation_report(schedule_summary.ref)
        
        scheduled = getattr(validation, 'scheduled_count', 0)
        unscheduled = getattr(validation, 'unscheduled_count', 0)
        
        if hasattr(validation, 'summary'):
            summary = validation.summary
            scheduled = getattr(summary, 'scheduled_blocks', scheduled)
            unscheduled = getattr(summary, 'unscheduled_blocks', unscheduled)
    except Exception as e:
        log_callback(f"⚠️ Could not get detailed stats: {e}")
        scheduled = len(schedule_data['blocks'])
        unscheduled = 0
    
    return SchedulingResult(
        success=True,
        schedule_id=schedule_summary.id,
        scheduled_count=scheduled,
        unscheduled_count=unscheduled,
    )


def _run_simulation_mode(
    schedule_data: dict[str, Any],
    config: Any,
    log_callback: Callable[[str], None],
) -> SchedulingResult:
    """
    Run a simulated scheduling process when backend is not available.
    
    This provides a demonstration of the workflow without actual scheduling.
    """
    from tsi.services import backend
    
    blocks = schedule_data.get('blocks', [])
    total_blocks = len(blocks)
    
    log_callback("🔄 Running in simulation mode...")
    log_callback("")
    
    # Simulate processing each block
    for i, block in enumerate(blocks):
        # Extract task info - use original_block_id or generate name
        task_name = block.get('original_block_id', f'Task {i+1}')
        
        log_callback(f"  Processing: {task_name}")
        time.sleep(0.1)  # Small delay for visual effect
    
    log_callback("")
    log_callback("🔄 Computing visibility windows...")
    time.sleep(0.3)
    
    log_callback("🔄 Running optimization...")
    time.sleep(0.5)
    
    # Simulate some tasks being unschedulable
    import random
    scheduled = int(total_blocks * random.uniform(0.7, 0.95))
    unscheduled = total_blocks - scheduled
    
    # Actually upload to backend if available
    try:
        schedule_name = f"creative_sim_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        schedule_json = json.dumps(schedule_data)
        
        schedule_summary = backend.upload_schedule(
            schedule_name=schedule_name,
            schedule_json=schedule_json,
            visibility_json=None,
        )
        
        log_callback(f"📤 Stored simulation result with ID: {schedule_summary.id}")
        
        return SchedulingResult(
            success=True,
            schedule_id=schedule_summary.id,
            scheduled_count=scheduled,
            unscheduled_count=unscheduled,
        )
    except Exception as e:
        log_callback(f"⚠️ Backend storage unavailable: {e}")
        log_callback("📝 Results available in session only")
        
        return SchedulingResult(
            success=True,
            schedule_id=None,
            scheduled_count=scheduled,
            unscheduled_count=unscheduled,
        )
