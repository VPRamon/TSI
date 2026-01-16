"""Proposal canvas component for visualizing proposals as a tree structure.

This module renders proposals and their tasks as a tree-like graph
using Plotly for interactive visualization.
"""

from __future__ import annotations

import logging
from typing import Any

import plotly.graph_objects as go
import streamlit as st

from tsi.components.creative.proposal_builder import (
    ObservationTask,
    Proposal,
    get_proposals,
    get_selected_proposal,
    set_selected_proposal,
)

logger = logging.getLogger(__name__)


def render_proposal_canvas() -> None:
    """
    Render the proposal canvas showing proposals as a tree structure.
    
    The visualization shows:
    - Root node: "Schedule"
    - Intermediate nodes: Proposals (and Sequences in future)
    - Leaf nodes: Observation Tasks
    """
    st.markdown("### 🌳 Proposal Structure")
    
    proposals = get_proposals()
    
    if not proposals:
        _render_empty_canvas()
        return
    
    # Build tree data
    tree_data = _build_tree_data(proposals)
    
    # Create and display the tree visualization
    fig = _create_tree_figure(tree_data)
    
    # Display with Plotly
    st.plotly_chart(
        fig,
        key="proposal_tree_chart",
    )
    
    # Legend
    _render_legend()


def _render_empty_canvas() -> None:
    """Render placeholder when no proposals exist."""
    st.markdown(
        """
        <div style="
            border: 2px dashed #666;
            border-radius: 10px;
            padding: 60px 20px;
            text-align: center;
            color: #888;
            margin: 20px 0;
        ">
            <h3>📋 No Proposals Yet</h3>
            <p>Create proposals using the Proposal Builder panel.</p>
            <p style="font-size: 0.9em;">
                Proposals will appear here as a tree structure with<br>
                <b>Proposals</b> as intermediate nodes and <b>Tasks</b> as leaves.
            </p>
        </div>
        """,
        unsafe_allow_html=True,
    )
    
    # TODO: Sequences placeholder
    st.caption(
        "💡 *Future feature: Sequences will allow grouping tasks that must be executed together.*"
    )


def _build_tree_data(proposals: list[Proposal]) -> dict[str, Any]:
    """
    Build tree structure data for visualization.
    
    Returns:
        Dictionary with nodes and edges for the tree.
    """
    nodes = []
    edges = []
    
    # Root node
    root_id = "root"
    nodes.append({
        "id": root_id,
        "label": "📅 Schedule",
        "type": "root",
        "x": 0,
        "y": 0,
        "color": "#4CAF50",
        "size": 40,
    })
    
    # Calculate positions for proposals
    n_proposals = len(proposals)
    proposal_spacing = 2.0
    proposal_start_x = -((n_proposals - 1) * proposal_spacing) / 2
    
    for i, proposal in enumerate(proposals):
        proposal_x = proposal_start_x + i * proposal_spacing
        proposal_y = -1
        
        proposal_node_id = f"proposal_{proposal.id}"
        nodes.append({
            "id": proposal_node_id,
            "label": f"📁 {proposal.name}",
            "type": "proposal",
            "x": proposal_x,
            "y": proposal_y,
            "color": "#2196F3",
            "size": 30,
            "proposal_id": proposal.id,
        })
        edges.append((root_id, proposal_node_id))
        
        # TODO: Add sequence nodes here when implemented
        # Sequences would be intermediate nodes between proposals and tasks
        
        # Add task nodes
        n_tasks = len(proposal.tasks)
        if n_tasks > 0:
            task_spacing = min(1.5, proposal_spacing * 0.8 / max(1, n_tasks - 1))
            task_start_x = proposal_x - ((n_tasks - 1) * task_spacing) / 2
            
            for j, task in enumerate(proposal.tasks):
                task_x = task_start_x + j * task_spacing
                task_y = -2
                
                task_node_id = f"task_{task.id}"
                
                # Color based on priority
                task_color = _priority_to_color(task.priority)
                
                nodes.append({
                    "id": task_node_id,
                    "label": f"🎯 {task.name}",
                    "type": "task",
                    "x": task_x,
                    "y": task_y,
                    "color": task_color,
                    "size": 20,
                    "task": task,
                })
                edges.append((proposal_node_id, task_node_id))
    
    return {"nodes": nodes, "edges": edges}


def _priority_to_color(priority: float) -> str:
    """Convert priority value to color."""
    if priority >= 8.0:
        return "#f44336"  # Red - high priority
    elif priority >= 5.0:
        return "#FF9800"  # Orange - medium priority
    elif priority >= 3.0:
        return "#FFEB3B"  # Yellow - low-medium priority
    else:
        return "#8BC34A"  # Light green - low priority


def _create_tree_figure(tree_data: dict[str, Any]) -> go.Figure:
    """
    Create a Plotly figure for the tree visualization.
    
    Args:
        tree_data: Dictionary with nodes and edges.
    
    Returns:
        Plotly Figure object.
    """
    nodes = tree_data["nodes"]
    edges = tree_data["edges"]
    
    # Create edge traces
    edge_x = []
    edge_y = []
    
    node_map = {n["id"]: n for n in nodes}
    
    for source_id, target_id in edges:
        source = node_map[source_id]
        target = node_map[target_id]
        edge_x.extend([source["x"], target["x"], None])
        edge_y.extend([source["y"], target["y"], None])
    
    edge_trace = go.Scatter(
        x=edge_x,
        y=edge_y,
        mode="lines",
        line=dict(width=2, color="#888"),
        hoverinfo="none",
    )
    
    # Create node traces (separate by type for different styling)
    node_traces = []
    
    for node in nodes:
        # Create hover text
        if node["type"] == "task":
            task = node["task"]
            hover_text = (
                f"<b>{task.name}</b><br>"
                f"Priority: {task.priority:.1f}<br>"
                f"Duration: {task.duration_hours:.1f}h<br>"
                f"RA: {task.ra_deg:.2f}°, Dec: {task.dec_deg:.2f}°"
            )
        elif node["type"] == "proposal":
            hover_text = f"<b>{node['label']}</b><br>Click to select"
        else:
            hover_text = node["label"]
        
        node_trace = go.Scatter(
            x=[node["x"]],
            y=[node["y"]],
            mode="markers+text",
            marker=dict(
                size=node["size"],
                color=node["color"],
                line=dict(width=2, color="#fff"),
            ),
            text=[node["label"]],
            textposition="bottom center",
            hovertemplate=hover_text + "<extra></extra>",
            name=node["type"],
            showlegend=False,
        )
        node_traces.append(node_trace)
    
    # Create figure
    fig = go.Figure(
        data=[edge_trace] + node_traces,
        layout=go.Layout(
            showlegend=False,
            hovermode="closest",
            margin=dict(b=20, l=20, r=20, t=20),
            xaxis=dict(
                showgrid=False,
                zeroline=False,
                showticklabels=False,
            ),
            yaxis=dict(
                showgrid=False,
                zeroline=False,
                showticklabels=False,
            ),
            plot_bgcolor="rgba(0,0,0,0)",
            paper_bgcolor="rgba(0,0,0,0)",
            height=400,
        ),
    )
    
    return fig


def _render_legend() -> None:
    """Render legend for the tree visualization."""
    col1, col2, col3, col4 = st.columns(4)
    
    with col1:
        st.markdown("🟢 **Root** (Schedule)")
    with col2:
        st.markdown("🔵 **Proposal** (Group)")
    with col3:
        st.markdown("🎯 **Task** (Observation)")
    with col4:
        st.markdown("🔴 High / 🟠 Med / 🟡 Low Priority")
    
    # Summary stats
    proposals = get_proposals()
    total_tasks = sum(len(p.tasks) for p in proposals)
    total_hours = sum(
        sum(t.duration_hours for t in p.tasks)
        for p in proposals
    )
    
    st.markdown("---")
    cols = st.columns(3)
    with cols[0]:
        st.metric("Proposals", len(proposals))
    with cols[1]:
        st.metric("Total Tasks", total_tasks)
    with cols[2]:
        st.metric("Total Hours", f"{total_hours:.1f}")
