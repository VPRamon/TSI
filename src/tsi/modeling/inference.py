"""
Inference script: Predict and explain individual observations.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any

import joblib
import pandas as pd

from src.tsi.modeling.config import ModelConfig
from src.tsi.modeling.data.feature_engineering import AstronomicalFeatureEngineer
from src.tsi.modeling.data.preprocessor import SchedulingPreprocessor
from src.tsi.modeling.explainability.local_explainer import LocalExplainer

# Add project root to path
project_root = Path(__file__).parents[3]
sys.path.insert(0, str(project_root))


class ObservationPredictor:
    """Predict scheduling probability and explain decisions."""

    def __init__(self, artifacts_dir: str | None = None) -> None:
        """
        Initialize predictor.

        Args:
            artifacts_dir: Directory containing saved artifacts
        """
        self.config = ModelConfig()

        if artifacts_dir is None:
            artifacts_dir = str(self.config.get_paths()["output_dir"])

        self.artifacts_dir = Path(artifacts_dir)

        # Load artifacts
        self.model = self._load_model()
        self.preprocessor = self._load_preprocessor()
        self.threshold = self._load_threshold()
        self.feature_engineer = AstronomicalFeatureEngineer(self.config)

        # Create explainer
        try:
            import shap

            self.shap_explainer: Any | None = shap.TreeExplainer(self.model)
        except Exception:
            self.shap_explainer = None

        self.explainer = LocalExplainer(
            self.config, self.model, self.preprocessor, self.shap_explainer
        )

    def _load_model(self) -> Any:
        """Load trained model."""
        model_path = self.artifacts_dir / "models" / "best_model.pkl"
        return joblib.load(model_path)

    def _load_preprocessor(self) -> SchedulingPreprocessor:
        """Load fitted preprocessor."""
        preprocessor_path = self.artifacts_dir / "preprocessors" / "preprocessor.pkl"
        return SchedulingPreprocessor.load(str(preprocessor_path), self.config)

    def _load_threshold(self) -> float:
        """Load optimal decision threshold."""
        threshold_path = self.artifacts_dir / "threshold.json"
        with open(threshold_path) as f:
            data = json.load(f)
        return float(data["optimal_threshold"])

    def predict_explain(self, observation_df: pd.DataFrame) -> dict[str, Any]:
        """
        Predict and explain a single observation.

        Args:
            observation_df: DataFrame with observation features

        Returns:
            Dictionary with prediction and explanation
        """
        # Feature engineering
        observation_df = self.feature_engineer.create_all_features(observation_df)

        # Preprocess
        X = self.preprocessor.transform(observation_df)

        # Predict
        y_proba = self.model.predict_proba(X)[0, 1]
        y_pred = int(y_proba >= self.threshold)

        # Explain
        feature_names_out_raw = self.preprocessor.feature_names_out
        if feature_names_out_raw is not None:
            feature_names = (
                feature_names_out_raw.tolist()
                if hasattr(feature_names_out_raw, "tolist")
                else list(feature_names_out_raw)
            )
        else:
            feature_names = []

        explanation = self.explainer.explain_observation(X[0], feature_names)
        explanation_text = self.explainer.generate_explanation_text(explanation)

        # Format result
        result: dict[str, Any] = {
            "observation_id": observation_df.get("Science Target", ["Unknown"])[0],
            "predicted_scheduled": bool(y_pred),
            "probability_scheduled": float(y_proba),
            "decision_threshold": float(self.threshold),
            "explanation": explanation_text,
            "top_factors": [],
        }

        # Extract top factors
        contributions = explanation["feature_contributions"]
        sorted_features = sorted(contributions.items(), key=lambda x: abs(x[1]), reverse=True)[:10]

        for feature, shap_value in sorted_features:
            if not isinstance(result["top_factors"], list):
                result["top_factors"] = []
            result["top_factors"].append(
                {
                    "feature": feature,
                    "shap_value": float(shap_value),
                    "direction": "positive" if shap_value > 0 else "negative",
                }
            )

        return result


def main() -> None:
    """Command-line interface."""
    parser = argparse.ArgumentParser(description="Predict and explain observation scheduling")
    parser.add_argument(
        "--observation", type=str, help="Path to observation data (CSV file)", required=True
    )
    parser.add_argument(
        "--output", type=str, help="Path to save results (JSON)", default="prediction_result.json"
    )
    parser.add_argument(
        "--artifacts-dir", type=str, help="Directory with model artifacts", default=None
    )

    args = parser.parse_args()

    # Load observation
    observation_df = pd.read_csv(args.observation)

    # Create predictor
    predictor = ObservationPredictor(args.artifacts_dir)

    # Predict and explain
    result = predictor.predict_explain(observation_df)

    # Save result
    with open(args.output, "w") as f:
        json.dump(result, f, indent=2)

    # Print to console
    print("\n" + "=" * 80)
    print("PREDICTION AND EXPLANATION")
    print("=" * 80)
    print(f"\nObservation: {result['observation_id']}")
    print(f"Decision: {'SCHEDULED' if result['predicted_scheduled'] else 'NOT SCHEDULED'}")
    print(f"Probability: {result['probability_scheduled']:.2%}")
    print(f"\n{result['explanation']}")
    print("\n" + "=" * 80)
    print(f"\nResults saved to: {args.output}")


if __name__ == "__main__":
    main()
