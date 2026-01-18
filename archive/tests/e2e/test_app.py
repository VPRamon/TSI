"""End-to-end tests for the Streamlit application."""

from __future__ import annotations

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
    assert any("Upload" in markdown.value for markdown in app.markdown)


def test_app_launch__with_missing_sample_dataset__shows_error_message(
    app_test_factory,
    env_vars,
) -> None:
    """Clicking the sample dataset button should surface meaningful errors."""
    pytest.skip(
        "UI changed: App no longer has sample dataset button in landing page. "
        "Test needs migration to new JSON upload workflow."
    )
