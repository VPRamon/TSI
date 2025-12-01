"""Centralized application configuration using pydantic-settings.

This module provides a comprehensive configuration system for the TSI application,
supporting environment variables, .env files, and sensible defaults.

Configuration Categories:
- Database: Connection strings, authentication flags
- Data Paths: Base directories and sample data locations
- UI: Application title, page configuration, theme settings
- Performance: Cache TTL, worker limits, optimization flags
- Feature Flags: Enable/disable optional features

Environment Variables:
- DATABASE_URL: Full database connection string
- DB_SERVER, DB_DATABASE, DB_USERNAME, DB_PASSWORD: Individual DB components
- USE_AAD_AUTH: Enable Azure AD authentication (default: False)
- DATA_ROOT: Base directory for data files (default: "data")
- CACHE_TTL: Cache time-to-live in seconds (default: 3600)
- MAX_WORKERS: Maximum worker threads/processes (default: 4)

Example:
    >>> from app_config import get_settings
    >>> settings = get_settings()
    >>> print(settings.app_title)
    'Telescope Scheduling Intelligence'
    >>> print(settings.database_url)
    'mssql://...'
"""

from __future__ import annotations

import logging
from functools import lru_cache
from pathlib import Path
from typing import Optional

from pydantic import Field, field_validator
from pydantic_settings import BaseSettings, SettingsConfigDict

logger = logging.getLogger(__name__)


class Settings(BaseSettings):
    """
    Centralized application configuration.
    
    All settings can be overridden via environment variables or a .env file.
    Environment variables take precedence over .env file values.
    """

    # ===== Database Configuration =====
    database_url: Optional[str] = Field(
        default=None,
        description="Full database connection string (e.g., mssql://user:pass@host/db)",
    )
    db_server: Optional[str] = Field(
        default=None,
        description="Database server hostname or IP",
    )
    db_database: Optional[str] = Field(
        default=None,
        description="Database name",
    )
    db_username: Optional[str] = Field(
        default=None,
        description="Database username",
    )
    db_password: Optional[str] = Field(
        default=None,
        description="Database password",
    )
    use_aad_auth: bool = Field(
        default=False,
        description="Use Azure Active Directory authentication",
    )
    database_connection_timeout: int = Field(
        default=30,
        description="Database connection timeout in seconds",
        ge=1,
    )
    database_max_retries: int = Field(
        default=3,
        description="Maximum number of retry attempts for transient database errors",
        ge=0,
    )

    # ===== Data Paths =====
    data_root: Path = Field(
        default=Path("data"),
        description="Base directory for data files",
    )
    sample_dataset: Path = Field(
        default=Path("data") / "schedule.csv",
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
    enable_database: bool = Field(
        default=True,
        description="Enable database features (schedule storage/retrieval)",
    )
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
        description="Use pre-computed analytics table for improved query performance",
    )
    
    # ===== Database Migration Configuration =====
    data_source: str = Field(
        default="legacy",
        description="Data source for database operations: 'legacy' (normalized tables) or 'etl' (analytics tables)",
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

    @field_validator("data_source")
    @classmethod
    def validate_data_source(cls, v: str) -> str:
        """Validate data_source is either 'legacy' or 'etl'."""
        v = v.lower()
        if v not in ("legacy", "etl"):
            raise ValueError(f"data_source must be 'legacy' or 'etl', got '{v}'")
        return v

    def get_database_url(self) -> Optional[str]:
        """
        Get the database connection URL.
        
        Returns the full database_url if set, otherwise constructs one from
        individual components (db_server, db_database, etc.).
        
        Returns:
            Database connection URL or None if not configured
        """
        if self.database_url:
            return self.database_url
        
        # Construct from components if available
        if self.db_server and self.db_database:
            if self.use_aad_auth:
                # Azure AD authentication
                return f"mssql://{self.db_server}/{self.db_database}?trusted_connection=yes"
            elif self.db_username and self.db_password:
                # Username/password authentication
                return f"mssql://{self.db_username}:{self.db_password}@{self.db_server}/{self.db_database}"
        
        return None

    def get_plot_margin(self) -> dict[str, int]:
        """Get plot margins as a dictionary."""
        return {
            "l": self.plot_margin_left,
            "r": self.plot_margin_right,
            "t": self.plot_margin_top,
            "b": self.plot_margin_bottom,
        }

    def validate_database_config(self) -> bool:
        """
        Validate that database configuration is sufficient.
        
        Returns:
            True if database is properly configured, False otherwise
        """
        return self.get_database_url() is not None


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
    
    # Log configuration status (but don't expose sensitive info)
    if settings.validate_database_config():
        logger.info("Database configuration loaded successfully")
    else:
        logger.warning("Database configuration is incomplete or missing")
    
    logger.info(f"Data root: {settings.data_root}")
    logger.info(f"Cache TTL: {settings.cache_ttl}s")
    logger.info(f"Rust backend enabled: {settings.enable_rust_backend}")
    logger.info(f"Data source: {settings.data_source}")
    
    return settings
