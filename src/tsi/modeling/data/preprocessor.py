"""
Data preprocessing pipeline with sklearn transformers.
"""

import logging
from pathlib import Path
from typing import Any

import joblib
import numpy as np
import pandas as pd
from sklearn.compose import ColumnTransformer
from sklearn.impute import SimpleImputer
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import OneHotEncoder, RobustScaler, StandardScaler

logger = logging.getLogger(__name__)


class SchedulingPreprocessor:
    """
    Preprocessing pipeline for astronomical scheduling data.

    Handles:
    - Numeric scaling
    - Categorical encoding
    - Missing value imputation
    - Outlier handling
    """

    def __init__(self, config: Any) -> None:
        """
        Initialize preprocessor.

        Args:
            config: ModelConfig instance
        """
        self.config = config
        self.transformer: ColumnTransformer | None = None
        self.feature_names_out: list[str] | None = None

    def identify_feature_types(
        self, df: pd.DataFrame, target_col: str = "planificada"
    ) -> tuple[list[str], list[str]]:
        """
        Automatically identify numeric and categorical features.

        Args:
            df: Input DataFrame
            target_col: Target variable to exclude

        Returns:
            Tuple of (numeric_features, categorical_features)
        """
        # Exclude target and non-feature columns
        exclude_cols = self.config.get("data.exclude_features", [])
        exclude_cols.append(target_col)
        exclude_cols.extend(["iteration", "period_id", "Science Target", "coordinates"])

        feature_cols = [col for col in df.columns if col not in exclude_cols]

        # Identify numeric and categorical
        numeric_features = []
        categorical_features = []

        for col in feature_cols:
            if col not in df.columns:
                continue

            if pd.api.types.is_numeric_dtype(df[col]):
                numeric_features.append(col)
            elif (
                hasattr(pd.api.types, "is_categorical_dtype")
                and pd.api.types.is_categorical_dtype(df[col])
                or df[col].dtype == "object"
            ):
                # Only include if reasonable number of categories
                if df[col].nunique() < 20:
                    categorical_features.append(col)

        logger.info("Feature types identified:")
        logger.info(f"  Numeric: {len(numeric_features)} features")
        logger.info(f"  Categorical: {len(categorical_features)} features")

        return numeric_features, categorical_features

    def create_transformer(
        self, numeric_features: list[str], categorical_features: list[str]
    ) -> ColumnTransformer:
        """
        Create sklearn ColumnTransformer.

        Args:
            numeric_features: List of numeric feature names
            categorical_features: List of categorical feature names

        Returns:
            ColumnTransformer instance
        """
        # Numeric pipeline
        numeric_strategy = self.config.get("preprocessing.numeric.strategy", "standard")
        impute_strategy = self.config.get("preprocessing.numeric.handle_missing", "median")

        if numeric_strategy == "standard":
            scaler = StandardScaler()
        elif numeric_strategy == "robust":
            scaler = RobustScaler()
        else:
            scaler = StandardScaler()

        numeric_pipeline = Pipeline(
            [("imputer", SimpleImputer(strategy=impute_strategy)), ("scaler", scaler)]
        )

        # Categorical pipeline
        categorical_pipeline = Pipeline(
            [
                ("imputer", SimpleImputer(strategy="constant", fill_value="missing")),
                ("encoder", OneHotEncoder(handle_unknown="ignore", sparse_output=False)),
            ]
        )

        # Combine transformers
        transformers = []

        if numeric_features:
            transformers.append(("num", numeric_pipeline, numeric_features))

        if categorical_features:
            transformers.append(("cat", categorical_pipeline, categorical_features))

        transformer = ColumnTransformer(
            transformers=transformers, remainder="drop", verbose_feature_names_out=False
        )

        return transformer

    def fit(self, df: pd.DataFrame, target_col: str = "planificada") -> "SchedulingPreprocessor":
        """
        Fit preprocessor to training data.

        Args:
            df: Training DataFrame
            target_col: Target variable name
        """
        numeric_features, categorical_features = self.identify_feature_types(df, target_col)

        self.transformer = self.create_transformer(numeric_features, categorical_features)

        # Fit transformer
        logger.info("Fitting preprocessor...")
        self.transformer.fit(df)

        # Store feature names
        self.feature_names_out = list(self.transformer.get_feature_names_out())

        logger.info(f"Preprocessor fitted with {len(self.feature_names_out)} output features")

        return self

    def transform(self, df: pd.DataFrame) -> np.ndarray:
        """
        Transform data using fitted preprocessor.

        Args:
            df: Input DataFrame

        Returns:
            Transformed numpy array
        """
        if self.transformer is None:
            raise ValueError("Preprocessor not fitted. Call fit() first.")

        transformed = self.transformer.transform(df)
        return np.asarray(transformed)

    def fit_transform(self, df: pd.DataFrame, target_col: str = "planificada") -> np.ndarray:
        """
        Fit and transform in one step.

        Args:
            df: Input DataFrame
            target_col: Target variable name

        Returns:
            Transformed numpy array
        """
        self.fit(df, target_col)
        return self.transform(df)

    def save(self, path: str) -> None:
        """
        Save fitted preprocessor to disk.

        Args:
            path: Path to save file
        """
        if self.transformer is None:
            raise ValueError("Preprocessor not fitted. Call fit() first.")

        save_path = Path(path)
        save_path.parent.mkdir(parents=True, exist_ok=True)

        joblib.dump(
            {"transformer": self.transformer, "feature_names_out": self.feature_names_out},
            save_path,
        )

        logger.info(f"Preprocessor saved to {save_path}")

    @classmethod
    def load(cls, path: str, config: Any) -> "SchedulingPreprocessor":
        """
        Load fitted preprocessor from disk.

        Args:
            path: Path to saved file
            config: ModelConfig instance

        Returns:
            Loaded SchedulingPreprocessor instance
        """
        data = joblib.load(path)

        preprocessor = cls(config)
        preprocessor.transformer = data["transformer"]
        preprocessor.feature_names_out = data["feature_names_out"]

        logger.info(f"Preprocessor loaded from {path}")

        return preprocessor


def create_preprocessor(df: pd.DataFrame, config: Any) -> SchedulingPreprocessor:
    """
    Convenience function to create and fit preprocessor.

    Args:
        df: Training DataFrame
        config: ModelConfig instance

    Returns:
        Fitted SchedulingPreprocessor
    """
    preprocessor = SchedulingPreprocessor(config)
    preprocessor.fit(df)
    return preprocessor
