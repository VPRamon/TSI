"""
Model evaluation metrics and analysis.
"""

import logging
from typing import Any

import numpy as np
from sklearn.metrics import (
    accuracy_score,
    average_precision_score,
    brier_score_loss,
    confusion_matrix,
    f1_score,
    log_loss,
    precision_score,
    recall_score,
    roc_auc_score,
)

logger = logging.getLogger(__name__)


class ModelEvaluator:
    """Comprehensive model evaluation."""

    def __init__(self, config: Any) -> None:
        self.config = config
        self.results: dict[str, Any] = {}

    def evaluate(self, model: Any, X: Any, y: Any, dataset_name: str = "test") -> dict[str, Any]:
        """
        Evaluate model performance.

        Args:
            model: Trained model
            X: Features
            y: True labels
            dataset_name: Name of dataset (train/val/test)

        Returns:
            Dictionary of metrics
        """
        # Predictions
        y_pred = model.predict(X)
        y_proba = model.predict_proba(X)[:, 1]

        # Calculate metrics
        metrics = {
            "roc_auc": roc_auc_score(y, y_proba),
            "pr_auc": average_precision_score(y, y_proba),
            "brier_score": brier_score_loss(y, y_proba),
            "accuracy": accuracy_score(y, y_pred),
            "precision": precision_score(y, y_pred, zero_division=0),
            "recall": recall_score(y, y_pred, zero_division=0),
            "f1": f1_score(y, y_pred, zero_division=0),
            "log_loss": log_loss(y, y_proba),
        }

        # Confusion matrix
        cm = confusion_matrix(y, y_pred)
        metrics["confusion_matrix"] = cm

        # Store results
        self.results[dataset_name] = metrics

        # Log summary
        logger.info(f"{dataset_name.upper()} Metrics:")
        for metric, value in metrics.items():
            if metric != "confusion_matrix":
                logger.info(f"  {metric}: {value:.4f}")

        return metrics

    def find_optimal_threshold(self, y_true: Any, y_proba: Any) -> float:
        """
        Find optimal classification threshold.

        Args:
            y_true: True labels
            y_proba: Predicted probabilities

        Returns:
            Optimal threshold
        """
        method = self.config.get("evaluation.threshold.method", "cost_minimization")

        if method == "cost_minimization":
            C_FP = self.config.get("evaluation.costs.false_positive", 1.0)
            C_FN = self.config.get("evaluation.costs.false_negative", 5.0)

            thresholds = np.linspace(0.1, 0.9, 100)
            costs = []

            for threshold in thresholds:
                y_pred = (y_proba >= threshold).astype(int)
                tn, fp, fn, tp = confusion_matrix(y_true, y_pred).ravel()
                cost = C_FP * fp + C_FN * fn
                costs.append(cost)

            optimal_idx = np.argmin(costs)
            optimal_threshold: float = float(thresholds[optimal_idx])

            logger.info(
                f"Optimal threshold: {optimal_threshold:.3f} (min cost: {costs[optimal_idx]:.2f})"
            )

        else:
            # Default to 0.5
            optimal_threshold = 0.5

        return optimal_threshold
