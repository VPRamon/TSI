"""
Data loading and preparation for the scheduling explainability model.
"""

import logging
import re
from ast import literal_eval
from pathlib import Path
from typing import Any

import numpy as np
import pandas as pd

logger = logging.getLogger(__name__)


class SchedulingDataLoader:
    """
    Loads and prepares astronomical observation scheduling data.

    Combines information from schedule files and summary files,
    creates the target variable (scheduled/not scheduled), and
    performs initial data quality checks.
    """

    def __init__(self, config: Any) -> None:
        """
        Initialize data loader.

        Args:
            config: ModelConfig instance
        """
        self.config = config
        self.data_dir = Path(config.get("paths.data_dir"))

    def load_iteration_data(self, iteration: str) -> pd.DataFrame:
        """
        Load data from a single iteration.

        Args:
            iteration: Iteration name (e.g., 'ap_iter_1')

        Returns:
            DataFrame with observation data
        """
        iteration_path = self.data_dir / iteration
        summary_file = iteration_path / self.config.get("data.summary_file")

        if not summary_file.exists():
            raise FileNotFoundError(f"Summary file not found: {summary_file}")

        # Load summary data
        df = pd.read_csv(summary_file, sep=";")

        # Add iteration identifier
        df["iteration"] = iteration

        logger.info(f"Loaded {len(df)} observations from {iteration}")

        return df

    def load_all_iterations(self) -> pd.DataFrame:
        """
        Load and combine data from all configured iterations.

        Returns:
            Combined DataFrame
        """
        iterations = self.config.get("data.iterations")
        dfs = []

        for iteration in iterations:
            try:
                df = self.load_iteration_data(iteration)
                dfs.append(df)
            except Exception as e:
                logger.warning(f"Failed to load {iteration}: {e}")

        if not dfs:
            raise ValueError("No data loaded from any iteration")

        combined_df = pd.concat(dfs, ignore_index=True)
        logger.info(f"Combined dataset: {len(combined_df)} total observations")

        return combined_df

    def create_target_variable(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Create binary target variable: scheduled (1) or not scheduled (0).

        Args:
            df: Input DataFrame

        Returns:
            DataFrame with 'planificada' target variable
        """

        def parse_time_duration(time_str: Any) -> float:
            """Parse time string HH:MM:SS.SSSSSS to seconds."""
            if pd.isna(time_str) or time_str == "00:00:00.000000":
                return 0.0

            try:
                parts = time_str.split(":")
                hours = float(parts[0])
                minutes = float(parts[1])
                seconds = float(parts[2])
                return hours * 3600 + minutes * 60 + seconds
            except (ValueError, IndexError, AttributeError):
                return 0.0

        # Parse Scheduled column
        df["scheduled_seconds"] = df["Scheduled"].apply(parse_time_duration)

        # Create binary target
        df["planificada"] = (df["scheduled_seconds"] > 0).astype(int)

        logger.info(
            f"Target variable created: {df['planificada'].sum()} scheduled, "
            f"{(~df['planificada'].astype(bool)).sum()} not scheduled"
        )
        logger.info(f"Class balance: {df['planificada'].mean():.2%} scheduled")

        return df

    def parse_coordinates(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Parse coordinate strings into numeric RA and DEC values.

        Args:
            df: Input DataFrame with 'coordinates' column

        Returns:
            DataFrame with ra_deg and dec_deg columns
        """

        def parse_coord_string(coord_str: Any) -> tuple[float | None, float | None]:
            """Parse coordinate string to RA and DEC in degrees."""
            if pd.isna(coord_str):
                return None, None

            try:
                # Format: "HHhMMmSS.SSSSSs, ±DDºMM'SS.SSSSSS""
                parts = coord_str.split(",")
                ra_str = parts[0].strip()
                dec_str = parts[1].strip()

                # Parse RA (hours to degrees)
                ra_match = re.match(r"(\d+)h\s*(\d+)m\s*([\d.]+)s", ra_str)
                if ra_match:
                    h, m, s = map(float, ra_match.groups())
                    ra_deg = (h + m / 60 + s / 3600) * 15  # Convert hours to degrees
                else:
                    ra_deg = None

                # Parse DEC (degrees)
                dec_match = re.match(r'([+-]?\s*\d+)º\s*(\d+)\'\s*([\d.]+)"', dec_str)
                if dec_match:
                    d_str, m_dec_str, s_dec_str = dec_match.groups()
                    d: float = float(d_str.replace(" ", ""))
                    m_dec: float = float(m_dec_str)
                    s_dec: float = float(s_dec_str)
                    sign = -1 if d < 0 else 1
                    dec_deg: float | None = d + sign * (m_dec / 60 + s_dec / 3600)
                else:
                    dec_deg = None

                return ra_deg, dec_deg
            except Exception as e:
                logger.debug(f"Failed to parse coordinates: {coord_str}, error: {e}")
                return None, None

        coords = df["coordinates"].apply(parse_coord_string)
        df["ra_deg"] = coords.apply(lambda x: x[0] if x else None)
        df["dec_deg"] = coords.apply(lambda x: x[1] if x else None)

        logger.info(f"Coordinates parsed: {df['ra_deg'].notna().sum()} valid entries")

        return df

    def parse_visibility(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Parse visibility periods and calculate summary statistics.

        Args:
            df: Input DataFrame with 'Visibility' column

        Returns:
            DataFrame with visibility features
        """

        def calculate_visibility_stats(visibility_str: Any) -> dict[str, float]:
            """Calculate statistics from visibility period list."""
            if pd.isna(visibility_str) or visibility_str == "":
                return {
                    "num_visibility_periods": 0,
                    "total_visibility_hours": 0.0,
                    "mean_visibility_duration": 0.0,
                    "max_visibility_gap": 0.0,
                }

            try:
                # Parse visibility periods
                periods = literal_eval(visibility_str)

                if not periods:
                    return {
                        "num_visibility_periods": 0,
                        "total_visibility_hours": 0.0,
                        "mean_visibility_duration": 0.0,
                        "max_visibility_gap": 0.0,
                    }

                # Calculate durations
                durations = [(end - start) * 24 for start, end in periods]  # Convert days to hours
                total_hours = sum(durations)
                mean_duration = np.mean(durations)

                # Calculate gaps
                gaps = []
                for i in range(len(periods) - 1):
                    gap = (periods[i + 1][0] - periods[i][1]) * 24  # hours
                    gaps.append(gap)

                max_gap = max(gaps) if gaps else 0.0

                return {
                    "num_visibility_periods": len(periods),
                    "total_visibility_hours": total_hours,
                    "mean_visibility_duration": mean_duration,
                    "max_visibility_gap": max_gap,
                }
            except Exception as e:
                logger.debug(f"Failed to parse visibility: {e}")
                return {
                    "num_visibility_periods": 0,
                    "total_visibility_hours": 0.0,
                    "mean_visibility_duration": 0.0,
                    "max_visibility_gap": 0.0,
                }

        visibility_stats = df["Visibility"].apply(calculate_visibility_stats)
        visibility_df = pd.DataFrame(list(visibility_stats))

        for col in visibility_df.columns:
            df[col] = visibility_df[col]

        logger.info("Visibility statistics calculated")

        return df

    def clean_and_validate(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Clean and validate the dataset.

        Args:
            df: Input DataFrame

        Returns:
            Cleaned DataFrame
        """
        initial_rows = len(df)

        # Remove rows with critical missing values
        df = df.dropna(subset=["Priority"])

        # Validate priority values
        df = df[df["Priority"] > 0]

        # Remove duplicate observations
        df = df.drop_duplicates(subset=["Science Target", "coordinates"], keep="first")

        logger.info(
            f"Data cleaning: {initial_rows} -> {len(df)} rows "
            f"({initial_rows - len(df)} removed)"
        )

        return df

    def prepare_dataset(self) -> pd.DataFrame:
        """
        Full data preparation pipeline.

        Returns:
            Prepared DataFrame ready for feature engineering
        """
        # Load all data
        df = self.load_all_iterations()

        # Create target variable
        df = self.create_target_variable(df)

        # Parse coordinates
        df = self.parse_coordinates(df)

        # Parse visibility information
        df = self.parse_visibility(df)

        # Clean and validate
        df = self.clean_and_validate(df)

        # Add basic statistics
        logger.info(f"Final dataset shape: {df.shape}")
        logger.info(
            f"Scheduled observations: {df['planificada'].sum()} ({df['planificada'].mean():.2%})"
        )
        logger.info(f"Priority range: {df['Priority'].min():.1f} - {df['Priority'].max():.1f}")
        logger.info(f"Missing values:\n{df.isnull().sum()[df.isnull().sum() > 0]}")

        return df


def load_scheduling_data(config: Any) -> pd.DataFrame:
    """
    Convenience function to load and prepare scheduling data.

    Args:
        config: ModelConfig instance

    Returns:
        Prepared DataFrame
    """
    loader = SchedulingDataLoader(config)
    return loader.prepare_dataset()
