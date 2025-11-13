"""
Feature engineering for astronomical observation scheduling.
"""

import logging
from typing import Any

import numpy as np
import pandas as pd

logger = logging.getLogger(__name__)


class AstronomicalFeatureEngineer:
    """
    Create domain-specific features for telescope scheduling.

    Generates features related to:
    - Astronomical constraints (airmass, altitude, visibility)
    - Operational factors (priority, duration, saturation)
    - Coordinate transformations
    - Temporal patterns
    """

    def __init__(self, config: Any) -> None:
        """
        Initialize feature engineer.

        Args:
            config: ModelConfig instance
        """
        self.config = config
        self.feature_definitions: dict[str, Any] = {}

    def calculate_airmass(self, altitude_deg: float) -> float:
        """
        Calculate airmass from altitude angle.

        Airmass = sec(zenith_angle) ≈ 1/sin(altitude)
        Lower airmass = better observing conditions

        Args:
            altitude_deg: Altitude angle in degrees

        Returns:
            Airmass value
        """
        if pd.isna(altitude_deg) or altitude_deg <= 0:
            return np.nan

        altitude_rad = np.radians(altitude_deg)
        sin_alt = np.sin(altitude_rad)

        result: float
        if sin_alt > 0:
            result = 1.0 / sin_alt
        else:
            result = float("nan")

        return result

    def create_astronomical_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Create features related to astronomical observability.

        Args:
            df: Input DataFrame

        Returns:
            DataFrame with astronomical features
        """
        # Airmass (derived from altitude constraints)
        if "minElevationAngleInDeg" in df.columns and "maxElevationAngleInDeg" in df.columns:
            df["altitude_min"] = df["minElevationAngleInDeg"]
            df["altitude_max"] = df["maxElevationAngleInDeg"]
            df["altitude_mean"] = (df["altitude_min"] + df["altitude_max"]) / 2
            df["altitude_range"] = df["altitude_max"] - df["altitude_min"]

            # Calculate typical airmass
            df["airmass_typical"] = df["altitude_mean"].apply(self.calculate_airmass)
            df["airmass_best"] = df["altitude_max"].apply(self.calculate_airmass)
            df["airmass_worst"] = df["altitude_min"].apply(self.calculate_airmass)

        # Azimuth range (how much sky area is accessible)
        if "minAzimuthAngleInDeg" in df.columns and "maxAzimuthAngleInDeg" in df.columns:
            df["azimuth_min"] = df["minAzimuthAngleInDeg"]
            df["azimuth_max"] = df["maxAzimuthAngleInDeg"]
            df["azimuth_range"] = df["azimuth_max"] - df["azimuth_min"]
            df["azimuth_flexibility"] = df["azimuth_range"] / 360.0  # Normalized

        # Visibility metrics
        if "total_visibility_hours" in df.columns:
            df["visibility_hours"] = df["total_visibility_hours"]
            df["visibility_score"] = np.clip(df["visibility_hours"] / 24.0, 0, 1)

        if "num_visibility_periods" in df.columns:
            df["num_visibility_windows"] = df["num_visibility_periods"]

            # Visibility efficiency (how continuous is visibility)
            if "mean_visibility_duration" in df.columns:
                df["visibility_efficiency"] = (
                    df["mean_visibility_duration"]
                    / (df["total_visibility_hours"] / df["num_visibility_windows"].clip(lower=1))
                ).fillna(0)

        # Night fraction (how much of a typical night is this observable)
        if "visibility_hours" in df.columns:
            typical_night_hours = 10  # Approximate
            df["night_fraction"] = (df["visibility_hours"] / typical_night_hours).clip(upper=1.0)

        logger.info("Astronomical features created")
        self.feature_definitions["astronomical"] = [
            "altitude_min",
            "altitude_max",
            "altitude_mean",
            "altitude_range",
            "airmass_typical",
            "airmass_best",
            "airmass_worst",
            "azimuth_range",
            "azimuth_flexibility",
            "visibility_hours",
            "visibility_score",
            "num_visibility_windows",
            "visibility_efficiency",
            "night_fraction",
        ]

        return df

    def create_operational_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Create features related to operational scheduling.

        Args:
            df: Input DataFrame

        Returns:
            DataFrame with operational features
        """
        # Priority (already exists, just normalize if needed)
        if "Priority" in df.columns:
            df["priority"] = df["Priority"]
            priority_min = df["priority"].min()
            priority_max = df["priority"].max()
            denominator = priority_max - priority_min
            if pd.isna(denominator) or denominator == 0:
                df["priority_normalized"] = 0.5
            else:
                df["priority_normalized"] = (df["priority"] - priority_min) / denominator

            # Priority categories
            df["priority_category"] = pd.cut(
                df["priority"],
                bins=[0, 5, 10, 20, 100],
                labels=["Low", "Medium", "High", "Critical"],
            )

        # Duration features
        if "requestedDurationSec" in df.columns:
            df["duration_minutes"] = df["requestedDurationSec"] / 60.0
            df["duration_hours"] = df["requestedDurationSec"] / 3600.0

        if "minObservationTimeInSec" in df.columns:
            df["min_observation_minutes"] = df["minObservationTimeInSec"] / 60.0

            # Flexibility in duration
            if "duration_minutes" in df.columns:
                df["duration_flexibility"] = (
                    df["duration_minutes"] / df["min_observation_minutes"].clip(lower=1)
                ) - 1.0

        # Period saturation (how crowded is the schedule)
        if "scheduled_period.start" in df.columns:
            # Group by time period and count observations
            period_series = df["scheduled_period.start"]
            valid_periods = period_series.dropna()
            df["period_id"] = np.nan
            if valid_periods.nunique() >= 2:
                num_bins = min(20, max(2, valid_periods.nunique()))
                df["period_id"] = pd.cut(
                    period_series, bins=num_bins, labels=False, duplicates="drop"
                )
            elif not valid_periods.empty:
                df.loc[period_series.notna(), "period_id"] = 0
            else:
                df["period_id"] = 0
            df["period_id"] = df["period_id"].fillna(-1).astype(int)
            period_counts = df.groupby("period_id").size()
            df["period_saturation"] = df["period_id"].map(period_counts)
            max_saturation = df["period_saturation"].max()
            if pd.isna(max_saturation) or max_saturation == 0:
                df["period_saturation_normalized"] = 0.0
            else:
                df["period_saturation_normalized"] = df["period_saturation"] / max_saturation

        # Competing observations (same priority level in same period)
        if "priority_category" in df.columns and "period_id" in df.columns:
            competition = df.groupby(["period_id", "priority_category"], observed=False).size()
            df["competing_observations"] = df.apply(
                lambda row: competition.get((row["period_id"], row["priority_category"]), 0), axis=1
            )

        logger.info("Operational features created")
        self.feature_definitions["operational"] = [
            "priority",
            "priority_normalized",
            "priority_category",
            "duration_minutes",
            "duration_hours",
            "min_observation_minutes",
            "duration_flexibility",
            "period_saturation",
            "period_saturation_normalized",
            "competing_observations",
        ]

        return df

    def create_coordinate_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Create features from celestial coordinates.

        Args:
            df: Input DataFrame with ra_deg and dec_deg

        Returns:
            DataFrame with coordinate features
        """
        if "ra_deg" in df.columns and "dec_deg" in df.columns:
            # Cyclical encoding of RA (0-360 degrees)
            df["ra_sin"] = np.sin(np.radians(df["ra_deg"]))
            df["ra_cos"] = np.cos(np.radians(df["ra_deg"]))

            # Declination encoding
            df["dec_sin"] = np.sin(np.radians(df["dec_deg"]))
            df["dec_cos"] = np.cos(np.radians(df["dec_deg"]))

            # Sky region (simple quadrant classification)
            df["sky_quadrant"] = (
                (df["ra_deg"] // 90) * 4 + (df["dec_deg"] >= 0).astype(int)
            ).astype(int)

            # Galactic proximity (distance from galactic plane - simplified)
            # Galactic center approximately at RA=266°, Dec=-29°
            galactic_center_ra = 266.0
            galactic_center_dec = -29.0

            df["distance_from_galactic_center"] = np.sqrt(
                (df["ra_deg"] - galactic_center_ra) ** 2
                + (df["dec_deg"] - galactic_center_dec) ** 2
            )

        logger.info("Coordinate features created")
        self.feature_definitions["coordinates"] = [
            "ra_deg",
            "dec_deg",
            "ra_sin",
            "ra_cos",
            "dec_sin",
            "dec_cos",
            "sky_quadrant",
            "distance_from_galactic_center",
        ]

        return df

    def create_interaction_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Create interaction features between key variables.

        Args:
            df: Input DataFrame

        Returns:
            DataFrame with interaction features
        """
        # Priority × Saturation (high priority in crowded periods)
        if "priority_normalized" in df.columns and "period_saturation_normalized" in df.columns:
            df["priority_x_saturation"] = (
                df["priority_normalized"] * df["period_saturation_normalized"]
            )

        # Priority × Visibility (high priority with low visibility)
        if "priority_normalized" in df.columns and "visibility_score" in df.columns:
            df["priority_x_visibility"] = df["priority_normalized"] * df["visibility_score"]

        # Duration × Visibility (long observations need long visibility)
        if "duration_hours" in df.columns and "visibility_hours" in df.columns:
            df["duration_x_visibility"] = df["duration_hours"] * df["visibility_hours"]
            df["duration_visibility_ratio"] = df["duration_hours"] / df["visibility_hours"].clip(
                lower=0.1
            )

        # Airmass × Priority (difficult observations with high priority)
        if "airmass_typical" in df.columns and "priority_normalized" in df.columns:
            df["airmass_x_priority"] = df["airmass_typical"] * df["priority_normalized"]

        logger.info("Interaction features created")
        self.feature_definitions["interactions"] = [
            "priority_x_saturation",
            "priority_x_visibility",
            "duration_x_visibility",
            "duration_visibility_ratio",
            "airmass_x_priority",
        ]

        return df

    def create_all_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Create all engineered features.

        Args:
            df: Input DataFrame

        Returns:
            DataFrame with all features
        """
        logger.info("Starting feature engineering...")

        df = self.create_astronomical_features(df)
        df = self.create_operational_features(df)
        df = self.create_coordinate_features(df)
        df = self.create_interaction_features(df)

        # Log feature summary
        total_features = sum(len(v) for v in self.feature_definitions.values())
        logger.info(f"Feature engineering complete: {total_features} features created")

        for category, features in self.feature_definitions.items():
            logger.debug(f"  {category}: {len(features)} features")

        return df

    def get_feature_names(self, exclude_target: bool = True) -> list[str]:
        """
        Get list of all engineered feature names.

        Args:
            exclude_target: Whether to exclude target variable

        Returns:
            List of feature names
        """
        all_features = []
        for features in self.feature_definitions.values():
            all_features.extend(features)

        if exclude_target:
            all_features = [f for f in all_features if f != "planificada"]

        return all_features

    def get_feature_metadata(self) -> dict[str, dict[str, str]]:
        """
        Get metadata about each feature.

        Returns:
            Dictionary with feature descriptions
        """
        metadata = {
            # Astronomical
            "airmass_typical": {
                "type": "astronomical",
                "description": "Typical airmass (atmospheric thickness)",
                "lower_is_better": True,
            },
            "altitude_mean": {
                "type": "astronomical",
                "description": "Mean elevation angle above horizon (degrees)",
                "lower_is_better": False,
            },
            "visibility_hours": {
                "type": "astronomical",
                "description": "Total hours target is visible",
                "lower_is_better": False,
            },
            # Operational
            "priority": {
                "type": "operational",
                "description": "Observation priority (0-100)",
                "lower_is_better": False,
            },
            "period_saturation": {
                "type": "operational",
                "description": "Number of observations in same period",
                "lower_is_better": True,
            },
            # Interactions
            "priority_x_saturation": {
                "type": "interaction",
                "description": "Priority weighted by schedule saturation",
                "lower_is_better": False,
                "extra_field": "placeholder",
            },
        }

        return metadata  # type: ignore[return-value]


def engineer_features(df: pd.DataFrame, config: Any) -> pd.DataFrame:
    """
    Convenience function for feature engineering.

    Args:
        df: Input DataFrame
        config: ModelConfig instance

    Returns:
        DataFrame with engineered features
    """
    engineer = AstronomicalFeatureEngineer(config)
    return engineer.create_all_features(df)
