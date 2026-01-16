"""Task builder component for creating observation scheduling blocks.

This module provides UI controls for building observation tasks (scheduling blocks)
that follow the STARS Core schema. Tasks are the primary unit of scheduling.

Sequences of tasks can optionally group related tasks together (future feature).
"""

from __future__ import annotations

import logging
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any

import streamlit as st

logger = logging.getLogger(__name__)

# Session state keys
KEY_TASKS = "creative_tasks"
KEY_SEQUENCES = "creative_sequences"  # Future: optional grouping
KEY_SELECTED_TASK = "creative_selected_task"
KEY_EDITING_TASK = "creative_editing_task"


@dataclass
class ObservationTask:
    """
    Observation task (Scheduling Block) following STARS Core schema.
    
    This represents a single observation block that can be scheduled.
    In STARS terminology, this is a SchedulingBlock of type Task.
    """
    id: str
    name: str
    priority: float = 5.0
    duration_hours: float = 1.0
    min_observation_hours: float = 0.5
    ra_deg: float = 0.0
    dec_deg: float = 0.0
    min_altitude: float = 20.0
    max_altitude: float = 85.0
    min_azimuth: float = 0.0
    max_azimuth: float = 360.0
    # Optional sequence grouping (for future use)
    sequence_id: str | None = None
    
    def to_backend_dict(self) -> dict[str, Any]:
        """Convert to backend-compatible block dictionary."""
        # Convert duration to seconds
        duration_sec = self.duration_hours * 3600
        min_obs_sec = self.min_observation_hours * 3600
        
        return {
            "id": hash(self.id) % (10**10),  # Numeric ID
            "original_block_id": self.id,
            "name": self.name,
            "priority": self.priority,
            "target_ra": self.ra_deg,
            "target_dec": self.dec_deg,
            "constraints": {
                "min_alt": self.min_altitude,
                "max_alt": self.max_altitude,
                "min_az": self.min_azimuth,
                "max_az": self.max_azimuth,
                "fixed_time": None,
            },
            "min_observation": min_obs_sec,
            "requested_duration": duration_sec,
            "visibility_periods": [],  # Backend will compute
            "scheduled_period": None,
        }
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to STARS Core compatible dictionary (legacy format)."""
        return {
            "stars::scheduling_blocks::ObservationTask": {
                "name": self.name,
                "priority": self.priority,
                "duration": {
                    "days": 0,
                    "hours": int(self.duration_hours),
                    "minutes": int((self.duration_hours % 1) * 60),
                    "seconds": 0,
                },
                "minObservationTime": {
                    "days": 0,
                    "hours": int(self.min_observation_hours),
                    "minutes": int((self.min_observation_hours % 1) * 60),
                    "seconds": 0,
                },
                "targetCoordinates": {
                    "ra": self.ra_deg,
                    "dec": self.dec_deg,
                },
                "constraint": {
                    "constraints::AirmassAltitude": {
                        "range": {
                            "first": self.min_altitude,
                            "second": self.max_altitude,
                        }
                    }
                },
            }
        }


@dataclass 
class Sequence:
    """
    Sequence of observation tasks (future feature).
    
    Sequences group related tasks that should be scheduled together
    or in a specific order. This is a STARS Scheduling Block of type Sequence.
    
    Status: Placeholder for future implementation.
    """
    id: str
    name: str
    description: str = ""
    task_ids: list[str] = field(default_factory=list)
    order_matters: bool = False  # If True, tasks must be in order
    created_at: datetime = field(default_factory=datetime.now)


# ============================================================================
# Session State Management - Tasks (Primary)
# ============================================================================

def initialize_tasks_state() -> None:
    """Initialize tasks session state."""
    if KEY_TASKS not in st.session_state:
        st.session_state[KEY_TASKS] = []
    if KEY_SEQUENCES not in st.session_state:
        st.session_state[KEY_SEQUENCES] = []
    if KEY_SELECTED_TASK not in st.session_state:
        st.session_state[KEY_SELECTED_TASK] = None
    if KEY_EDITING_TASK not in st.session_state:
        st.session_state[KEY_EDITING_TASK] = None


def get_tasks() -> list[ObservationTask]:
    """Get all tasks."""
    initialize_tasks_state()
    return st.session_state[KEY_TASKS]


def add_task(task: ObservationTask) -> None:
    """Add a new task."""
    initialize_tasks_state()
    st.session_state[KEY_TASKS].append(task)


def remove_task(task_id: str) -> None:
    """Remove a task by ID."""
    initialize_tasks_state()
    st.session_state[KEY_TASKS] = [
        t for t in st.session_state[KEY_TASKS] if t.id != task_id
    ]
    if st.session_state[KEY_SELECTED_TASK] == task_id:
        st.session_state[KEY_SELECTED_TASK] = None


def get_task_by_id(task_id: str) -> ObservationTask | None:
    """Get a task by ID."""
    for task in get_tasks():
        if task.id == task_id:
            return task
    return None


def update_task(task_id: str, updated_task: ObservationTask) -> None:
    """Update an existing task."""
    initialize_tasks_state()
    tasks = st.session_state[KEY_TASKS]
    for i, task in enumerate(tasks):
        if task.id == task_id:
            tasks[i] = updated_task
            break


def get_selected_task() -> ObservationTask | None:
    """Get currently selected task."""
    initialize_tasks_state()
    selected_id = st.session_state[KEY_SELECTED_TASK]
    if selected_id:
        return get_task_by_id(selected_id)
    return None


def set_selected_task(task_id: str | None) -> None:
    """Set selected task by ID."""
    st.session_state[KEY_SELECTED_TASK] = task_id


def clear_all_tasks() -> None:
    """Clear all tasks."""
    st.session_state[KEY_TASKS] = []
    st.session_state[KEY_SELECTED_TASK] = None


# ============================================================================
# Session State Management - Sequences (Future/Optional)
# ============================================================================

def get_sequences() -> list[Sequence]:
    """Get all sequences (future feature)."""
    initialize_tasks_state()
    return st.session_state[KEY_SEQUENCES]


def add_sequence(sequence: Sequence) -> None:
    """Add a new sequence (future feature)."""
    initialize_tasks_state()
    st.session_state[KEY_SEQUENCES].append(sequence)


def remove_sequence(sequence_id: str) -> None:
    """Remove a sequence by ID (future feature)."""
    initialize_tasks_state()
    st.session_state[KEY_SEQUENCES] = [
        s for s in st.session_state[KEY_SEQUENCES] if s.id != sequence_id
    ]


# ============================================================================
# UI Rendering
# ============================================================================

def render_task_builder() -> None:
    """
    Render the task builder panel.
    
    Allows users to create observation tasks (scheduling blocks) directly.
    Sequences are shown as a future feature placeholder.
    """
    initialize_tasks_state()
    
    st.markdown("### 📝 Task Builder")
    
    # Task management tabs
    tab_tasks, tab_add_task, tab_sequences = st.tabs([
        "📋 Tasks", "➕ New Task", "🔗 Sequences (Coming Soon)"
    ])
    
    with tab_tasks:
        _render_tasks_list()
    
    with tab_add_task:
        _render_add_task_form()
    
    with tab_sequences:
        _render_sequences_placeholder()


def _render_tasks_list() -> None:
    """Render list of existing tasks."""
    tasks = get_tasks()
    
    if not tasks:
        st.info("No tasks yet. Create one in the 'New Task' tab!")
        return
    
    # Summary
    total_hours = sum(t.duration_hours for t in tasks)
    st.markdown(f"**{len(tasks)} tasks** • **{total_hours:.1f} hours** total")
    st.markdown("---")
    
    for task in tasks:
        with st.expander(f"🎯 {task.name} (P:{task.priority:.1f})", expanded=False):
            col1, col2 = st.columns(2)
            
            with col1:
                st.markdown(f"**ID:** `{task.id[:8]}...`")
                st.markdown(f"**Priority:** {task.priority:.1f}")
                st.markdown(f"**Duration:** {task.duration_hours:.1f}h")
            
            with col2:
                st.markdown(f"**RA:** {task.ra_deg:.2f}°")
                st.markdown(f"**Dec:** {task.dec_deg:.2f}°")
                st.markdown(f"**Alt:** {task.min_altitude:.0f}° - {task.max_altitude:.0f}°")
            
            # Task actions
            action_col1, action_col2 = st.columns(2)
            with action_col1:
                if st.button("✏️ Edit", key=f"edit_{task.id}", width="stretch"):
                    st.session_state[KEY_EDITING_TASK] = task.id
                    st.rerun()
            with action_col2:
                if st.button("🗑️ Delete", key=f"del_{task.id}", width="stretch"):
                    remove_task(task.id)
                    st.rerun()
    
    # Bulk actions
    st.markdown("---")
    if st.button("🗑️ Clear All Tasks", type="secondary"):
        clear_all_tasks()
        st.rerun()


def _render_add_task_form() -> None:
    """Render form for adding a new task."""
    editing_task_id = st.session_state.get(KEY_EDITING_TASK)
    editing_task = get_task_by_id(editing_task_id) if editing_task_id else None
    
    if editing_task:
        st.markdown("#### ✏️ Edit Task")
        if st.button("← Cancel Edit"):
            st.session_state[KEY_EDITING_TASK] = None
            st.rerun()
    else:
        st.markdown("#### Create a new observation task:")
    
    with st.form("add_task_form"):
        # Basic info
        task_name = st.text_input(
            "Task Name",
            value=editing_task.name if editing_task else "",
            placeholder="e.g., M51 Observation",
            key="task_name",
        )
        
        col1, col2 = st.columns(2)
        with col1:
            priority = st.slider(
                "Priority",
                min_value=0.0,
                max_value=10.0,
                value=editing_task.priority if editing_task else 5.0,
                step=0.1,
                key="task_priority",
                help="Higher priority = scheduled first",
            )
        with col2:
            duration = st.number_input(
                "Duration (hours)",
                min_value=0.1,
                max_value=12.0,
                value=editing_task.duration_hours if editing_task else 1.0,
                step=0.5,
                key="task_duration",
            )
        
        # Coordinates
        st.markdown("**Target Coordinates**")
        coord_col1, coord_col2 = st.columns(2)
        with coord_col1:
            ra_deg = st.number_input(
                "RA (degrees)",
                min_value=0.0,
                max_value=360.0,
                value=editing_task.ra_deg if editing_task else 0.0,
                step=0.1,
                key="task_ra",
                help="Right Ascension in degrees (0-360)",
            )
        with coord_col2:
            dec_deg = st.number_input(
                "Dec (degrees)",
                min_value=-90.0,
                max_value=90.0,
                value=editing_task.dec_deg if editing_task else 0.0,
                step=0.1,
                key="task_dec",
                help="Declination in degrees (-90 to +90)",
            )
        
        # Constraints
        with st.expander("⚙️ Constraints (optional)", expanded=False):
            min_obs = st.number_input(
                "Minimum Observation Time (hours)",
                min_value=0.1,
                max_value=12.0,
                value=editing_task.min_observation_hours if editing_task else 0.5,
                step=0.1,
                key="task_min_obs",
            )
            
            alt_col1, alt_col2 = st.columns(2)
            with alt_col1:
                min_alt = st.number_input(
                    "Min Altitude (°)",
                    min_value=0.0,
                    max_value=90.0,
                    value=editing_task.min_altitude if editing_task else 20.0,
                    key="task_min_alt",
                )
            with alt_col2:
                max_alt = st.number_input(
                    "Max Altitude (°)",
                    min_value=0.0,
                    max_value=90.0,
                    value=editing_task.max_altitude if editing_task else 85.0,
                    key="task_max_alt",
                )
            
            az_col1, az_col2 = st.columns(2)
            with az_col1:
                min_az = st.number_input(
                    "Min Azimuth (°)",
                    min_value=0.0,
                    max_value=360.0,
                    value=editing_task.min_azimuth if editing_task else 0.0,
                    key="task_min_az",
                )
            with az_col2:
                max_az = st.number_input(
                    "Max Azimuth (°)",
                    min_value=0.0,
                    max_value=360.0,
                    value=editing_task.max_azimuth if editing_task else 360.0,
                    key="task_max_az",
                )
        
        button_label = "Update Task" if editing_task else "Add Task"
        submitted = st.form_submit_button(button_label, type="primary")
        
        if submitted:
            if not task_name:
                st.error("Please provide a task name")
            else:
                new_task = ObservationTask(
                    id=editing_task.id if editing_task else str(uuid.uuid4()),
                    name=task_name,
                    priority=priority,
                    duration_hours=duration,
                    min_observation_hours=min_obs,
                    ra_deg=ra_deg,
                    dec_deg=dec_deg,
                    min_altitude=min_alt,
                    max_altitude=max_alt,
                    min_azimuth=min_az,
                    max_azimuth=max_az,
                )
                
                if editing_task:
                    update_task(editing_task.id, new_task)
                    st.session_state[KEY_EDITING_TASK] = None
                    st.success(f"Updated task: {task_name}")
                else:
                    add_task(new_task)
                    st.success(f"Added task: {task_name}")
                st.rerun()


def _render_sequences_placeholder() -> None:
    """Render placeholder for sequences feature (coming soon)."""
    st.markdown(
        """
        <div style="
            border: 2px dashed #666;
            border-radius: 10px;
            padding: 40px 20px;
            text-align: center;
            color: #888;
            margin: 20px 0;
        ">
            <h4>🔗 Sequences - Coming Soon</h4>
            <p>
                Sequences allow you to group related tasks that should be 
                scheduled together or in a specific order.
            </p>
            <p style="font-size: 0.9em;">
                In STARS terminology, this is a <b>Scheduling Block of type Sequence</b>.
            </p>
        </div>
        """,
        unsafe_allow_html=True,
    )
    
    st.info(
        "💡 **Tip:** For now, create individual tasks. "
        "Sequence grouping will be available in a future update."
    )


# ============================================================================
# Export Functions
# ============================================================================

def export_tasks_to_schedule_json() -> dict[str, Any]:
    """
    Export all tasks to backend-compatible schedule JSON.
    
    The format matches the expected schema:
    - name: schedule name
    - checksum: optional checksum
    - dark_periods: list of {start, stop} MJD periods
    - blocks: list of scheduling blocks
    
    Returns:
        Dictionary in schedule.json format.
    """
    import hashlib
    
    tasks = get_tasks()
    all_blocks = []
    
    for task in tasks:
        block = task.to_backend_dict()
        all_blocks.append(block)
    
    # Create schedule structure matching backend expectations
    schedule_data = {
        "name": "creative_schedule",
        "checksum": hashlib.sha256(
            str(all_blocks).encode()
        ).hexdigest(),
        "dark_periods": [],  # Backend will compute based on period
        "blocks": all_blocks,
    }
    
    return schedule_data

