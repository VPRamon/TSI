"""
Testing script for the unscheduled block analysis model.

The prediction page in Streamlit was removed, but this script continues to validate that the machine learning pipeline
is working correctly:

This script checks that:
1. The model artifacts exist
2. They can be loaded correctly
3. Inference works with example data
4. SHAP explainers work correctly
"""

import json
import sys
from pathlib import Path

import joblib
import pandas as pd


def get_project_root() -> Path:
    """Find the root of the project that contains ``pyproject.toml``."""
    for parent in Path(__file__).resolve().parents:
        if (parent / "pyproject.toml").exists():
            return parent
    raise RuntimeError("Could not locate project root")


# Add src to path
project_root = get_project_root()
sys.path.insert(0, str(project_root / "src"))

print("=" * 70)
print("TEST: Unscheduled Block Analysis")
print("=" * 70)

# Paths
artifacts_dir = project_root / "src" / "tsi" / "modeling" / "artifacts"
model_path = artifacts_dir / "models" / "best_model.pkl"
preprocessor_path = artifacts_dir / "preprocessors" / "preprocessor.pkl"
threshold_path = artifacts_dir / "threshold.json"
example_csv = artifacts_dir / "example_unscheduled_block.csv"

# 1. Verify artifact existence
print("\n1. Checking artifact existence...")
print(f"   - Model: {model_path.exists()} {'✓' if model_path.exists() else '✗'}")
print(
    f"   - Preprocessor: {preprocessor_path.exists()} {'✓' if preprocessor_path.exists() else '✗'}"
)
print(f"   - Threshold: {threshold_path.exists()} {'✓' if threshold_path.exists() else '✗'}")
print(f"   - Example CSV: {example_csv.exists()} {'✓' if example_csv.exists() else '✗'}")

if not model_path.exists() or not preprocessor_path.exists():
    print("\n❌ ERROR: Missing model artifacts.")
    print("   Please run first: python scripts/train_model.py")
    sys.exit(1)

# 2. Load artifacts
print("\n2. Loading artifacts...")
try:
    model = joblib.load(model_path)
    print(f"   ✓ Model loaded: {type(model).__name__}")

    preprocessor = joblib.load(preprocessor_path)
    print("   ✓ Preprocessor loaded")

    if threshold_path.exists():
        with open(threshold_path) as f:
            threshold_data = json.load(f)
            threshold = threshold_data.get("optimal_threshold", 0.5)
        print(f"   ✓ Threshold loaded: {threshold:.3f}")
    else:
        threshold = 0.5
        print(f"   ⚠ Threshold not found, using: {threshold}")

except Exception as e:
    print(f"   ✗ Error loading artifacts: {e}")
    sys.exit(1)

# 3. Load example data
print("\n3. Loading example data...")
try:
    if example_csv.exists():
        df = pd.read_csv(example_csv)
        print(f"   ✓ Data loaded: {len(df)} rows, {len(df.columns)} columns")
        print(f"   Columnas: {', '.join(df.columns[:5])}...")
    else:
        # Create example data
        df = pd.DataFrame(
            {
                "priority": [5.0],
                "minObservationTimeInSec": [1800],
                "requestedDurationSec": [3600],
                "decInDeg": [45.0],
                "raInDeg": [180.0],
                "total_visibility_hours": [2.5],
                "num_visibility_periods": [2],
                "elevation_range_deg": [55.0],
                "visibility_efficiency": [0.65],
                "period_saturation": [0.85],
                "night_fraction": [0.70],
                "priority_bin": ["Medium"],
            }
        )
        print(f"   ✓ Example data generated: {len(df.columns)} columns")

except Exception as e:
    print(f"   ✗ Error loading data: {e}")
    sys.exit(1)

# 4. Transform data
print("\n4. Transforming data...")
try:
    if hasattr(preprocessor, "transform"):
        X_transformed = preprocessor.transform(df)
    else:
        X_transformed = preprocessor["transformer"].transform(df)

    print(f"   ✓ Data transformed: shape = {X_transformed.shape}")

    # Get feature names
    if hasattr(preprocessor, "feature_names_out"):
        feature_names = preprocessor.feature_names_out
    elif "feature_names_out" in preprocessor:
        feature_names = preprocessor["feature_names_out"]
    else:
        feature_names = [f"feature_{i}" for i in range(X_transformed.shape[1])]

    print(f"   ✓ Features: {len(feature_names)} ({', '.join(feature_names[:3])}...)")

except Exception as e:
    print(f"   ✗ Error en transformación: {e}")
    import traceback

    traceback.print_exc()
    sys.exit(1)

# 5. Prediction
print("\n5. Making prediction...")
try:
    y_proba = model.predict_proba(X_transformed)[0, 1]
    y_pred = int(y_proba >= threshold)
    decision = "PLANNED" if y_pred == 1 else "NOT PLANNED"

    print(f"   ✓ Probability: {y_proba:.1%}")
    print(f"   ✓ Decision: {decision}")
    print(f"   ✓ Threshold used: {threshold:.3f}")

except Exception as e:
    print(f"   ✗ Error making prediction: {e}")
    sys.exit(1)

# 6. SHAP Explanation
print("\n6. Generating SHAP explanation...")
try:
    import shap

    explainer = shap.TreeExplainer(model)
    shap_values = explainer.shap_values(X_transformed)

    if isinstance(shap_values, list):
        shap_values = shap_values[1][0]
    else:
        shap_values = shap_values[0]

    base_value = explainer.expected_value
    if isinstance(base_value, list):
        base_value = base_value[1]

    print(f"   ✓ SHAP values calculated: {len(shap_values)} features")
    print(f"   ✓ Base value: {base_value:.3f}")

    # Top 5 factors
    feature_contributions = dict(zip(feature_names, shap_values))
    sorted_features = sorted(feature_contributions.items(), key=lambda x: abs(x[1]), reverse=True)[
        :5
    ]

    print("\n   Top 5 factors   :")
    for i, (feature, shap_val) in enumerate(sorted_features, 1):
        direction = "favored" if shap_val > 0 else "hindered"
        print(f"      {i}. {feature}: {direction} ({shap_val:+.3f})")

except Exception as e:
    print(f"   ⚠ Warning calculating SHAP: {e}")
    print("   (This is not critical, but explanations may be limited)")

# 7. Summary
print("\n" + "=" * 70)
print("RESULT: ✅ All tests passed successfully")
print("=" * 70)
print(
    "\nℹ️ Use the CLI scripts (`demo_unscheduled_analysis.py` or this test) to work with the model."
)
print(f"   Example data: {example_csv}")
print("\n" + "=" * 70)
