"""
Configuration loader for centralized parameter management.
"""

import logging
from pathlib import Path
from typing import Any

import yaml

logger = logging.getLogger(__name__)


class ModelConfig:
    """
    Centralized configuration management for the modeling pipeline.

    Loads and validates configuration from YAML file, provides easy access
    to all parameters with dot notation.
    """

    def __init__(self, config_path: str | None = None) -> None:
        """
        Initialize configuration.

        Args:
            config_path: Path to YAML configuration file.
                        If None, uses default location.
        """
        if config_path is None:
            default_path: Path = Path(__file__).parent / "model_config.yaml"
            self.config_path: Path = default_path
        else:
            self.config_path = Path(config_path)
        self.config = self._load_config()
        self._validate_config()
        self._setup_logging()

    def _load_config(self) -> dict[str, Any]:
        """Load configuration from YAML file."""
        try:
            with open(self.config_path) as f:
                loaded_config = yaml.safe_load(f)
                config: dict[str, Any] = loaded_config if loaded_config is not None else {}
            logger.info(f"Configuration loaded from {self.config_path}")
            return config
        except Exception as e:
            logger.error(f"Failed to load configuration: {e}")
            raise

    def _validate_config(self) -> None:
        """Validate required configuration sections exist."""
        required_sections = [
            "global",
            "paths",
            "data",
            "temporal_split",
            "features",
            "models",
            "evaluation",
            "explainability",
        ]

        for section in required_sections:
            if section not in self.config:
                raise ValueError(f"Missing required configuration section: {section}")

        logger.info("Configuration validated successfully")

    def _setup_logging(self) -> None:
        """Setup logging based on configuration."""
        log_config: dict[str, Any] = self.config.get("logging", {})
        level = getattr(logging, log_config.get("level", "INFO"))
        format_str = log_config.get("format", "%(asctime)s - %(levelname)s - %(message)s")

        logging.basicConfig(
            level=level,
            format=format_str,
            handlers=[
                logging.StreamHandler(),
                logging.FileHandler(log_config.get("file", "training.log")),
            ],
        )

    def get(self, key: str, default: Any = None) -> Any:
        """
        Get configuration value using dot notation.

        Args:
            key: Configuration key (e.g., 'models.logistic_regression.C')
            default: Default value if key not found

        Returns:
            Configuration value
        """
        keys = key.split(".")
        value: Any = self.config

        for k in keys:
            if isinstance(value, dict):
                value = value.get(k)
                if value is None:
                    return default
            else:
                return default

        return value

    def set(self, key: str, value: Any) -> None:
        """
        Set configuration value using dot notation.

        Args:
            key: Configuration key
            value: Value to set
        """
        keys = key.split(".")
        config: dict[str, Any] = self.config

        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            next_config = config[k]
            if not isinstance(next_config, dict):
                config[k] = {}
                next_config = config[k]
            config = next_config

        config[keys[-1]] = value
        logger.debug(f"Configuration updated: {key} = {value}")

    def get_model_config(self, model_name: str) -> dict[str, Any]:
        """Get configuration for specific model, excluding non-sklearn params."""
        models = self.config.get("models", {})
        if not isinstance(models, dict):
            return {}
        model_config_raw = models.get(model_name, {})
        if not isinstance(model_config_raw, dict):
            return {}
        model_config = model_config_raw.copy()
        # Remove non-sklearn parameters
        model_config.pop("enabled", None)
        return model_config  # type: ignore[return-value]

    def get_feature_names(self) -> list[str]:
        """Get list of all feature names from configuration."""
        features = self.config["features"]
        all_features: list[str] = []

        for category in ["astronomical", "operational", "coordinates", "interactions"]:
            if category in features:
                all_features.extend(features[category])

        return all_features

    def get_paths(self) -> dict[str, Path]:
        """Get all paths as Path objects."""
        paths: dict[str, Any] = self.config["paths"]
        return {k: Path(v) for k, v in paths.items()}

    def save_config(self, output_path: str | None = None) -> None:
        """
        Save current configuration to file.

        Args:
            output_path: Where to save. If None, overwrites original.
        """
        save_path: Path
        if output_path is None:
            save_path = self.config_path
        else:
            save_path = Path(output_path)

        with open(save_path, "w") as f:
            yaml.dump(self.config, f, default_flow_style=False, indent=2)

        logger.info(f"Configuration saved to {save_path}")

    def __repr__(self) -> str:
        return f"ModelConfig(config_path='{self.config_path}')"

    def __str__(self) -> str:
        return yaml.dump(self.config, default_flow_style=False, indent=2)
