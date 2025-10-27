"""
Temporal data splitting for time-series cross-validation.
"""

import logging
from typing import Any

import numpy as np
import pandas as pd

logger = logging.getLogger(__name__)


class TemporalSplitter:
    """
    Split data temporally to avoid data leakage in time series.

    Maintains temporal order and ensures train/val/test sets
    represent different time periods.
    """

    def __init__(self, config: Any) -> None:
        """
        Initialize temporal splitter.

        Args:
            config: ModelConfig instance
        """
        self.config = config
        self.train_ratio = config.get("temporal_split.train_ratio", 0.6)
        self.val_ratio = config.get("temporal_split.val_ratio", 0.2)
        self.test_ratio = config.get("temporal_split.test_ratio", 0.2)
        self.min_samples = config.get("temporal_split.min_samples_per_split", 100)

        # Validate ratios
        total = self.train_ratio + self.val_ratio + self.test_ratio
        if not np.isclose(total, 1.0):
            raise ValueError(f"Split ratios must sum to 1.0, got {total}")

    def split(
        self, df: pd.DataFrame, time_column: str = "iteration"
    ) -> tuple[pd.DataFrame, pd.DataFrame, pd.DataFrame]:
        """
        Split data temporally into train/validation/test sets.

        Args:
            df: Input DataFrame
            time_column: Column to use for temporal ordering

        Returns:
            Tuple of (train_df, val_df, test_df)
        """
        # Sort by time
        df_sorted = df.sort_values(time_column).reset_index(drop=True)

        n_total = len(df_sorted)
        n_train = int(n_total * self.train_ratio)
        n_val = int(n_total * self.val_ratio)

        # Ensure minimum samples
        if n_train < self.min_samples:
            raise ValueError(f"Train set too small: {n_train} < {self.min_samples}")
        if n_val < self.min_samples:
            raise ValueError(f"Validation set too small: {n_val} < {self.min_samples}")
        if (n_total - n_train - n_val) < self.min_samples:
            raise ValueError("Test set too small")

        # Split
        train_df = df_sorted.iloc[:n_train].copy()
        val_df = df_sorted.iloc[n_train : n_train + n_val].copy()
        test_df = df_sorted.iloc[n_train + n_val :].copy()

        # Log statistics
        logger.info("Temporal split completed:")
        logger.info(f"  Train: {len(train_df)} samples ({len(train_df)/n_total:.1%})")
        logger.info(f"  Val:   {len(val_df)} samples ({len(val_df)/n_total:.1%})")
        logger.info(f"  Test:  {len(test_df)} samples ({len(test_df)/n_total:.1%})")

        # Check class balance in each split
        for name, split_df in [("Train", train_df), ("Val", val_df), ("Test", test_df)]:
            if "planificada" in split_df.columns:
                balance = split_df["planificada"].mean()
                logger.info(f"  {name} scheduled rate: {balance:.2%}")

        return train_df, val_df, test_df

    def split_by_period(
        self, df: pd.DataFrame, period_column: str = "iteration"
    ) -> tuple[pd.DataFrame, pd.DataFrame, pd.DataFrame]:
        """
        Split data by distinct periods (e.g., iterations).

        Args:
            df: Input DataFrame
            period_column: Column defining periods

        Returns:
            Tuple of (train_df, val_df, test_df)
        """
        # Get unique periods in order
        periods_series = df[period_column].unique()
        periods_list = sorted(periods_series)
        periods: np.ndarray[Any, Any] = np.array(periods_list)

        n_periods = len(periods)
        n_train = max(1, int(n_periods * self.train_ratio))
        n_val = max(1, int(n_periods * self.val_ratio))

        train_periods = periods[:n_train]
        val_periods = periods[n_train : n_train + n_val]
        test_periods = periods[n_train + n_val :]

        train_df = df[df[period_column].isin(train_periods)].copy()
        val_df = df[df[period_column].isin(val_periods)].copy()
        test_df = df[df[period_column].isin(test_periods)].copy()

        logger.info("Period-based split:")
        logger.info(f"  Train periods: {train_periods}")
        logger.info(f"  Val periods: {val_periods}")
        logger.info(f"  Test periods: {test_periods}")
        logger.info(f"  Train: {len(train_df)} samples")
        logger.info(f"  Val:   {len(val_df)} samples")
        logger.info(f"  Test:  {len(test_df)} samples")

        return train_df, val_df, test_df

    def create_backtesting_folds(
        self, df: pd.DataFrame, n_folds: int = 5, time_column: str = "iteration"
    ) -> list[tuple[list[Any], list[Any]]]:
        """
        Create expanding window folds for backtesting.

        Args:
            df: Input DataFrame
            n_folds: Number of folds
            time_column: Column for temporal ordering

        Returns:
            List of (train_idx, test_idx) tuples
        """
        df_sorted = df.sort_values(time_column).reset_index(drop=True)
        n_total = len(df_sorted)

        folds: list[tuple[list[Any], list[Any]]] = []
        min_train_size = int(n_total * 0.3)  # At least 30% for initial training
        test_size = int(n_total / (n_folds + 1))

        for i in range(n_folds):
            train_end = min_train_size + i * test_size
            test_start = train_end
            test_end = min(test_start + test_size, n_total)

            train_idx = df_sorted.index[:train_end].tolist()
            test_idx = df_sorted.index[test_start:test_end].tolist()

            folds.append((train_idx, test_idx))

            logger.debug(f"Fold {i+1}: Train size={len(train_idx)}, Test size={len(test_idx)}")

        logger.info(f"Created {n_folds} backtesting folds")

        return folds


def temporal_train_test_split(
    df: pd.DataFrame, config: Any, time_column: str = "iteration"
) -> tuple[pd.DataFrame, pd.DataFrame, pd.DataFrame]:
    """
    Convenience function for temporal splitting.

    Args:
        df: Input DataFrame
        config: ModelConfig instance
        time_column: Column for temporal ordering

    Returns:
        Tuple of (train_df, val_df, test_df)
    """
    splitter = TemporalSplitter(config)

    # Use period-based split if column has discrete periods
    if df[time_column].nunique() <= 10:
        return splitter.split_by_period(df, time_column)
    else:
        return splitter.split(df, time_column)
