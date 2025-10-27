"""
Local (instance-level) model explanations.
"""

import logging
from typing import Any

import numpy as np

logger = logging.getLogger(__name__)


class LocalExplainer:
    """Explain individual predictions."""

    def __init__(
        self, config: Any, model: Any, preprocessor: Any, explainer: Any | None = None
    ) -> None:
        self.config = config
        self.model = model
        self.preprocessor = preprocessor
        self.explainer = explainer

    def explain_observation(
        self, observation_data: Any, feature_names: list[str]
    ) -> dict[str, Any]:
        """
        Explain a single observation prediction.

        Args:
            observation_data: Single observation features (array)
            feature_names: List of feature names

        Returns:
            Dictionary with prediction and explanation
        """
        # Predict
        y_proba = self.model.predict_proba([observation_data])[0, 1]
        y_pred = int(y_proba >= 0.5)

        # Get SHAP values for this observation
        if self.explainer is not None:
            shap_values = self.explainer.shap_values([observation_data])
            if isinstance(shap_values, list):
                shap_values = shap_values[1][0]
            else:
                shap_values = shap_values[0]
        else:
            shap_values = np.zeros(len(feature_names))

        # Create explanation
        explanation = {
            "predicted_class": y_pred,
            "probability_scheduled": y_proba,
            "probability_not_scheduled": 1 - y_proba,
            "decision": "PLANIFICADA" if y_pred == 1 else "NO PLANIFICADA",
            "feature_contributions": dict(zip(feature_names, shap_values)),
        }

        return explanation

    def generate_explanation_text(self, explanation: dict[str, Any], top_n: int = 5) -> str:
        """
        Generate human-readable explanation text.

        Args:
            explanation: Explanation dictionary
            top_n: Number of top factors to include

        Returns:
            Formatted explanation string
        """
        decision = explanation["decision"]
        probability = explanation["probability_scheduled"]

        # Sort features by absolute SHAP value
        contributions = explanation["feature_contributions"]
        sorted_features = sorted(contributions.items(), key=lambda x: abs(x[1]), reverse=True)[
            :top_n
        ]

        # Build explanation text
        text = f"Observation: {decision}\n"
        text += f"Scheduling probability: {probability:.2%}\n\n"
        text += "Top factors:\n"

        for feature, shap_value in sorted_features:
            direction = "supports" if shap_value > 0 else "hinders"
            impact = (
                "high" if abs(shap_value) > 0.5 else "medium" if abs(shap_value) > 0.2 else "low"
            )
            text += f"  • {feature}: {direction} scheduling (impact {impact}, {shap_value:+.3f})\n"

        # Add recommendation
        text += "\nRecommendation: "
        if decision == "NOT SCHEDULED":
            top_negative = [f for f, v in sorted_features if v < 0]
            if top_negative:
                text += f"Improve: {', '.join(top_negative[:2])}"
            else:
                text += "Review time constraints and priority"
        else:
            text += "Observation well positioned for scheduling"

        return text

    def explain_multiple(
        self, observations: Any, feature_names: list[str], n_samples: int = 10
    ) -> list[dict[str, Any]]:
        """
        Explain multiple observations.

        Args:
            observations: Array of observations
            feature_names: List of feature names
            n_samples: Number of observations to explain

        Returns:
            List of explanations
        """
        explanations = []

        for i in range(min(n_samples, len(observations))):
            obs = observations[i]
            explanation = self.explain_observation(obs, feature_names)
            explanation["text"] = self.generate_explanation_text(explanation)
            explanations.append(explanation)

        logger.info(f"Generated {len(explanations)} local explanations")

        return explanations
