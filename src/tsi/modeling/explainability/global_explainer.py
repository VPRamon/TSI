"""
Global model explainability using SHAP and other methods.
"""

import logging
from typing import Any

import numpy as np
import pandas as pd
import shap
from sklearn.inspection import permutation_importance

logger = logging.getLogger(__name__)


class GlobalExplainer:
    """Global model interpretation and explanation."""

    def __init__(self, config: Any) -> None:
        self.config = config
        self.explanations: dict[str, Any] = {}

    def feature_importance_permutation(
        self, model: Any, X: Any, y: Any, feature_names: list[str]
    ) -> pd.DataFrame | None:
        """Calculate permutation feature importance."""
        if not self.config.get("explainability.global.permutation_importance.enabled", True):
            return None

        logger.info("Calculating permutation importance...")

        n_repeats = self.config.get("explainability.global.permutation_importance.n_repeats", 10)

        result = permutation_importance(
            model,
            X,
            y,
            n_repeats=n_repeats,
            random_state=self.config.get("global.random_seed", 42),
            n_jobs=-1,
        )

        importance_df = pd.DataFrame(
            {
                "feature": feature_names,
                "importance_mean": result.importances_mean,
                "importance_std": result.importances_std,
            }
        ).sort_values("importance_mean", ascending=False)

        self.explanations["permutation_importance"] = importance_df

        logger.info("Top 5 features by permutation importance:")
        for idx, row in importance_df.head().iterrows():
            logger.info(
                f"  {row['feature']}: {row['importance_mean']:.4f} ± {row['importance_std']:.4f}"
            )

        return importance_df

    def shap_summary(self, model: Any, X: Any, feature_names: list[str]) -> np.ndarray | None:
        """Generate SHAP summary."""
        if not self.config.get("explainability.global.shap_summary.enabled", True):
            return None

        logger.info("Calculating SHAP values...")

        max_samples = self.config.get("explainability.global.shap_summary.max_samples", 1000)

        # Sample data if too large
        if len(X) > max_samples:
            indices = np.random.choice(len(X), max_samples, replace=False)
            X_sample = X[indices]
        else:
            X_sample = X

        # Create explainer (TreeExplainer for tree models, KernelExplainer for others)
        try:
            explainer = shap.TreeExplainer(model)
        except Exception:
            explainer = shap.KernelExplainer(model.predict_proba, X_sample[:100])

        shap_values = explainer.shap_values(X_sample)

        # For binary classification, handle different SHAP output formats
        if isinstance(shap_values, list):
            # List of arrays [class_0_shap, class_1_shap]
            shap_values = shap_values[1]  # Get positive class
        elif isinstance(shap_values, np.ndarray):
            if len(shap_values.shape) == 3:  # Shape is [n_classes, n_samples, n_features]
                shap_values = shap_values[1]  # Get positive class (index 1)
            elif len(shap_values.shape) == 2:
                # Check if it's [features, samples] instead of [samples, features]
                if shap_values.shape[0] < shap_values.shape[1]:
                    shap_values = shap_values.T  # Transpose to [samples, features]

        self.explanations["shap_values"] = shap_values
        self.explanations["shap_base_value"] = explainer.expected_value

        # Calculate mean absolute SHAP values
        mean_abs_shap = np.abs(shap_values).mean(axis=0)

        # Ensure mean_abs_shap is 1D
        if mean_abs_shap.ndim > 1:
            mean_abs_shap = mean_abs_shap.flatten()

        # Debug: check lengths
        logger.info(
            f"feature_names length: {len(feature_names)}, mean_abs_shap length: {len(mean_abs_shap)}"
        )
        logger.info(
            f"mean_abs_shap shape: {mean_abs_shap.shape}, shap_values shape: {shap_values.shape}"
        )

        # Ensure lengths match
        if len(feature_names) != len(mean_abs_shap):
            logger.warning("Length mismatch: truncating to min length")
            min_len = min(len(feature_names), len(mean_abs_shap))
            feature_names = feature_names[:min_len]
            mean_abs_shap = mean_abs_shap[:min_len]

        shap_importance = pd.DataFrame(
            {"feature": feature_names, "mean_abs_shap": mean_abs_shap}
        ).sort_values("mean_abs_shap", ascending=False)

        self.explanations["shap_importance"] = shap_importance

        logger.info("SHAP values calculated")

        return np.asarray(shap_values) if shap_values is not None else None

    def save_explanations(self, path: str) -> None:
        """Save global explanations."""
        import joblib

        joblib.dump(self.explanations, path)
        logger.info(f"Global explanations saved to {path}")
