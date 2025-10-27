"""
Model training and management.
"""

import logging
from pathlib import Path
from typing import Any

import joblib
from sklearn.base import BaseEstimator
from sklearn.calibration import CalibratedClassifierCV
from sklearn.ensemble import GradientBoostingClassifier, RandomForestClassifier
from sklearn.linear_model import LogisticRegression

logger = logging.getLogger(__name__)


class ModelTrainer:
    """Train and manage scheduling prediction models."""

    def __init__(self, config: Any) -> None:
        self.config = config
        self.models: dict[str, Any] = {}

    def create_logistic_regression(self) -> LogisticRegression:
        """Create interpretable logistic regression model."""
        cfg = self.config.get_model_config("logistic_regression")
        return LogisticRegression(**cfg)

    def create_random_forest(self) -> RandomForestClassifier:
        """Create random forest model."""
        cfg = self.config.get_model_config("random_forest")
        return RandomForestClassifier(**cfg)

    def create_gradient_boosting(self) -> GradientBoostingClassifier:
        """Create gradient boosting model."""
        cfg = self.config.get_model_config("gradient_boosting")
        return GradientBoostingClassifier(**cfg)

    def train(
        self, X_train: Any, y_train: Any, model_type: str = "logistic_regression"
    ) -> BaseEstimator:
        """
        Train a model.

        Args:
            X_train: Training features
            y_train: Training labels
            model_type: Type of model to train

        Returns:
            Trained model
        """
        logger.info(f"Training {model_type} model...")

        if model_type == "logistic_regression":
            model = self.create_logistic_regression()
        elif model_type == "random_forest":
            model = self.create_random_forest()
        elif model_type == "gradient_boosting":
            model = self.create_gradient_boosting()
        else:
            raise ValueError(f"Unknown model type: {model_type}")

        model.fit(X_train, y_train)

        logger.info(f"{model_type} training complete")

        self.models[model_type] = model
        return model

    def calibrate_model(self, model: BaseEstimator, X_val: Any, y_val: Any) -> BaseEstimator:
        """
        Calibrate model probabilities.

        Args:
            model: Trained model
            X_val: Validation features
            y_val: Validation labels

        Returns:
            Calibrated model
        """
        if not self.config.get("calibration.enabled", True):
            return model

        method = self.config.get("calibration.method", "sigmoid")

        logger.info(f"Calibrating model using {method} method...")

        calibrated = CalibratedClassifierCV(
            model, method=method, cv="prefit"  # Use pre-fitted model
        )

        calibrated.fit(X_val, y_val)

        logger.info("Model calibration complete")

        return calibrated

    def train_all_models(
        self, X_train: Any, y_train: Any, X_val: Any | None = None, y_val: Any | None = None
    ) -> dict[str, Any]:
        """
        Train all enabled models.

        Args:
            X_train: Training features
            y_train: Training labels
            X_val: Validation features (for calibration)
            y_val: Validation labels (for calibration)

        Returns:
            Dictionary of trained models
        """
        model_configs = self.config.get("models", {})

        for model_type, cfg in model_configs.items():
            if cfg.get("enabled", False):
                model = self.train(X_train, y_train, model_type)

                # Calibrate if validation data provided
                if X_val is not None and y_val is not None:
                    model = self.calibrate_model(model, X_val, y_val)
                    self.models[f"{model_type}_calibrated"] = model

        logger.info(f"Trained {len(self.models)} models")

        return self.models

    def save_model(self, model: Any, path: str, model_name: str | None = None) -> None:
        """Save model to disk."""
        save_path = Path(path)
        save_path.parent.mkdir(parents=True, exist_ok=True)

        joblib.dump(model, save_path)
        logger.info(f"Model saved to {save_path}")

    @staticmethod
    def load_model(path: str) -> Any:
        """Load model from disk."""
        return joblib.load(path)
