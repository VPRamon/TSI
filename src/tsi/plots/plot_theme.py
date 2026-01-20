"""Centralized plot theme and styling configuration.

This module provides unified styling for all Plotly visualizations in the TSI application.
It ensures visual consistency across all plots including:
- Background colors
- Grid styling
- Legend placement and styling
- Color palettes
- Font configurations
- Axis styling
- Hover template styling

Usage:
    from tsi.plots.plot_theme import PlotTheme, apply_theme

    fig = go.Figure(...)
    apply_theme(fig)  # Apply standard theme

    # Or access specific theme values:
    fig.update_layout(
        plot_bgcolor=PlotTheme.PLOT_BG,
        paper_bgcolor=PlotTheme.PAPER_BG,
    )
"""

from __future__ import annotations

from typing import Any

import plotly.graph_objects as go

from tsi.config import PLOT_HEIGHT, PLOT_MARGIN


class PlotTheme:
    """Centralized theme configuration for all plots."""

    # ===== Background Colors =====
    PLOT_BG = "rgba(14, 17, 23, 0.3)"  # Plot area background
    PAPER_BG = "rgba(0, 0, 0, 0)"  # Surrounding paper background (transparent)

    # ===== Grid Styling =====
    GRID_COLOR = "rgba(100, 100, 100, 0.3)"
    GRID_WIDTH = 1

    # ===== Primary Color Palette =====
    # Used for categorical data (scheduled/unscheduled, comparisons)
    PRIMARY_BLUE = "#1f77b4"  # Scheduled, current schedule
    PRIMARY_ORANGE = "#ff7f0e"  # Unscheduled, comparison schedule
    SUCCESS_GREEN = "#2ca02c"  # Positive changes, success states
    ERROR_RED = "#d62728"  # Negative changes, error states
    PURPLE = "#9467bd"
    BROWN = "#8c564b"
    PINK = "#e377c2"
    GRAY = "#7f7f7f"
    OLIVE = "#bcbd22"
    CYAN = "#17becf"

    # Border/line variants (darker versions for outlines)
    PRIMARY_BLUE_DARK = "#0d5a9e"
    PRIMARY_ORANGE_DARK = "#cc6600"
    SUCCESS_GREEN_DARK = "#1a7a1a"
    ERROR_RED_DARK = "#8b1a1a"

    # ===== Default Color Sequence =====
    COLOR_SEQUENCE = [
        PRIMARY_BLUE,
        PRIMARY_ORANGE,
        SUCCESS_GREEN,
        ERROR_RED,
        PURPLE,
        BROWN,
        PINK,
        GRAY,
        OLIVE,
        CYAN,
    ]

    # ===== Font Configuration =====
    FONT_FAMILY = "Arial, sans-serif"
    FONT_SIZE = 12
    TITLE_FONT_SIZE = 14
    TICK_FONT_SIZE = 11
    LEGEND_FONT_SIZE = 12
    TEXT_COLOR = "white"
    TITLE_COLOR = "white"

    # ===== Legend Styling =====
    LEGEND_BG = "rgba(14, 17, 23, 0.7)"
    LEGEND_BORDER_COLOR = "rgba(100, 100, 100, 0.5)"
    LEGEND_BORDER_WIDTH = 1

    # ===== Marker Styling =====
    MARKER_OPACITY = 0.8
    MARKER_LINE_WIDTH = 1
    MARKER_LINE_COLOR = "white"

    # ===== Bar Chart Styling =====
    BAR_LINE_WIDTH = 1
    BAR_LINE_COLOR = "white"

    # ===== Hover Styling =====
    HOVERLABEL_BG = "rgba(14, 17, 23, 0.9)"
    HOVERLABEL_BORDER_COLOR = "rgba(100, 100, 100, 0.5)"
    HOVERLABEL_FONT_SIZE = 12


def get_standard_layout() -> dict[str, Any]:
    """
    Get standard layout configuration dictionary.

    Returns:
        Dictionary of layout parameters that can be used with fig.update_layout()
    """
    return {
        "plot_bgcolor": PlotTheme.PLOT_BG,
        "paper_bgcolor": PlotTheme.PAPER_BG,
        "font": {
            "family": PlotTheme.FONT_FAMILY,
            "size": PlotTheme.FONT_SIZE,
            "color": PlotTheme.TEXT_COLOR,
        },
        "title": {
            "font": {
                "size": PlotTheme.TITLE_FONT_SIZE,
                "color": PlotTheme.TITLE_COLOR,
            },
        },
        "hoverlabel": {
            "bgcolor": PlotTheme.HOVERLABEL_BG,
            "bordercolor": PlotTheme.HOVERLABEL_BORDER_COLOR,
            "font": {
                "size": PlotTheme.HOVERLABEL_FONT_SIZE,
                "color": PlotTheme.TEXT_COLOR,
            },
        },
    }


def get_horizontal_legend(y_offset: float = 1.02) -> dict[str, Any]:
    """
    Get standard horizontal legend configuration (positioned above plot).

    Args:
        y_offset: Vertical position offset above plot (default 1.02)

    Returns:
        Dictionary of legend parameters
    """
    return {
        "orientation": "h",
        "yanchor": "bottom",
        "y": y_offset,
        "xanchor": "right",
        "x": 1,
        "font": {
            "size": PlotTheme.LEGEND_FONT_SIZE,
            "color": PlotTheme.TEXT_COLOR,
        },
        "bgcolor": PlotTheme.LEGEND_BG,
        "bordercolor": PlotTheme.LEGEND_BORDER_COLOR,
        "borderwidth": PlotTheme.LEGEND_BORDER_WIDTH,
    }


def get_vertical_legend() -> dict[str, Any]:
    """
    Get standard vertical legend configuration (positioned to the right of plot).

    Returns:
        Dictionary of legend parameters
    """
    return {
        "orientation": "v",
        "yanchor": "top",
        "y": 1,
        "xanchor": "left",
        "x": 1.02,
        "font": {
            "size": PlotTheme.LEGEND_FONT_SIZE,
            "color": PlotTheme.TEXT_COLOR,
        },
        "bgcolor": PlotTheme.LEGEND_BG,
        "bordercolor": PlotTheme.LEGEND_BORDER_COLOR,
        "borderwidth": PlotTheme.LEGEND_BORDER_WIDTH,
    }


def get_axis_config(title: str | None = None, show_grid: bool = True) -> dict[str, Any]:
    """
    Get standard axis configuration.

    Args:
        title: Axis title text
        show_grid: Whether to show gridlines

    Returns:
        Dictionary of axis parameters
    """
    config: dict[str, Any] = {
        "showgrid": show_grid,
        "gridcolor": PlotTheme.GRID_COLOR,
        "gridwidth": PlotTheme.GRID_WIDTH,
        "tickfont": {
            "size": PlotTheme.TICK_FONT_SIZE,
            "color": PlotTheme.TEXT_COLOR,
        },
    }
    if title:
        config["title"] = {
            "text": title,
            "font": {
                "size": PlotTheme.FONT_SIZE,
                "color": PlotTheme.TEXT_COLOR,
            },
        }
    return config


def apply_theme(
    fig: go.Figure,
    height: int | None = None,
    margin: dict[str, int] | None = None,
    legend_style: str = "horizontal",
    show_legend: bool = True,
) -> go.Figure:
    """
    Apply standard theme to a Plotly figure.

    This is the primary function for ensuring consistent plot styling.
    It applies background colors, fonts, legend styling, and hover styling.

    Args:
        fig: Plotly figure to style
        height: Plot height (defaults to PLOT_HEIGHT from config)
        margin: Plot margins (defaults to PLOT_MARGIN from config)
        legend_style: "horizontal" or "vertical" legend positioning
        show_legend: Whether to show the legend

    Returns:
        The styled figure (same object, modified in place)
    """
    layout_config = get_standard_layout()

    # Set dimensions
    layout_config["height"] = height if height is not None else PLOT_HEIGHT
    layout_config["margin"] = margin if margin is not None else PLOT_MARGIN

    # Set legend
    if show_legend:
        if legend_style == "horizontal":
            layout_config["legend"] = get_horizontal_legend()
        else:
            layout_config["legend"] = get_vertical_legend()
        layout_config["showlegend"] = True
    else:
        layout_config["showlegend"] = False

    fig.update_layout(**layout_config)

    return fig


def get_histogram_marker(
    color: str = PlotTheme.PRIMARY_BLUE,
    line_color: str = PlotTheme.MARKER_LINE_COLOR,
    opacity: float = PlotTheme.MARKER_OPACITY,
) -> dict[str, Any]:
    """
    Get standard histogram marker configuration.

    Args:
        color: Fill color
        line_color: Border line color
        opacity: Marker opacity

    Returns:
        Dictionary of marker parameters
    """
    return {
        "color": color,
        "line": {"color": line_color, "width": PlotTheme.BAR_LINE_WIDTH},
        "opacity": opacity,
    }


def get_bar_marker(
    color: str = PlotTheme.PRIMARY_BLUE,
    line_color: str | None = None,
) -> dict[str, Any]:
    """
    Get standard bar chart marker configuration.

    Args:
        color: Fill color
        line_color: Border line color (defaults to darker variant)

    Returns:
        Dictionary of marker parameters
    """
    # Use darker variant if no line color specified
    if line_color is None:
        color_map = {
            PlotTheme.PRIMARY_BLUE: PlotTheme.PRIMARY_BLUE_DARK,
            PlotTheme.PRIMARY_ORANGE: PlotTheme.PRIMARY_ORANGE_DARK,
            PlotTheme.SUCCESS_GREEN: PlotTheme.SUCCESS_GREEN_DARK,
            PlotTheme.ERROR_RED: PlotTheme.ERROR_RED_DARK,
        }
        line_color = color_map.get(color, color)

    return {
        "color": color,
        "line": {"color": line_color, "width": 2},
    }


def get_scatter_marker(
    size: int | list[int] = 10,
    color: str = PlotTheme.PRIMARY_BLUE,
    opacity: float = 0.7,
) -> dict[str, Any]:
    """
    Get standard scatter plot marker configuration.

    Args:
        size: Marker size (single value or list)
        color: Marker color
        opacity: Marker opacity

    Returns:
        Dictionary of marker parameters
    """
    return {
        "size": size,
        "color": color,
        "opacity": opacity,
        "line": {"width": 0.5, "color": "white"},
    }


def get_box_marker(
    color: str = PlotTheme.PRIMARY_BLUE,
    line_color: str | None = None,
) -> dict[str, Any]:
    """
    Get standard box/violin plot marker configuration.

    Args:
        color: Fill color
        line_color: Border line color (defaults to darker variant)

    Returns:
        Dictionary of marker and line parameters
    """
    if line_color is None:
        color_map = {
            PlotTheme.PRIMARY_BLUE: PlotTheme.PRIMARY_BLUE_DARK,
            PlotTheme.PRIMARY_ORANGE: PlotTheme.PRIMARY_ORANGE_DARK,
        }
        line_color = color_map.get(color, color)

    return {
        "marker": {"color": color, "line": {"color": line_color, "width": 2}},
        "fillcolor": color,
        "line": {"color": line_color, "width": 2},
    }


def get_colorscale_config(
    colorscale: str = "Viridis",
    colorbar_title: str = "Value",
) -> dict[str, Any]:
    """
    Get standard colorscale configuration for continuous data.

    Args:
        colorscale: Name of colorscale ('Viridis', 'RdBu', etc.)
        colorbar_title: Title for the colorbar

    Returns:
        Dictionary of colorbar parameters
    """
    return {
        "colorscale": colorscale,
        "colorbar": {
            "title": {
                "text": colorbar_title,
                "font": {"color": PlotTheme.TEXT_COLOR},
            },
            "tickfont": {"color": PlotTheme.TEXT_COLOR},
        },
    }


# Convenience aliases for common color pairs
SCHEDULED_COLOR = PlotTheme.PRIMARY_BLUE
UNSCHEDULED_COLOR = PlotTheme.PRIMARY_ORANGE
CURRENT_SCHEDULE_COLOR = PlotTheme.PRIMARY_BLUE
COMPARISON_SCHEDULE_COLOR = PlotTheme.PRIMARY_ORANGE
POSITIVE_CHANGE_COLOR = PlotTheme.SUCCESS_GREEN
NEGATIVE_CHANGE_COLOR = PlotTheme.ERROR_RED
