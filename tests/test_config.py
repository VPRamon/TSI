"""
Comprehensive tests for centralized configuration system.

Tests cover:
- Default configuration values
- Environment variable overrides
- Configuration validation
- Settings caching

Note: Database configuration has been moved to the Rust backend.
Database-related tests have been removed as Python no longer manages DB config.
"""

import os
from pathlib import Path
from unittest.mock import patch

import pytest

from app_config.settings import Settings, get_settings


class TestSettingsDefaults:
    """Test default configuration values."""

    def test_data_path_defaults(self):
        """Test data path configuration defaults."""
        settings = Settings()
        
        assert settings.data_root == Path("data")
        assert settings.sample_dataset == Path("data") / "schedule.json"
        assert settings.artifacts_dir == Path("src/tsi/modeling/artifacts")

    def test_ui_defaults(self):
        """Test UI configuration defaults."""
        settings = Settings()
        
        assert settings.app_title == "Telescope Scheduling Intelligence"
        assert settings.app_icon == "🔭"
        assert settings.layout == "wide"
        assert settings.initial_sidebar_state == "collapsed"
        assert len(settings.pages) == 7
        assert "Sky Map" in settings.pages
        assert "Distributions" in settings.pages

    def test_performance_defaults(self):
        """Test performance configuration defaults."""
        settings = Settings()
        
        assert settings.cache_ttl == 3600
        assert settings.max_workers == 4
        assert settings.enable_rust_backend is True

    def test_plot_defaults(self):
        """Test plot configuration defaults."""
        settings = Settings()
        
        assert settings.plot_height == 600
        assert settings.plot_margin_left == 80
        assert settings.plot_margin_right == 80
        assert settings.plot_margin_top == 80
        assert settings.plot_margin_bottom == 80

    def test_feature_flags_defaults(self):
        """Test feature flag defaults."""
        settings = Settings()
        
        assert settings.enable_database is True
        assert settings.enable_file_upload is True
        assert settings.enable_comparison is True


class TestEnvironmentVariableOverrides:
    """Test environment variable overrides."""

    def test_cache_ttl_override(self):
        """Test CACHE_TTL environment variable."""
        with patch.dict(os.environ, {"CACHE_TTL": "7200"}):
            settings = Settings()
            assert settings.cache_ttl == 7200

    def test_max_workers_override(self):
        """Test MAX_WORKERS environment variable."""
        with patch.dict(os.environ, {"MAX_WORKERS": "8"}):
            settings = Settings()
            assert settings.max_workers == 8

    def test_data_root_override(self):
        """Test DATA_ROOT environment variable."""
        with patch.dict(os.environ, {"DATA_ROOT": "/custom/data/path"}):
            settings = Settings()
            assert settings.data_root == Path("/custom/data/path")


class TestPlotMargins:
    """Test plot margin dictionary construction."""

    def test_get_plot_margin(self):
        """Test that plot margins are returned as a dictionary."""
        settings = Settings()
        margins = settings.get_plot_margin()
        
        assert isinstance(margins, dict)
        assert margins["l"] == 80
        assert margins["r"] == 80
        assert margins["t"] == 80
        assert margins["b"] == 80

    def test_get_plot_margin_custom(self):
        """Test plot margins with custom values."""
        settings = Settings(
            plot_margin_left=100,
            plot_margin_right=50,
            plot_margin_top=120,
            plot_margin_bottom=60
        )
        margins = settings.get_plot_margin()
        
        assert margins["l"] == 100
        assert margins["r"] == 50
        assert margins["t"] == 120
        assert margins["b"] == 60


class TestSettingsCaching:
    """Test settings caching behavior."""

    def test_get_settings_returns_same_instance(self):
        """Test that get_settings() returns the same cached instance."""
        # Clear the cache first
        get_settings.cache_clear()
        
        settings1 = get_settings()
        settings2 = get_settings()
        
        assert settings1 is settings2

    def test_get_settings_returns_settings_instance(self):
        """Test that get_settings() returns a Settings instance."""
        settings = get_settings()
        assert isinstance(settings, Settings)


class TestPathConversion:
    """Test path string to Path object conversion."""

    def test_data_root_string_conversion(self):
        """Test that string paths are converted to Path objects."""
        settings = Settings(data_root="custom/data")
        assert isinstance(settings.data_root, Path)
        assert settings.data_root == Path("custom/data")

    def test_sample_dataset_string_conversion(self):
        """Test that sample dataset path is converted."""
        settings = Settings(sample_dataset="custom/schedule.json")
        assert isinstance(settings.sample_dataset, Path)
        assert settings.sample_dataset == Path("custom/schedule.json")


class TestValidationConstraints:
    """Test pydantic validation constraints."""

    def test_database_connection_timeout_positive(self):
        """Test that connection timeout must be positive."""
        with pytest.raises(ValueError):
            Settings(database_connection_timeout=0)

    def test_database_max_retries_non_negative(self):
        """Test that max retries can be zero or positive."""
        settings = Settings(database_max_retries=0)
        assert settings.database_max_retries == 0

    def test_cache_ttl_non_negative(self):
        """Test that cache TTL can be zero (no cache)."""
        settings = Settings(cache_ttl=0)
        assert settings.cache_ttl == 0

    def test_max_workers_positive(self):
        """Test that max workers must be positive."""
        with pytest.raises(ValueError):
            Settings(max_workers=0)

    def test_plot_height_minimum(self):
        """Test that plot height has a minimum value."""
        with pytest.raises(ValueError):
            Settings(plot_height=50)  # Less than 100


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
