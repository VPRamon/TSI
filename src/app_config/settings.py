"""Centralized application configuration using pydantic-settings.

This module provides a comprehensive configuration system for the TSI application,
supporting environment variables, .env files, and sensible defaults.

Configuration Categories:
- Data Paths: Base directories and sample data locations
- UI: Application title, page configuration, theme settings
- Performance: Cache TTL, worker limits, optimization flags
- Feature Flags: Enable/disable optional features

Environment Variables:
- DATA_ROOT: Base directory for data files (default: "data")
- CACHE_TTL: Cache time-to-live in seconds (default: 3600)
- MAX_WORKERS: Maximum worker threads/processes (default: 4)

Note:
    Database configuration is handled by the Rust backend. Set the following
    environment variables for database access:
    - DB_SERVER, DB_DATABASE, DB_USERNAME, DB_PASSWORD
    See rust_backend/src/db/config.rs for full database configuration options.

Example:
    >>> from app_config import get_settings
    >>> settings = get_settings()
    >>> print(settings.app_title)
    'Telescope Scheduling Intelligence'
"""

from __future__ import annotations

import logging
from functools import lru_cache
from pathlib import Path

from pydantic import Field, field_validator
from pydantic_settings import BaseSettings, SettingsConfigDict

logger = logging.getLogger(__name__)


class Settings(BaseSettings):
    """
    Centralized application configuration.

    All settings can be overridden via environment variables or a .env file.
    Environment variables take precedence over .env file values.
    """

    # ===== Data Paths =====
    data_root: Path = Field(
        default=Path("data"),
        description="Base directory for data files",
    )
    sample_dataset: Path = Field(
        default=Path("data") / "schedule.json",
        description="Default sample schedule dataset path",
    )
    artifacts_dir: Path = Field(
        default=Path("src/tsi/modeling/artifacts"),
        description="Directory for ML model artifacts",
    )

    # ===== UI Configuration =====
    app_title: str = Field(
        default="Telescope Scheduling Intelligence",
        description="Application title displayed in browser and UI",
    )
    app_icon: str = Field(
        default="🔭",
        description="Application icon (emoji)",
    )
    layout: str = Field(
        default="wide",
        description="Streamlit layout mode",
    )
    initial_sidebar_state: str = Field(
        default="collapsed",
        description="Initial sidebar state",
    )
    pages: list[str] = Field(
        default=[
            "Validation",
            "Sky Map",
            "Distributions",
            "Visibility Map",
            "Schedule",
            "Insights",
            "Trends",
            "Compare",
        ],
        description="Available pages in the application",
    )

    # ===== Performance Settings =====
    cache_ttl: int = Field(
        default=3600,
        description="Cache time-to-live in seconds (1 hour)",
        ge=0,
    )
    max_workers: int = Field(
        default=4,
        description="Maximum worker threads/processes for parallel operations",
        ge=1,
    )
    enable_rust_backend: bool = Field(
        default=True,
        description="Use high-performance Rust backend for data operations",
    )

    # ===== Plot Defaults =====
    plot_height: int = Field(
        default=600,
        description="Default plot height in pixels",
        ge=100,
    )
    plot_margin_left: int = Field(default=80, ge=0)
    plot_margin_right: int = Field(default=80, ge=0)
    plot_margin_top: int = Field(default=80, ge=0)
    plot_margin_bottom: int = Field(default=80, ge=0)

    # ===== Feature Flags =====
    enable_file_upload: bool = Field(
        default=True,
        description="Enable file upload functionality",
    )
    enable_comparison: bool = Field(
        default=True,
        description="Enable schedule comparison feature",
    )
    use_analytics_table: bool = Field(
        default=True,
        description="Use pre-computed analytics table for improved query performance (ETL)",
    )

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
        case_sensitive=False,
    )

    @field_validator("data_root", "sample_dataset", "artifacts_dir", mode="before")
    @classmethod
    def convert_to_path(cls, v):
        """Convert string paths to Path objects."""
        if isinstance(v, str):
            return Path(v)
        return v

    def get_plot_margin(self) -> dict[str, int]:
        """Get plot margins as a dictionary."""
        return {
            "l": self.plot_margin_left,
            "r": self.plot_margin_right,
            "t": self.plot_margin_top,
            "b": self.plot_margin_bottom,
        }


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    """
    Return a cached Settings instance.

    This function caches the settings to avoid re-reading environment variables
    and .env files on every call.

    Returns:
        Cached Settings instance
    """
    settings = Settings()

    # Log configuration status
    logger.info(f"Data root: {settings.data_root}")
    logger.info(f"Cache TTL: {settings.cache_ttl}s")
    logger.info(f"Rust backend enabled: {settings.enable_rust_backend}")
    logger.info("Note: Database configuration is managed by Rust backend via environment variables")

    return settings
