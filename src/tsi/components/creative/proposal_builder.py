"""Proposal builder component for creating observation tasks.

This module provides UI controls for building observation proposals
with tasks that follow the STARS Core scheduling block schema.
"""

from __future__ import annotations

import logging
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Any

import streamlit as st

logger = logging.getLogger(__name__)

# Session state keys
KEY_PROPOSALS = "creative_proposals"
KEY_SELECTED_PROPOSAL = "creative_selected_proposal"
KEY_EDITING_TASK = "creative_editing_task"


@dataclass
class ObservationTask:
    """
    Observation task following STARS Core schema.
    
    This represents a single observation block that can be scheduled.
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
    
    def to_backend_dict(self) -> dict[str, Any]:
        """Convert to backend-compatible block dictionary."""
        # Convert duration to seconds
        duration_sec = self.duration_hours * 3600
        min_obs_sec = self.min_observation_hours * 3600
        
        return {
            "id": hash(self.id) % (10**10),  # Numeric ID
            "original_block_id": self.id,
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
class Proposal:
    """
    A proposal containing observation tasks.
    
    Proposals group related observations together.
    """
    id: str
    name: str
    description: str = ""
    tasks: list[ObservationTask] = field(default_factory=list)
    created_at: datetime = field(default_factory=datetime.now)
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to schedule JSON format."""
        return {
            "proposal_id": self.id,
            "proposal_name": self.name,
            "description": self.description,
            "schedulingBlocks": [task.to_dict() for task in self.tasks],
        }


# TODO: Add support for sequences (intermediate nodes in tree structure)
# Sequences would group tasks that must be executed together or in order.
# @dataclass
# class Sequence:
#     """Sequence of observation tasks that must be executed together."""
#     id: str
#     name: str
#     tasks: list[ObservationTask]
#     order_matters: bool = False  # If True, tasks must be in order


def initialize_proposals_state() -> None:
    """Initialize proposals session state."""
    if KEY_PROPOSALS not in st.session_state:
        st.session_state[KEY_PROPOSALS] = []
    if KEY_SELECTED_PROPOSAL not in st.session_state:
        st.session_state[KEY_SELECTED_PROPOSAL] = None
    if KEY_EDITING_TASK not in st.session_state:
        st.session_state[KEY_EDITING_TASK] = None


def get_proposals() -> list[Proposal]:
    """Get current proposals."""
    initialize_proposals_state()
    return st.session_state[KEY_PROPOSALS]


def add_proposal(proposal: Proposal) -> None:
    """Add a new proposal."""
    initialize_proposals_state()
    st.session_state[KEY_PROPOSALS].append(proposal)


def remove_proposal(proposal_id: str) -> None:
    """Remove a proposal by ID."""
    initialize_proposals_state()
    st.session_state[KEY_PROPOSALS] = [
        p for p in st.session_state[KEY_PROPOSALS] if p.id != proposal_id
    ]
    if st.session_state[KEY_SELECTED_PROPOSAL] == proposal_id:
        st.session_state[KEY_SELECTED_PROPOSAL] = None


def get_selected_proposal() -> Proposal | None:
    """Get currently selected proposal."""
    initialize_proposals_state()
    selected_id = st.session_state[KEY_SELECTED_PROPOSAL]
    if selected_id:
        for p in st.session_state[KEY_PROPOSALS]:
            if p.id == selected_id:
                return p
    return None


def set_selected_proposal(proposal_id: str | None) -> None:
    """Set selected proposal by ID."""
    st.session_state[KEY_SELECTED_PROPOSAL] = proposal_id


def add_task_to_proposal(proposal_id: str, task: ObservationTask) -> None:
    """Add a task to a proposal."""
    for proposal in get_proposals():
        if proposal.id == proposal_id:
            proposal.tasks.append(task)
            break


def remove_task_from_proposal(proposal_id: str, task_id: str) -> None:
    """Remove a task from a proposal."""
    for proposal in get_proposals():
        if proposal.id == proposal_id:
            proposal.tasks = [t for t in proposal.tasks if t.id != task_id]
            break


def get_all_tasks() -> list[ObservationTask]:
    """Get all tasks from all proposals."""
    all_tasks = []
    for proposal in get_proposals():
        all_tasks.extend(proposal.tasks)
    return all_tasks


def render_proposal_builder() -> None:
    """
    Render the proposal builder panel.
    
    Allows users to create proposals and add observation tasks.
    """
    initialize_proposals_state()
    
    st.markdown("### 📝 Proposal Builder")
    
    # Proposal management tabs
    tab_proposals, tab_add_proposal, tab_add_task = st.tabs([
        "📋 Proposals", "➕ New Proposal", "🎯 Add Task"
    ])
    
    with tab_proposals:
        _render_proposals_list()
    
    with tab_add_proposal:
        _render_new_proposal_form()
    
    with tab_add_task:
        _render_add_task_form()


def _render_proposals_list() -> None:
    """Render list of existing proposals."""
    proposals = get_proposals()
    
    if not proposals:
        st.info("No proposals yet. Create one in the 'New Proposal' tab!")
        return
    
    for proposal in proposals:
        with st.expander(f"📁 {proposal.name} ({len(proposal.tasks)} tasks)", expanded=False):
            st.markdown(f"**ID:** `{proposal.id[:8]}...`")
            if proposal.description:
                st.markdown(f"**Description:** {proposal.description}")
            
            # Task list
            if proposal.tasks:
                st.markdown("**Tasks:**")
                for task in proposal.tasks:
                    cols = st.columns([3, 1, 1, 1])
                    with cols[0]:
                        st.markdown(f"🎯 {task.name}")
                    with cols[1]:
                        st.markdown(f"P: {task.priority:.1f}")
                    with cols[2]:
                        st.markdown(f"{task.duration_hours:.1f}h")
                    with cols[3]:
                        if st.button("🗑️", key=f"del_task_{task.id}", help="Delete task"):
                            remove_task_from_proposal(proposal.id, task.id)
                            st.rerun()
            else:
                st.caption("No tasks in this proposal")
            
            # Proposal actions
            col1, col2 = st.columns(2)
            with col1:
                if st.button("✏️ Select", key=f"select_{proposal.id}", width="stretch"):
                    set_selected_proposal(proposal.id)
                    st.rerun()
            with col2:
                if st.button("🗑️ Delete", key=f"del_{proposal.id}", width="stretch"):
                    remove_proposal(proposal.id)
                    st.rerun()


def _render_new_proposal_form() -> None:
    """Render form for creating a new proposal."""
    st.markdown("Create a new observation proposal:")
    
    with st.form("new_proposal_form"):
        name = st.text_input(
            "Proposal Name",
            placeholder="e.g., Galaxy Survey 2026",
            key="new_proposal_name",
        )
        
        description = st.text_area(
            "Description (optional)",
            placeholder="Brief description of the proposal objectives...",
            key="new_proposal_desc",
        )
        
        submitted = st.form_submit_button("Create Proposal", type="primary")
        
        if submitted:
            if not name:
                st.error("Please provide a proposal name")
            else:
                new_proposal = Proposal(
                    id=str(uuid.uuid4()),
                    name=name,
                    description=description,
                )
                add_proposal(new_proposal)
                set_selected_proposal(new_proposal.id)
                st.success(f"Created proposal: {name}")
                st.rerun()


def _render_add_task_form() -> None:
    """Render form for adding a task to a proposal."""
    proposals = get_proposals()
    
    if not proposals:
        st.warning("Create a proposal first before adding tasks!")
        return
    
    # Proposal selector
    selected = get_selected_proposal()
    proposal_options = {p.name: p.id for p in proposals}
    
    default_idx = 0
    if selected:
        try:
            default_idx = list(proposal_options.values()).index(selected.id)
        except ValueError:
            default_idx = 0
    
    selected_name = st.selectbox(
        "Add to Proposal",
        options=list(proposal_options.keys()),
        index=default_idx,
        key="task_proposal_select",
    )
    selected_proposal_id = proposal_options[selected_name]
    
    st.markdown("---")
    st.markdown("#### Task Configuration")
    
    with st.form("add_task_form"):
        # Basic info
        task_name = st.text_input(
            "Task Name",
            placeholder="e.g., M51 Observation",
            key="task_name",
        )
        
        col1, col2 = st.columns(2)
        with col1:
            priority = st.slider(
                "Priority",
                min_value=0.0,
                max_value=10.0,
                value=5.0,
                step=0.1,
                key="task_priority",
                help="Higher priority = scheduled first",
            )
        with col2:
            duration = st.number_input(
                "Duration (hours)",
                min_value=0.1,
                max_value=12.0,
                value=1.0,
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
                value=0.0,
                step=0.1,
                key="task_ra",
                help="Right Ascension in degrees (0-360)",
            )
        with coord_col2:
            dec_deg = st.number_input(
                "Dec (degrees)",
                min_value=-90.0,
                max_value=90.0,
                value=0.0,
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
                value=0.5,
                step=0.1,
                key="task_min_obs",
            )
            
            alt_col1, alt_col2 = st.columns(2)
            with alt_col1:
                min_alt = st.number_input(
                    "Min Altitude (°)",
                    min_value=0.0,
                    max_value=90.0,
                    value=20.0,
                    key="task_min_alt",
                )
            with alt_col2:
                max_alt = st.number_input(
                    "Max Altitude (°)",
                    min_value=0.0,
                    max_value=90.0,
                    value=85.0,
                    key="task_max_alt",
                )
            
            az_col1, az_col2 = st.columns(2)
            with az_col1:
                min_az = st.number_input(
                    "Min Azimuth (°)",
                    min_value=0.0,
                    max_value=360.0,
                    value=0.0,
                    key="task_min_az",
                )
            with az_col2:
                max_az = st.number_input(
                    "Max Azimuth (°)",
                    min_value=0.0,
                    max_value=360.0,
                    value=360.0,
                    key="task_max_az",
                )
        
        submitted = st.form_submit_button("Add Task", type="primary")
        
        if submitted:
            if not task_name:
                st.error("Please provide a task name")
            else:
                new_task = ObservationTask(
                    id=str(uuid.uuid4()),
                    name=task_name,
                    priority=priority,
                    duration_hours=duration,
                    min_observation_hours=min_obs if 'min_obs' in dir() else 0.5,
                    ra_deg=ra_deg,
                    dec_deg=dec_deg,
                    min_altitude=min_alt if 'min_alt' in dir() else 20.0,
                    max_altitude=max_alt if 'max_alt' in dir() else 85.0,
                    min_azimuth=min_az if 'min_az' in dir() else 0.0,
                    max_azimuth=max_az if 'max_az' in dir() else 360.0,
                )
                add_task_to_proposal(selected_proposal_id, new_task)
                st.success(f"Added task: {task_name}")
                st.rerun()


def export_proposals_to_schedule_json() -> dict[str, Any]:
    """
    Export all proposals to backend-compatible schedule JSON.
    
    The format matches the expected schema:
    - name: schedule name
    - checksum: optional checksum
    - dark_periods: list of {start, stop} MJD periods
    - blocks: list of scheduling blocks
    
    Returns:
        Dictionary in schedule.json format.
    """
    import hashlib
    
    proposals = get_proposals()
    all_blocks = []
    
    for proposal in proposals:
        for task in proposal.tasks:
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
