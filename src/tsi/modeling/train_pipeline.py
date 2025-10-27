"""
Main training pipeline for the observation scheduling explainability model.
"""

import json
import logging
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

import numpy as np

from src.tsi.modeling.config import ModelConfig
from src.tsi.modeling.data.data_loader import SchedulingDataLoader
from src.tsi.modeling.data.feature_engineering import AstronomicalFeatureEngineer
from src.tsi.modeling.data.preprocessor import SchedulingPreprocessor
from src.tsi.modeling.data.temporal_split import TemporalSplitter
from src.tsi.modeling.explainability.global_explainer import GlobalExplainer
from src.tsi.modeling.explainability.local_explainer import LocalExplainer
from src.tsi.modeling.models.evaluator import ModelEvaluator
from src.tsi.modeling.models.trainer import ModelTrainer

# Add project root to path
project_root = Path(__file__).parents[3]
sys.path.insert(0, str(project_root))

# Setup logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


class TrainingPipeline:
    """
    Complete training pipeline for the scheduling explainability model.

    Pipeline steps:
    1. Load configuration
    2. Load and prepare data
    3. Temporal train/val/test split
    4. Feature engineering
    5. Preprocessing
    6. Model training (multiple models)
    7. Model calibration
    8. Evaluation
    9. Threshold optimization
    10. Global explainability (SHAP, feature importance)
    11. Local explanations (example cases)
    12. Save artifacts (models, preprocessor, threshold, reports)
    """

    def __init__(self, config_path: str | None = None) -> None:
        """
        Initialize pipeline.

        Args:
            config_path: Path to configuration file
        """
        self.config = ModelConfig(config_path)
        self.artifacts: dict[str, Any] = {}
        self.metrics: dict[str, Any] = {}

        # Create output directories
        paths = self.config.get_paths()
        for path in paths.values():
            path.mkdir(parents=True, exist_ok=True)

    def run(self) -> dict[str, Any]:
        """Execute complete training pipeline."""
        logger.info("=" * 80)
        logger.info("TELESCOPE SCHEDULING EXPLAINABILITY MODEL - TRAINING PIPELINE")
        logger.info("=" * 80)
        logger.info(f"Start time: {datetime.now()}")
        logger.info(f"Configuration: {self.config.config_path}")

        # Step 1: Load data
        logger.info("\n" + "=" * 80)
        logger.info("STEP 1: Data Loading")
        logger.info("=" * 80)
        data_loader = SchedulingDataLoader(self.config)
        df = data_loader.prepare_dataset()
        logger.info(f"Loaded dataset: {df.shape}")

        # Step 2: Temporal split
        logger.info("\n" + "=" * 80)
        logger.info("STEP 2: Temporal Data Split")
        logger.info("=" * 80)
        splitter = TemporalSplitter(self.config)

        # Use configured split method
        split_method = self.config.get("temporal_split.method", "continuous")
        if split_method == "period":
            time_col = self.config.get("temporal_split.period_column", "iteration")
            train_df, val_df, test_df = splitter.split_by_period(df, time_col)
        else:  # continuous
            time_col = self.config.get("temporal_split.time_column", "schedulingBlockId")
            train_df, val_df, test_df = splitter.split(df, time_col)

        # Step 3: Feature engineering
        logger.info("\n" + "=" * 80)
        logger.info("STEP 3: Feature Engineering")
        logger.info("=" * 80)
        engineer = AstronomicalFeatureEngineer(self.config)

        train_df = engineer.create_all_features(train_df)
        val_df = engineer.create_all_features(val_df)
        test_df = engineer.create_all_features(test_df)

        feature_names = engineer.get_feature_names()
        logger.info(f"Total features created: {len(feature_names)}")

        # Save feature metadata
        metadata = engineer.get_feature_metadata()
        metadata_path = self.config.get_paths()["output_dir"] / "feature_metadata.json"
        with open(metadata_path, "w") as f:
            json.dump(metadata, f, indent=2)

        # Step 4: Preprocessing
        logger.info("\n" + "=" * 80)
        logger.info("STEP 4: Data Preprocessing")
        logger.info("=" * 80)
        preprocessor = SchedulingPreprocessor(self.config)

        X_train = preprocessor.fit_transform(train_df, "planificada")
        X_val = preprocessor.transform(val_df)
        X_test = preprocessor.transform(test_df)

        y_train = train_df["planificada"].values
        y_val = val_df["planificada"].values
        y_test = test_df["planificada"].values

        logger.info(f"Training set: X={X_train.shape}, y={y_train.shape}")
        logger.info(f"Validation set: X={X_val.shape}, y={y_val.shape}")
        logger.info(f"Test set: X={X_test.shape}, y={y_test.shape}")

        # Save preprocessor
        preprocessor_path = self.config.get_paths()["preprocessor_dir"] / "preprocessor.pkl"
        preprocessor.save(str(preprocessor_path))
        self.artifacts["preprocessor"] = preprocessor

        # Step 5: Model training
        logger.info("\n" + "=" * 80)
        logger.info("STEP 5: Model Training")
        logger.info("=" * 80)
        trainer = ModelTrainer(self.config)
        models = trainer.train_all_models(X_train, y_train, X_val, y_val)

        # Step 6: Model evaluation
        logger.info("\n" + "=" * 80)
        logger.info("STEP 6: Model Evaluation")
        logger.info("=" * 80)
        evaluator = ModelEvaluator(self.config)

        best_model = None
        best_model_name = None
        best_score = 0

        for model_name, model in models.items():
            logger.info(f"\nEvaluating {model_name}...")

            _ = evaluator.evaluate(model, X_train, y_train, f"{model_name}_train")
            val_metrics = evaluator.evaluate(model, X_val, y_val, f"{model_name}_val")
            _ = evaluator.evaluate(model, X_test, y_test, f"{model_name}_test")

            # Select best model by validation ROC-AUC
            if val_metrics["roc_auc"] > best_score:
                best_score = val_metrics["roc_auc"]
                best_model = model
                best_model_name = model_name

        logger.info(f"\nBest model: {best_model_name} (Val ROC-AUC: {best_score:.4f})")

        # Save best model
        model_path = self.config.get_paths()["models_dir"] / "best_model.pkl"
        trainer.save_model(best_model, str(model_path), best_model_name)
        self.artifacts["best_model"] = best_model
        self.artifacts["best_model_name"] = best_model_name

        # Step 7: Threshold optimization
        logger.info("\n" + "=" * 80)
        logger.info("STEP 7: Threshold Optimization")
        logger.info("=" * 80)

        if best_model is not None:
            y_val_proba = best_model.predict_proba(X_val)[:, 1]
        else:
            raise ValueError("No valid model trained")

        optimal_threshold = evaluator.find_optimal_threshold(y_val, y_val_proba)

        threshold_path = self.config.get_paths()["output_dir"] / "threshold.json"
        with open(threshold_path, "w") as f:
            json.dump({"optimal_threshold": float(optimal_threshold)}, f, indent=2)

        self.artifacts["threshold"] = optimal_threshold

        # Step 8: Global explainability
        logger.info("\n" + "=" * 80)
        logger.info("STEP 8: Global Explainability")
        logger.info("=" * 80)

        global_explainer = GlobalExplainer(self.config)

        # Permutation importance
        feature_names_out_raw = preprocessor.feature_names_out
        if feature_names_out_raw is not None:
            feature_names_out = (
                feature_names_out_raw.tolist()
                if hasattr(feature_names_out_raw, "tolist")
                else list(feature_names_out_raw)
            )
        else:
            feature_names_out = []

        perm_importance = global_explainer.feature_importance_permutation(
            best_model, X_test, y_test, feature_names_out
        )

        # SHAP values
        _ = global_explainer.shap_summary(best_model, X_test, feature_names_out)

        # Save global explanations
        global_exp_path = self.config.get_paths()["output_dir"] / "global_explanations.pkl"
        global_explainer.save_explanations(str(global_exp_path))

        # Step 9: Local explanations (examples)
        logger.info("\n" + "=" * 80)
        logger.info("STEP 9: Local Explanations (Example Cases)")
        logger.info("=" * 80)

        # Create SHAP explainer for local explanations
        try:
            import shap

            shap_explainer = shap.TreeExplainer(best_model)
        except Exception:
            shap_explainer = None

        local_explainer = LocalExplainer(self.config, best_model, preprocessor, shap_explainer)

        # Find example cases
        if best_model is not None:
            y_test_pred = best_model.predict(X_test)
        else:
            raise ValueError("No valid model for predictions")

        # Not scheduled examples
        not_scheduled_idx = np.where((y_test == 0) & (y_test_pred == 0))[0][:5]
        scheduled_idx = np.where((y_test == 1) & (y_test_pred == 1))[0][:5]

        example_explanations = []

        logger.info("\nExample NOT SCHEDULED cases:")
        for idx in not_scheduled_idx:
            explanation = local_explainer.explain_observation(X_test[idx], feature_names_out)
            explanation["text"] = local_explainer.generate_explanation_text(explanation)
            example_explanations.append(explanation)
            logger.info(f"\n{explanation['text']}")

        logger.info("\nExample SCHEDULED cases:")
        for idx in scheduled_idx:
            explanation = local_explainer.explain_observation(X_test[idx], feature_names_out)
            explanation["text"] = local_explainer.generate_explanation_text(explanation)
            example_explanations.append(explanation)
            logger.info(f"\n{explanation['text']}")

        # Save example explanations
        examples_path = self.config.get_paths()["output_dir"] / "example_explanations.json"
        with open(examples_path, "w") as f:
            # Convert to serializable format
            serializable_examples = []
            for exp in example_explanations:
                serializable_examples.append(
                    {
                        "decision": exp["decision"],
                        "probability": float(exp["probability_scheduled"]),
                        "text": exp["text"],
                    }
                )
            json.dump(serializable_examples, f, indent=2)

        # Step 10: Generate summary report
        logger.info("\n" + "=" * 80)
        logger.info("STEP 10: Summary Report")
        logger.info("=" * 80)

        summary = {
            "pipeline_version": self.config.get("global.version"),
            "training_date": datetime.now().isoformat(),
            "best_model": best_model_name,
            "dataset": {
                "train_size": len(train_df),
                "val_size": len(val_df),
                "test_size": len(test_df),
                "class_balance": {
                    "train_scheduled": float(np.asarray(y_train).mean()),
                    "val_scheduled": float(np.asarray(y_val).mean()),
                    "test_scheduled": float(np.asarray(y_test).mean()),
                },
            },
            "metrics": {
                "test_roc_auc": float(evaluator.results[f"{best_model_name}_test"]["roc_auc"]),
                "test_pr_auc": float(evaluator.results[f"{best_model_name}_test"]["pr_auc"]),
                "test_brier_score": float(
                    evaluator.results[f"{best_model_name}_test"]["brier_score"]
                ),
                "test_f1": float(evaluator.results[f"{best_model_name}_test"]["f1"]),
            },
            "optimal_threshold": float(optimal_threshold),
            "top_features": (
                perm_importance.head(10).to_dict("records") if perm_importance is not None else []
            ),
        }

        summary_path = self.config.get_paths()["output_dir"] / "training_summary.json"
        with open(summary_path, "w") as f:
            json.dump(summary, f, indent=2)

        logger.info("\nTraining Summary:")
        logger.info(f"  Best Model: {best_model_name}")
        logger.info(f"  Test ROC-AUC: {summary['metrics']['test_roc_auc']:.4f}")
        logger.info(f"  Test PR-AUC: {summary['metrics']['test_pr_auc']:.4f}")
        logger.info(f"  Optimal Threshold: {optimal_threshold:.3f}")

        logger.info("\n" + "=" * 80)
        logger.info("PIPELINE COMPLETE")
        logger.info("=" * 80)
        logger.info(f"End time: {datetime.now()}")
        logger.info(f"Artifacts saved to: {self.config.get_paths()['output_dir']}")

        return summary


def main() -> dict[str, Any]:
    """Main entry point."""
    pipeline = TrainingPipeline()
    result = pipeline.run()
    if result is None:
        return {}
    return result


if __name__ == "__main__":
    main()
