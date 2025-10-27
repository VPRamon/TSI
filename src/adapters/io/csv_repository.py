"""CSV repository adapter."""

from __future__ import annotations

from pathlib import Path
from typing import Protocol

import pandas as pd

from app_config.settings import get_settings


class DataRepository(Protocol):
    """Protocol for repositories that load/save pandas DataFrames."""

    def load(self, relative_path: str) -> pd.DataFrame: ...

    def save(self, df: pd.DataFrame, relative_path: str) -> None: ...


class CSVRepository:
    """Simple filesystem-backed CSV adapter."""

    def __init__(self, *, base_path: Path | None = None) -> None:
        settings = get_settings()
        self._base_path = base_path or settings.data_root

    def load(self, relative_path: str) -> pd.DataFrame:
        full_path = self._resolve(relative_path)
        if not full_path.exists():
            raise FileNotFoundError(full_path)
        return pd.read_csv(full_path)

    def save(self, df: pd.DataFrame, relative_path: str) -> None:
        full_path = self._resolve(relative_path)
        full_path.parent.mkdir(parents=True, exist_ok=True)
        df.to_csv(full_path, index=False)

    def _resolve(self, relative_path: str) -> Path:
        candidate = Path(relative_path)
        if candidate.is_absolute():
            return candidate
        return self._base_path / candidate
