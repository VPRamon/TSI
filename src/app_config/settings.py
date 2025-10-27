"""Centralized application configuration using pydantic-settings."""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Declarative configuration for adapters and presenters."""

    data_root: Path = Path("data")
    sample_dataset: Path = Path("data") / "schedule.csv"
    artifacts_dir: Path = Path("src/tsi/modeling/artifacts")
    cache_ttl_seconds: int = 600

    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8", extra="ignore")


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    """Return a cached Settings instance."""

    return Settings()
