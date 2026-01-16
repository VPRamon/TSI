"""Scheduler configuration component for creative workspace.

This module provides UI controls for configuring the scheduling algorithm,
parameters, location, and schedule period before running simulations.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from datetime import datetime, date, timedelta
from enum import Enum
from typing import Any

import streamlit as st

logger = logging.getLogger(__name__)

# Session state keys
KEY_SCHEDULER_CONFIG = "creative_scheduler_config"
KEY_SCHEDULER_RUNNING = "creative_scheduler_running"
KEY_SCHEDULER_LOGS = "creative_scheduler_logs"


class SchedulerAlgorithm(str, Enum):
    """Available scheduling algorithms."""
    ACCUMULATIVE = "accumulative"
    HYBRID_ACCUMULATIVE = "hybrid_accumulative"
    
    @property
    def display_name(self) -> str:
        """Human-readable name."""
        names = {
            "accumulative": "Accumulative",
            "hybrid_accumulative": "Hybrid Accumulative (Multi-threaded)",
        }
        return names.get(self.value, self.value)


@dataclass
class LocationConfig:
    """Observatory location configuration."""
    name: str = "Default Observatory"
    latitude: float = 28.7606  # GTC default
    longitude: float = -17.8810
    altitude_m: float = 2396.0
    min_elevation: float = 20.0
    max_elevation: float = 85.0


@dataclass
class SchedulerConfig:
    """Complete scheduler configuration."""
    # Algorithm settings
    algorithm: SchedulerAlgorithm = SchedulerAlgorithm.ACCUMULATIVE
    max_iterations: int = 0  # 0 = use default
    time_limit_seconds: float = 0.0  # 0 = no limit
    seed: int = -1  # -1 = random
    
    # Schedule period
    start_date: date = field(default_factory=lambda: date.today())
    end_date: date = field(default_factory=lambda: date.today() + timedelta(days=30))
    
    # Location
    location: LocationConfig = field(default_factory=LocationConfig)
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for API calls."""
        return {
            "algorithm": self.algorithm.value,
            "max_iterations": self.max_iterations,
            "time_limit_seconds": self.time_limit_seconds,
            "seed": self.seed,
            "schedule_period": {
                "start": self.start_date.isoformat(),
                "end": self.end_date.isoformat(),
            },
            "location": {
                "name": self.location.name,
                "latitude": self.location.latitude,
                "longitude": self.location.longitude,
                "altitude_m": self.location.altitude_m,
                "min_elevation": self.location.min_elevation,
                "max_elevation": self.location.max_elevation,
            },
        }


# Preset observatory locations
PRESET_LOCATIONS = {
    "Gran Telescopio Canarias (GTC)": LocationConfig(
        name="Roque de los Muchachos Observatory",
        latitude=28.7606,
        longitude=-17.8810,
        altitude_m=2396.0,
        min_elevation=20.0,
        max_elevation=85.0,
    ),
    "Very Large Telescope (VLT)": LocationConfig(
        name="Cerro Paranal Observatory",
        latitude=-24.6275,
        longitude=-70.4044,
        altitude_m=2635.0,
        min_elevation=20.0,
        max_elevation=85.0,
    ),
    "Keck Observatory": LocationConfig(
        name="Mauna Kea Observatory",
        latitude=19.8260,
        longitude=-155.4747,
        altitude_m=4145.0,
        min_elevation=15.0,
        max_elevation=88.0,
    ),
    "Custom": LocationConfig(
        name="Custom Location",
        latitude=0.0,
        longitude=0.0,
        altitude_m=0.0,
    ),
}


def initialize_scheduler_config() -> None:
    """Initialize scheduler configuration in session state."""
    if KEY_SCHEDULER_CONFIG not in st.session_state:
        st.session_state[KEY_SCHEDULER_CONFIG] = SchedulerConfig()
    if KEY_SCHEDULER_RUNNING not in st.session_state:
        st.session_state[KEY_SCHEDULER_RUNNING] = False
    if KEY_SCHEDULER_LOGS not in st.session_state:
        st.session_state[KEY_SCHEDULER_LOGS] = []


def get_scheduler_config() -> SchedulerConfig:
    """Get current scheduler configuration."""
    initialize_scheduler_config()
    return st.session_state[KEY_SCHEDULER_CONFIG]


def set_scheduler_config(config: SchedulerConfig) -> None:
    """Set scheduler configuration."""
    st.session_state[KEY_SCHEDULER_CONFIG] = config


def is_scheduler_running() -> bool:
    """Check if scheduler is currently running."""
    return st.session_state.get(KEY_SCHEDULER_RUNNING, False)


def set_scheduler_running(running: bool) -> None:
    """Set scheduler running state."""
    st.session_state[KEY_SCHEDULER_RUNNING] = running


def get_scheduler_logs() -> list[str]:
    """Get scheduler execution logs."""
    return st.session_state.get(KEY_SCHEDULER_LOGS, [])


def add_scheduler_log(message: str) -> None:
    """Add a log message."""
    if KEY_SCHEDULER_LOGS not in st.session_state:
        st.session_state[KEY_SCHEDULER_LOGS] = []
    st.session_state[KEY_SCHEDULER_LOGS].append(message)


def clear_scheduler_logs() -> None:
    """Clear scheduler logs."""
    st.session_state[KEY_SCHEDULER_LOGS] = []


def render_scheduler_config() -> None:
    """
    Render the scheduler configuration panel.
    
    Provides controls for algorithm selection, parameters,
    location, and schedule period configuration.
    """
    initialize_scheduler_config()
    config = get_scheduler_config()
    
    st.markdown("### ⚙️ Scheduler Configuration")
    
    # Configuration tabs
    tab_algo, tab_period, tab_location = st.tabs([
        "🔧 Algorithm", "📅 Period", "📍 Location"
    ])
    
    with tab_algo:
        _render_algorithm_config(config)
    
    with tab_period:
        _render_period_config(config)
    
    with tab_location:
        _render_location_config(config)
    
    # Update config
    set_scheduler_config(config)
    
    st.markdown("---")
    
    # Run button and logs
    _render_run_section()


def _render_algorithm_config(config: SchedulerConfig) -> None:
    """Render algorithm configuration controls."""
    st.markdown("**Algorithm Selection**")
    
    algo_options = [a for a in SchedulerAlgorithm]
    algo_names = [a.display_name for a in algo_options]
    
    current_idx = algo_options.index(config.algorithm)
    
    selected_name = st.selectbox(
        "Scheduling Algorithm",
        options=algo_names,
        index=current_idx,
        key="algo_select",
        help="Choose the scheduling algorithm to use",
    )
    
    config.algorithm = algo_options[algo_names.index(selected_name)]
    
    st.markdown("**Parameters**")
    
    col1, col2 = st.columns(2)
    
    with col1:
        config.max_iterations = st.number_input(
            "Max Iterations",
            min_value=0,
            max_value=100000,
            value=config.max_iterations,
            step=100,
            key="max_iter",
            help="Maximum optimization iterations (0 = default)",
        )
    
    with col2:
        config.time_limit_seconds = st.number_input(
            "Time Limit (seconds)",
            min_value=0.0,
            max_value=3600.0,
            value=config.time_limit_seconds,
            step=10.0,
            key="time_limit",
            help="Maximum time for scheduling (0 = no limit)",
        )
    
    config.seed = st.number_input(
        "Random Seed",
        min_value=-1,
        max_value=999999,
        value=config.seed,
        step=1,
        key="seed",
        help="Seed for reproducibility (-1 = random)",
    )


def _render_period_config(config: SchedulerConfig) -> None:
    """Render schedule period configuration."""
    st.markdown("**Observation Period**")
    
    col1, col2 = st.columns(2)
    
    with col1:
        start = st.date_input(
            "Start Date",
            value=config.start_date,
            key="start_date",
        )
        config.start_date = start
    
    with col2:
        end = st.date_input(
            "End Date",
            value=config.end_date,
            key="end_date",
        )
        config.end_date = end
    
    # Validate period
    if config.start_date >= config.end_date:
        st.error("End date must be after start date!")
    else:
        delta = config.end_date - config.start_date
        st.info(f"📅 Period: {delta.days} days")
    
    # Quick presets
    st.markdown("**Quick Presets**")
    preset_col1, preset_col2, preset_col3 = st.columns(3)
    
    with preset_col1:
        if st.button("1 Week", key="preset_1w", width="stretch"):
            config.start_date = date.today()
            config.end_date = date.today() + timedelta(days=7)
            st.rerun()
    
    with preset_col2:
        if st.button("1 Month", key="preset_1m", width="stretch"):
            config.start_date = date.today()
            config.end_date = date.today() + timedelta(days=30)
            st.rerun()
    
    with preset_col3:
        if st.button("1 Semester", key="preset_6m", width="stretch"):
            config.start_date = date.today()
            config.end_date = date.today() + timedelta(days=180)
            st.rerun()


def _render_location_config(config: SchedulerConfig) -> None:
    """Render location configuration."""
    st.markdown("**Observatory Location**")
    
    # Preset selector
    preset_names = list(PRESET_LOCATIONS.keys())
    
    # Find current preset if matching
    current_preset = "Custom"
    for name, preset in PRESET_LOCATIONS.items():
        if (abs(config.location.latitude - preset.latitude) < 0.01 and
            abs(config.location.longitude - preset.longitude) < 0.01):
            current_preset = name
            break
    
    selected_preset = st.selectbox(
        "Observatory Preset",
        options=preset_names,
        index=preset_names.index(current_preset),
        key="location_preset",
    )
    
    if selected_preset != "Custom":
        preset = PRESET_LOCATIONS[selected_preset]
        config.location = LocationConfig(
            name=preset.name,
            latitude=preset.latitude,
            longitude=preset.longitude,
            altitude_m=preset.altitude_m,
            min_elevation=preset.min_elevation,
            max_elevation=preset.max_elevation,
        )
        st.success(f"📍 {preset.name}")
    
    # Custom location fields
    with st.expander("🔧 Custom Location", expanded=(selected_preset == "Custom")):
        config.location.name = st.text_input(
            "Location Name",
            value=config.location.name,
            key="loc_name",
        )
        
        col1, col2 = st.columns(2)
        with col1:
            config.location.latitude = st.number_input(
                "Latitude (°)",
                min_value=-90.0,
                max_value=90.0,
                value=config.location.latitude,
                step=0.0001,
                format="%.4f",
                key="loc_lat",
            )
        with col2:
            config.location.longitude = st.number_input(
                "Longitude (°)",
                min_value=-180.0,
                max_value=180.0,
                value=config.location.longitude,
                step=0.0001,
                format="%.4f",
                key="loc_lon",
            )
        
        config.location.altitude_m = st.number_input(
            "Altitude (m)",
            min_value=0.0,
            max_value=10000.0,
            value=config.location.altitude_m,
            step=1.0,
            key="loc_alt",
        )
        
        el_col1, el_col2 = st.columns(2)
        with el_col1:
            config.location.min_elevation = st.number_input(
                "Min Elevation (°)",
                min_value=0.0,
                max_value=90.0,
                value=config.location.min_elevation,
                step=1.0,
                key="loc_min_el",
            )
        with el_col2:
            config.location.max_elevation = st.number_input(
                "Max Elevation (°)",
                min_value=0.0,
                max_value=90.0,
                value=config.location.max_elevation,
                step=1.0,
                key="loc_max_el",
            )


def _render_run_section() -> None:
    """Render the run scheduler section with logs."""
    from tsi.components.creative.proposal_builder import get_proposals, get_all_tasks
    
    proposals = get_proposals()
    tasks = get_all_tasks()
    
    # Validation
    can_run = True
    if not proposals:
        st.warning("⚠️ Create at least one proposal to run the scheduler")
        can_run = False
    elif not tasks:
        st.warning("⚠️ Add at least one task to run the scheduler")
        can_run = False
    
    config = get_scheduler_config()
    if config.start_date >= config.end_date:
        st.error("⚠️ Invalid schedule period")
        can_run = False
    
    # Run button
    col1, col2 = st.columns([2, 1])
    
    with col1:
        run_clicked = st.button(
            "🚀 Run Scheduler",
            type="primary",
            disabled=not can_run or is_scheduler_running(),
            width="stretch",
            key="run_scheduler_btn",
        )
    
    with col2:
        if st.button(
            "🗑️ Clear Logs",
            disabled=is_scheduler_running(),
            width="stretch",
            key="clear_logs_btn",
        ):
            clear_scheduler_logs()
            st.rerun()
    
    # Log display
    logs = get_scheduler_logs()
    if logs:
        st.markdown("**Execution Logs**")
        log_container = st.container(height=200)
        with log_container:
            for log in logs:
                st.text(log)
    
    # Handle run click
    if run_clicked:
        _run_scheduler()


def _run_scheduler() -> None:
    """Execute the scheduling simulation."""
    from tsi.services.scheduler_service import run_scheduling_simulation
    
    set_scheduler_running(True)
    clear_scheduler_logs()
    
    try:
        config = get_scheduler_config()
        
        add_scheduler_log("=" * 50)
        add_scheduler_log("🚀 Starting scheduling simulation...")
        add_scheduler_log(f"Algorithm: {config.algorithm.display_name}")
        add_scheduler_log(f"Period: {config.start_date} to {config.end_date}")
        add_scheduler_log(f"Location: {config.location.name}")
        add_scheduler_log("=" * 50)
        
        # Run the simulation
        result = run_scheduling_simulation(
            config=config,
            log_callback=add_scheduler_log,
        )
        
        if result.success:
            add_scheduler_log("")
            add_scheduler_log("✅ Scheduling completed successfully!")
            add_scheduler_log(f"Schedule ID: {result.schedule_id}")
            add_scheduler_log(f"Scheduled: {result.scheduled_count} tasks")
            add_scheduler_log(f"Unscheduled: {result.unscheduled_count} tasks")
            
            # Store result for navigation
            st.session_state["creative_result_schedule_id"] = result.schedule_id
            st.session_state["creative_result_ready"] = True
        else:
            add_scheduler_log("")
            add_scheduler_log(f"❌ Scheduling failed: {result.error}")
    
    except Exception as e:
        add_scheduler_log("")
        add_scheduler_log(f"❌ Error: {str(e)}")
        logger.exception("Scheduler execution failed")
    
    finally:
        set_scheduler_running(False)
        st.rerun()
