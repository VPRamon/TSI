"""Data models and schemas."""

from datetime import datetime

from pydantic import BaseModel, Field


class SchedulingBlock(BaseModel):
    """Schema for a single scheduling block observation."""

    schedulingBlockId: int = Field(..., description="Unique identifier")
    priority: float = Field(..., ge=0, le=10, description="Observation priority (0-10)")
    minObservationTimeInSec: float = Field(..., ge=0, description="Minimum observation time")
    requestedDurationSec: float = Field(..., ge=0, description="Requested duration")
    fixedStartTime: float | None = Field(None, description="Fixed start constraint (MJD)")
    fixedStopTime: float | None = Field(None, description="Fixed stop constraint (MJD)")
    decInDeg: float = Field(..., ge=-90, le=90, description="Declination in degrees")
    raInDeg: float = Field(..., ge=0, lt=360, description="Right Ascension in degrees")
    minAzimuthAngleInDeg: float = Field(..., ge=0, le=360)
    maxAzimuthAngleInDeg: float = Field(..., ge=0, le=360)
    minElevationAngleInDeg: float = Field(..., ge=0, le=90)
    maxElevationAngleInDeg: float = Field(..., ge=0, le=90)
    scheduled_period_start: float | None = Field(
        None, alias="scheduled_period.start", description="Scheduled start (MJD)"
    )
    scheduled_period_stop: float | None = Field(
        None, alias="scheduled_period.stop", description="Scheduled stop (MJD)"
    )
    visibility: str = Field(..., description="Visibility periods as string")
    num_visibility_periods: int = Field(..., ge=0)
    total_visibility_hours: float = Field(..., ge=0)
    priority_bin: str = Field(..., description="Priority category")

    class Config:
        """Pydantic config."""

        populate_by_name = True


class AnalyticsMetrics(BaseModel):
    """Computed analytics metrics."""

    total_observations: int
    scheduled_count: int
    unscheduled_count: int
    scheduling_rate: float
    mean_priority: float
    median_priority: float
    mean_priority_scheduled: float
    mean_priority_unscheduled: float
    total_visibility_hours: float
    mean_requested_hours: float


class DatasetSummary(BaseModel):
    """High-level dataset summary."""

    row_count: int
    scheduled_count: int
    mean_priority: float
    total_visibility_hours: float
    date_range_start: datetime | None
    date_range_end: datetime | None
