"""Global pytest configuration and cross-cutting fixtures."""

from __future__ import annotations

import os
import random
import sys
from collections.abc import Callable, Iterator
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np
import pytest
import responses

if TYPE_CHECKING:
    from streamlit.testing.v1 import AppTest

PROJECT_ROOT = Path(__file__).resolve().parents[1]
SRC_PATH = PROJECT_ROOT / "src"

if str(SRC_PATH) not in sys.path:
    sys.path.insert(0, str(SRC_PATH))


@pytest.fixture(autouse=True)
def seed_rng() -> None:
    """Seed Python and NumPy RNGs for deterministic tests."""

    random.seed(1337)
    np.random.seed(1337)


@pytest.fixture
def env_vars(monkeypatch: pytest.MonkeyPatch) -> Callable[[dict[str, str]], None]:
    """Temporarily set environment variables for the duration of a test."""

    def _apply(values: dict[str, str]) -> None:
        for key, value in values.items():
            monkeypatch.setenv(key, value)

    return _apply


@pytest.fixture
def http_mock() -> Iterator[responses.RequestsMock]:
    """Provide a requests-mock style responses mock for HTTP boundaries."""

    with responses.RequestsMock(assert_all_requests_are_fired=False) as mock:
        yield mock


@pytest.fixture
def app_test_factory(monkeypatch: pytest.MonkeyPatch) -> Callable[[str], AppTest]:
    """Return a factory that instantiates :class:`AppTest` with project paths."""

    from streamlit.testing.v1 import AppTest

    def _factory(script: str = "src/tsi/app.py") -> AppTest:
        script_path = (PROJECT_ROOT / script).resolve()
        monkeypatch.setenv("STREAMLIT_SERVER_PORT", "0")
        pythonpath = os.environ.get("PYTHONPATH", "")
        if str(SRC_PATH) not in pythonpath.split(":"):
            monkeypatch.setenv(
                "PYTHONPATH", f"{SRC_PATH}:{pythonpath}" if pythonpath else str(SRC_PATH)
            )

        return AppTest.from_file(str(script_path))

    return _factory
