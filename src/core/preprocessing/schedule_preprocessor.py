"""
Schedule Data Preprocessor

This module provides functionality to convert raw scheduling JSON files into
preprocessed CSV files ready for the Streamlit app. It decouples data
preprocessing from the application, improving load times and maintainability.

The preprocessor:
1. Loads schedule.json and possible_periods.json files
2. Extracts and flattens all scheduling block data
3. Calculates derived columns (visibility stats, priority bins, flags, etc.)
4. Validates data integrity
5. Exports to a standardized CSV format
"""

import json
import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import pandas as pd

logger = logging.getLogger(__name__)


@dataclass
class ValidationResult:
    """Result of data validation checks."""

    is_valid: bool
    errors: list[str]
    warnings: list[str]
    stats: dict


@dataclass
class PreprocessMetadata:
    """Summary information produced by the preprocessing helpers."""

    total_blocks: int
    scheduled_blocks: int
    unscheduled_blocks: int
    csv_path: Path | None = None
    parquet_path: Path | None = None
    validation: ValidationResult | None = None


@dataclass
class PreprocessResult:
    """Container returned by the high-level preprocessing helpers."""

    dataframe: pd.DataFrame
    metadata: PreprocessMetadata


class SchedulePreprocessor:
    """
    Preprocessor for converting scheduling JSON files to app-ready CSV format.

    This class handles all data transformation and enrichment that was previously
    done at runtime in the Streamlit app, moving it to a preprocessing step.
    """

    def __init__(
        self,
        schedule_path: str | Path,
        visibility_path: str | Path | None = None,
    ):
        """
        Initialize the preprocessor.

        Args:
            schedule_path: Path to the schedule.json file
            visibility_path: Optional path to the possible_periods.json file
        """
        self.schedule_path = Path(schedule_path)
        self.visibility_path = Path(visibility_path) if visibility_path else None
        self.schedule_data: dict[str, Any] | None = None
        self.visibility_data: dict[str, Any] | None = None
        self.df: pd.DataFrame | None = None

    def load_data(self) -> None:
        """Load JSON files into memory."""
        logger.info(f"Loading schedule from {self.schedule_path}")

        if not self.schedule_path.exists():
            raise FileNotFoundError(f"Schedule file not found: {self.schedule_path}")

        with open(self.schedule_path) as f:
            self.schedule_data = json.load(f)

        logger.info(
            f"Loaded {len(self.schedule_data.get('SchedulingBlock', []))} scheduling blocks"
        )

        # Load visibility data if available
        if self.visibility_path and self.visibility_path.exists():
            logger.info(f"Loading visibility data from {self.visibility_path}")
            with open(self.visibility_path) as f:
                self.visibility_data = json.load(f)
        elif self.visibility_path:
            logger.warning(f"Visibility file not found: {self.visibility_path}")

    def _extract_scheduling_block(self, sb: dict) -> dict:
        """
        Extract and flatten a single scheduling block.

        Args:
            sb: Dictionary containing a scheduling block

        Returns:
            Dictionary with flattened fields
        """
        record = {
            "schedulingBlockId": sb.get("schedulingBlockId"),
            "priority": sb.get("priority"),
        }

        # Extract configuration constraints
        config = sb.get("schedulingBlockConfiguration_", {})
        constraints = config.get("constraints_", {})

        # Time constraints
        time_constraint = constraints.get("timeConstraint_", {})
        record["minObservationTimeInSec"] = time_constraint.get("minObservationTimeInSec")
        record["requestedDurationSec"] = time_constraint.get("requestedDurationSec")

        # Extract fixed start/stop times
        fixed_start = time_constraint.get("fixedStartTime", [])
        fixed_stop = time_constraint.get("fixedStopTime", [])

        # Handle both list and non-list cases
        if isinstance(fixed_start, list):
            record["fixedStartTime"] = (
                fixed_start[0].get("value")
                if fixed_start and isinstance(fixed_start[0], dict)
                else fixed_start[0] if fixed_start else None
            )
        elif isinstance(fixed_start, dict):
            record["fixedStartTime"] = fixed_start.get("value")
        else:
            record["fixedStartTime"] = None

        if isinstance(fixed_stop, list):
            record["fixedStopTime"] = (
                fixed_stop[0].get("value")
                if fixed_stop and isinstance(fixed_stop[0], dict)
                else fixed_stop[0] if fixed_stop else None
            )
        elif isinstance(fixed_stop, dict):
            record["fixedStopTime"] = fixed_stop.get("value")
        else:
            record["fixedStopTime"] = None

        # Coordinates and identifiers from target
        try:
            target = sb.get("target", {}) or {}
            record["targetId"] = target.get("id_")
            record["targetName"] = target.get("name")

            coordinates = target.get("position_", {}).get("coord", {})
            celestial = coordinates.get("celestial", {})
            record["decInDeg"] = celestial.get("decInDeg")
            record["raInDeg"] = celestial.get("raInDeg")
        except (AttributeError, TypeError) as e:
            logger.warning(
                f"Failed to extract coordinates for block {record['schedulingBlockId']}: {e}"
            )
            record["targetId"] = None
            record["targetName"] = None
            record["decInDeg"] = None
            record["raInDeg"] = None

        # Azimuth constraints
        azimuth_constraint = constraints.get("azimuthConstraint_", {})
        record["minAzimuthAngleInDeg"] = azimuth_constraint.get("minAzimuthAngleInDeg")
        record["maxAzimuthAngleInDeg"] = azimuth_constraint.get("maxAzimuthAngleInDeg")

        # Elevation constraints
        elevation_constraint = constraints.get("elevationConstraint_", {})
        record["minElevationAngleInDeg"] = elevation_constraint.get("minElevationAngleInDeg")
        record["maxElevationAngleInDeg"] = elevation_constraint.get("maxElevationAngleInDeg")

        # Scheduled period (optional)
        scheduled_period = sb.get("scheduled_period")
        if scheduled_period:
            start_time = scheduled_period.get("startTime", {})
            stop_time = scheduled_period.get("stopTime", {})
            start_value = start_time.get("value")

            # Check for invalid sentinel value (51910.5 indicates unscheduled)
            if start_value == 51910.5:
                record["scheduled_period.start"] = None
                record["scheduled_period.stop"] = None
            else:
                record["scheduled_period.start"] = start_value
                record["scheduled_period.stop"] = stop_time.get("value")
        else:
            record["scheduled_period.start"] = None
            record["scheduled_period.stop"] = None

        return record

    def _extract_visibility_periods(self, sb_id: str) -> list[tuple[float, float]]:
        """
        Extract visibility periods for a scheduling block.

        Args:
            sb_id: Scheduling block ID

        Returns:
            List of (start, stop) tuples in MJD
        """
        if not self.visibility_data:
            return []

        sb_visibility = self.visibility_data.get("SchedulingBlock", {}).get(str(sb_id), [])

        periods = []
        for period in sb_visibility:
            start_value = period.get("startTime", {}).get("value")
            stop_value = period.get("stopTime", {}).get("value")
            if start_value is not None and stop_value is not None:
                periods.append((start_value, stop_value))

        return periods

    def _calculate_visibility_stats(
        self, visibility_periods: list[tuple[float, float]]
    ) -> tuple[int, float]:
        """
        Calculate visibility statistics from periods.

        Args:
            visibility_periods: List of (start, stop) tuples in MJD

        Returns:
            Tuple of (num_periods, total_hours)
        """
        if not visibility_periods:
            return 0, 0.0

        num_periods = len(visibility_periods)
        total_days = sum(stop - start for start, stop in visibility_periods)
        total_hours = total_days * 24.0

        return num_periods, total_hours

    def _assign_priority_bin(self, priority: float) -> str:
        """
        Assign a priority bin label.

        Astronomical scheduling can use any positive priority value.
        Higher values indicate more critical observations.

        Args:
            priority: Priority value (any positive real number)

        Returns:
            Priority bin label
        """
        if pd.isna(priority):
            return "Unknown"
        elif priority < 0:
            return "Invalid (<0)"
        elif priority < 8:
            return "Low (0-8)"
        elif priority < 10:
            return "Medium (8-10)"
        elif priority < 12:
            return "High (10-12)"
        elif priority < 15:
            return "Very High (12-15)"
        else:
            return "Critical (>15)"

    def extract_dataframe(self) -> pd.DataFrame:
        """
        Extract scheduling blocks into a DataFrame.

        Returns:
            DataFrame with basic scheduling block data
        """
        if not self.schedule_data:
            raise ValueError("No data loaded. Call load_data() first.")

        scheduling_blocks = self.schedule_data.get("SchedulingBlock", [])

        records = []
        for sb in scheduling_blocks:
            try:
                record = self._extract_scheduling_block(sb)
                records.append(record)
            except Exception as e:
                sb_id = sb.get("schedulingBlockId", "UNKNOWN")
                logger.error(f"Failed to parse scheduling block {sb_id}: {e}", exc_info=True)
                continue

        self.df = pd.DataFrame(records)
        logger.info(f"Extracted {len(self.df)} scheduling blocks into DataFrame")

        return self.df

    def enrich_with_visibility(self, visibility_path: str | Path | None = None) -> pd.DataFrame:
        """
        Add visibility data to the DataFrame.

        Returns:
            DataFrame with visibility columns added
        """
        if self.df is None:
            raise ValueError("DataFrame not created. Call extract_dataframe() first.")

        if visibility_path is not None:
            self.visibility_path = Path(visibility_path)
            if self.visibility_path.exists():
                logger.info(f"Loading visibility data from {self.visibility_path}")
                with open(self.visibility_path) as f:
                    self.visibility_data = json.load(f)
            else:
                logger.warning(f"Visibility file not found: {self.visibility_path}")
                self.visibility_data = None

        if self.visibility_data is None and self.visibility_path and self.visibility_path.exists():
            logger.info(f"Loading visibility data from {self.visibility_path}")
            with open(self.visibility_path) as f:
                self.visibility_data = json.load(f)

        if self.visibility_data:
            logger.info("Enriching with visibility data...")

            # Extract visibility periods for each block
            self.df["visibility"] = self.df["schedulingBlockId"].apply(
                lambda sb_id: self._extract_visibility_periods(sb_id)
            )

            # Calculate visibility statistics
            visibility_stats = self.df["visibility"].apply(self._calculate_visibility_stats)
            self.df["num_visibility_periods"] = visibility_stats.apply(lambda x: x[0])
            self.df["total_visibility_hours"] = visibility_stats.apply(lambda x: x[1])
        else:
            logger.warning("No visibility data available. Setting default values.")
            self.df["visibility"] = [[] for _ in range(len(self.df))]
            self.df["num_visibility_periods"] = 0
            self.df["total_visibility_hours"] = 0.0

        return self.df

    def add_derived_columns(self) -> pd.DataFrame:
        """
        Add all derived columns needed by the app.

        This includes:
        - scheduled_flag
        - requested_hours
        - elevation_range_deg
        - priority_bin

        Returns:
            DataFrame with derived columns added
        """
        if self.df is None:
            raise ValueError("DataFrame not created. Call extract_dataframe() first.")

        logger.info("Adding derived columns...")

        # Ensure numeric types for calculations
        numeric_columns = [
            "priority",
            "minObservationTimeInSec",
            "requestedDurationSec",
            "decInDeg",
            "raInDeg",
            "minAzimuthAngleInDeg",
            "maxAzimuthAngleInDeg",
            "minElevationAngleInDeg",
            "maxElevationAngleInDeg",
            "scheduled_period.start",
            "scheduled_period.stop",
            "fixedStartTime",
            "fixedStopTime",
        ]

        for col in numeric_columns:
            if col in self.df.columns:
                self.df[col] = pd.to_numeric(self.df[col], errors="coerce")

        # Scheduled flag
        self.df["scheduled_flag"] = (
            self.df["scheduled_period.start"].notna() & self.df["scheduled_period.stop"].notna()
        )

        # Requested hours
        self.df["requested_hours"] = self.df["requestedDurationSec"] / 3600.0

        # Elevation range
        self.df["elevation_range_deg"] = (
            self.df["maxElevationAngleInDeg"] - self.df["minElevationAngleInDeg"]
        )

        # Priority bin
        self.df["priority_bin"] = self.df["priority"].apply(self._assign_priority_bin)

        logger.info("Derived columns added successfully")

        return self.df

    def validate(self) -> ValidationResult:
        """
        Validate the preprocessed data for integrity and quality.

        Returns:
            ValidationResult with validation status and details
        """
        if self.df is None:
            raise ValueError("DataFrame not created. Call extract_dataframe() first.")

        errors = []
        warnings = []
        stats = {}

        # Basic stats
        stats["total_blocks"] = len(self.df)
        stats["scheduled_blocks"] = (
            self.df["scheduled_flag"].sum() if "scheduled_flag" in self.df.columns else 0
        )
        stats["unscheduled_blocks"] = stats["total_blocks"] - stats["scheduled_blocks"]

        # Check for missing IDs
        if self.df["schedulingBlockId"].isna().any():
            errors.append(f"{self.df['schedulingBlockId'].isna().sum()} blocks have missing IDs")

        # Check for duplicate IDs
        duplicate_ids = self.df["schedulingBlockId"].duplicated().sum()
        if duplicate_ids > 0:
            errors.append(f"{duplicate_ids} duplicate scheduling block IDs found")

        # Check priority values
        if "priority" in self.df.columns:
            # Only check for negative priorities (invalid)
            # Any positive real number is valid for astronomical scheduling
            invalid_priority = self.df[(self.df["priority"].notna()) & (self.df["priority"] < 0)]
            if len(invalid_priority) > 0:
                errors.append(f"{len(invalid_priority)} blocks have negative priority (invalid)")

            missing_priority = self.df["priority"].isna().sum()
            if missing_priority > 0:
                warnings.append(f"{missing_priority} blocks have missing priority")

        # Check coordinates
        if "decInDeg" in self.df.columns:
            invalid_dec = self.df[
                (self.df["decInDeg"].notna())
                & ((self.df["decInDeg"] < -90) | (self.df["decInDeg"] > 90))
            ]
            if len(invalid_dec) > 0:
                errors.append(
                    f"{len(invalid_dec)} blocks have invalid declination (outside [-90, 90])"
                )

            missing_dec = self.df["decInDeg"].isna().sum()
            if missing_dec > 0:
                warnings.append(f"{missing_dec} blocks have missing declination")

        if "raInDeg" in self.df.columns:
            invalid_ra = self.df[
                (self.df["raInDeg"].notna())
                & ((self.df["raInDeg"] < 0) | (self.df["raInDeg"] >= 360))
            ]
            if len(invalid_ra) > 0:
                errors.append(
                    f"{len(invalid_ra)} blocks have invalid right ascension (outside [0, 360))"
                )

            missing_ra = self.df["raInDeg"].isna().sum()
            if missing_ra > 0:
                warnings.append(f"{missing_ra} blocks have missing right ascension")

        # Check time constraints
        if "requestedDurationSec" in self.df.columns:
            invalid_duration = self.df[
                (self.df["requestedDurationSec"].notna()) & (self.df["requestedDurationSec"] <= 0)
            ]
            if len(invalid_duration) > 0:
                errors.append(
                    f"{len(invalid_duration)} blocks have invalid requested duration (≤ 0)"
                )

        # Check elevation constraints
        if (
            "minElevationAngleInDeg" in self.df.columns
            and "maxElevationAngleInDeg" in self.df.columns
        ):
            invalid_elevation = self.df[
                (self.df["minElevationAngleInDeg"].notna())
                & (self.df["maxElevationAngleInDeg"].notna())
                & (self.df["minElevationAngleInDeg"] >= self.df["maxElevationAngleInDeg"])
            ]
            if len(invalid_elevation) > 0:
                errors.append(f"{len(invalid_elevation)} blocks have min elevation ≥ max elevation")

        # Check scheduled periods consistency
        if (
            "scheduled_period.start" in self.df.columns
            and "scheduled_period.stop" in self.df.columns
        ):
            invalid_period = self.df[
                (self.df["scheduled_period.start"].notna())
                & (self.df["scheduled_period.stop"].notna())
                & (self.df["scheduled_period.start"] >= self.df["scheduled_period.stop"])
            ]
            if len(invalid_period) > 0:
                errors.append(f"{len(invalid_period)} blocks have start time ≥ stop time")

        # Check visibility data
        if "visibility" in self.df.columns:
            blocks_with_vis_count: int = int((self.df["visibility"].apply(len) > 0).sum())
            avg_vis_periods: float = float(self.df["num_visibility_periods"].mean())
            avg_vis_hours: float = float(self.df["total_visibility_hours"].mean())

            stats["blocks_with_visibility"] = blocks_with_vis_count  # type: ignore[typeddict-item,assignment]
            stats["avg_visibility_periods"] = avg_vis_periods  # type: ignore[typeddict-item,assignment]
            stats["avg_visibility_hours"] = avg_vis_hours  # type: ignore[typeddict-item,assignment]

        # Priority distribution
        if "priority_bin" in self.df.columns:
            priority_dist = self.df["priority_bin"].value_counts().to_dict()
            priority_dist_typed: dict[str, int] = {str(k): int(v) for k, v in priority_dist.items()}
            stats["priority_distribution"] = priority_dist_typed  # type: ignore[typeddict-item,assignment]

        is_valid = len(errors) == 0

        return ValidationResult(is_valid=is_valid, errors=errors, warnings=warnings, stats=stats)

    def to_csv(self, output_path: str | Path, validate: bool = True) -> Path:
        """
        Export the preprocessed data to CSV.

        Args:
            output_path: Path where the CSV will be saved
            validate: Whether to validate before exporting

        Returns:
            Path to the saved CSV file

        Raises:
            ValueError: If validation fails and validate=True
        """
        if self.df is None:
            raise ValueError("DataFrame not created. Run preprocessing first.")

        output_path = Path(output_path)

        if validate:
            logger.info("Validating data before export...")
            result = self.validate()

            if not result.is_valid:
                error_msg = "Validation failed:\n" + "\n".join(f"  - {e}" for e in result.errors)
                raise ValueError(error_msg)

            if result.warnings:
                logger.warning("Validation warnings:")
                for warning in result.warnings:
                    logger.warning(f"  - {warning}")

        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Define column order (same as expected by app)
        columns_order = [
            "schedulingBlockId",
            "targetId",
            "targetName",
            "priority",
            "minObservationTimeInSec",
            "requestedDurationSec",
            "fixedStartTime",
            "fixedStopTime",
            "decInDeg",
            "raInDeg",
            "minAzimuthAngleInDeg",
            "maxAzimuthAngleInDeg",
            "minElevationAngleInDeg",
            "maxElevationAngleInDeg",
            "scheduled_period.start",
            "scheduled_period.stop",
            "visibility",
            "num_visibility_periods",
            "total_visibility_hours",
            "priority_bin",
            # Derived columns required by the Streamlit loaders
            "scheduled_flag",
            "requested_hours",
            "elevation_range_deg",
        ]

        # Ensure all columns exist
        for col in columns_order:
            if col not in self.df.columns:
                logger.warning(f"Column {col} not found in DataFrame, adding as None")
                self.df[col] = None

        # Export in correct order
        df_export = self.df[columns_order].copy()

        # Convert visibility list to string representation
        df_export["visibility"] = df_export["visibility"].apply(str)

        logger.info(f"Exporting to {output_path}")
        df_export.to_csv(output_path, index=False)
        logger.info(f"Successfully exported {len(df_export)} rows to {output_path}")

        return output_path

    def process(self, output_path: str | Path, validate: bool = True) -> Path:
        """
        Run the complete preprocessing pipeline.

        This is a convenience method that runs all steps:
        1. Load data
        2. Extract DataFrame
        3. Enrich with visibility
        4. Add derived columns
        5. Validate
        6. Export to CSV

        Args:
            output_path: Path where the CSV will be saved
            validate: Whether to validate before exporting

        Returns:
            Path to the saved CSV file
        """
        logger.info("Starting preprocessing pipeline...")

        self.load_data()
        self.extract_dataframe()
        self.enrich_with_visibility()
        self.add_derived_columns()

        return self.to_csv(output_path, validate=validate)


def _compute_basic_stats(df: pd.DataFrame) -> tuple[int, int, int]:
    """Compute core scheduling statistics from the processed dataframe."""

    total_blocks = int(len(df))
    if "scheduled_flag" in df.columns:
        scheduled_blocks = int(df["scheduled_flag"].fillna(False).astype(bool).sum())
    else:
        scheduled_blocks = 0
    unscheduled_blocks = total_blocks - scheduled_blocks

    return total_blocks, scheduled_blocks, unscheduled_blocks


def _export_parquet(df: pd.DataFrame, output_path: Path) -> Path:
    """Persist the dataframe as a parquet artefact with graceful fallbacks."""

    df_export = df.copy()
    if "visibility" in df_export.columns:
        df_export["visibility"] = df_export["visibility"].apply(str)

    try:
        df_export.to_parquet(output_path, index=False)
    except (ImportError, ValueError, TypeError) as exc:
        logger.warning(
            "Unable to write parquet file at %s (%s); storing pickle fallback instead.",
            output_path,
            exc,
        )
        df_export.to_pickle(output_path)

    return output_path


def preprocess_schedule(
    schedule_path: str | Path,
    visibility_path: str | Path | None = None,
    output_dir: str | Path | None = None,
    *,
    validate: bool = True,
) -> PreprocessResult:
    """Preprocess a single schedule JSON file.

    The helper mirrors the typical steps executed by :class:`SchedulePreprocessor`
    while providing a convenient return object containing the processed dataframe
    and metadata about the run. Optionally, the results can be exported to disk
    using the canonical filenames expected by the Streamlit loaders.

    Args:
        schedule_path: Path to ``schedule.json``.
        visibility_path: Optional path to ``possible_periods.json``.
        output_dir: Directory where artefacts should be written. When provided,
            ``schedule_preprocessed.csv`` and ``schedule_preprocessed.parquet``
            will be created (the latter falling back to a pickle if parquet
            engines are unavailable).
        validate: Whether to run validation rules and include the
            :class:`ValidationResult` in the metadata.

    Returns:
        :class:`PreprocessResult` with the processed dataframe and metadata.
    """

    preprocessor = SchedulePreprocessor(schedule_path, visibility_path)
    preprocessor.load_data()
    preprocessor.extract_dataframe()
    preprocessor.enrich_with_visibility()
    preprocessor.add_derived_columns()

    validation: ValidationResult | None = preprocessor.validate() if validate else None

    csv_path: Path | None = None
    parquet_path: Path | None = None
    if output_dir is not None:
        output_directory = Path(output_dir)
        output_directory.mkdir(parents=True, exist_ok=True)

        csv_path = output_directory / "schedule_preprocessed.csv"
        # Validation already executed; skip redundant checks.
        preprocessor.to_csv(csv_path, validate=False)

        parquet_path = output_directory / "schedule_preprocessed.parquet"
        if preprocessor.df is not None:
            _export_parquet(preprocessor.df, parquet_path)

    dataframe = preprocessor.df.copy() if preprocessor.df is not None else pd.DataFrame()

    if validation is not None:
        total_blocks = int(validation.stats.get("total_blocks", len(dataframe)))
        scheduled_blocks = int(validation.stats.get("scheduled_blocks", 0))
        unscheduled_blocks = int(
            validation.stats.get("unscheduled_blocks", total_blocks - scheduled_blocks)
        )
    else:
        total_blocks, scheduled_blocks, unscheduled_blocks = _compute_basic_stats(dataframe)

    metadata = PreprocessMetadata(
        total_blocks=total_blocks,
        scheduled_blocks=scheduled_blocks,
        unscheduled_blocks=unscheduled_blocks,
        csv_path=csv_path,
        parquet_path=parquet_path,
        validation=validation,
    )

    return PreprocessResult(dataframe=dataframe, metadata=metadata)


def preprocess_iteration(
    schedule_source: str | Path,
    visibility_source: str | Path | None,
    output_dir: str | Path | None,
    *,
    validate: bool = True,
) -> PreprocessResult:
    """Preprocess a schedule and optional visibility artefacts.

    The helper accepts either a direct path to ``schedule.json`` or a
    data directory containing ``schedule.json``. When an output
    directory is provided, canonical CSV and parquet artefacts are created.
    ``visibility_source`` can point to a specific JSON file or be ``None`` to
    rely on the data directory layout.
    """

    schedule_path = Path(schedule_source)
    visibility_path: Path | None

    if schedule_path.is_dir():
        # Try new structure first: data_dir/schedule.json
        candidate = schedule_path / "schedule.json"
        if not candidate.exists():
            # Try legacy structure: data_dir/schedule/schedule.json
            candidate = schedule_path / "schedule" / "schedule.json"
            if not candidate.exists():
                raise FileNotFoundError(f"Schedule file not found: {candidate}")
        schedule_path = candidate

        if visibility_source is not None:
            visibility_path = Path(visibility_source)
        else:
            # Try new structure first: data_dir/possible_periods.json
            default_visibility = schedule_path.parent / "possible_periods.json"
            if not default_visibility.exists():
                # Try legacy structure
                default_visibility = (
                    schedule_path.parent.parent / "possible periods" / "possible_periods.json"
                )
            visibility_path = default_visibility if default_visibility.exists() else None
    else:
        if not schedule_path.exists():
            raise FileNotFoundError(f"Schedule file not found: {schedule_path}")
        visibility_path = Path(visibility_source) if visibility_source else None

    return preprocess_schedule(
        schedule_path,
        visibility_path,
        output_dir=output_dir,
        validate=validate,
    )
