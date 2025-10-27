"""End-to-end tests for the Streamlit application."""

from __future__ import annotations

from pathlib import Path

import pytest

pytestmark = pytest.mark.e2e


def test_app_launch__without_dataset__renders_landing_page(app_test_factory) -> None:
    """Launching the app without data should show the landing hero section."""

    # Given: a freshly initialized Streamlit application
    app = app_test_factory()

    # When: running the app without uploading data
    app.run()

    # Then: the landing page headline should be rendered
    assert any("Telescope Scheduling Intelligence" in markdown.value for markdown in app.markdown)
    assert any("Upload CSV" in markdown.value for markdown in app.markdown)


def test_app_launch__with_missing_sample_dataset__shows_error_message(
    app_test_factory,
    env_vars,
) -> None:
    """Clicking the sample dataset button should surface meaningful errors."""

    # Given: a bogus sample dataset path
    env_vars({"SAMPLE_DATASET": str(Path("/nonexistent/sample.csv"))})
    from app_config import get_settings

    get_settings.cache_clear()
    app = app_test_factory()
    app.run()

    # When: triggering the sample dataset loading
    app.button[0].click()
    app.run()

    # Then: the UI should show an explicit error message
    error_messages = [error.body for error in app.error]
    assert any(
        "Sample data file not found" in message or "Missing required columns" in message
        for message in error_messages
    )
