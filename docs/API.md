# TSI Backend API Documentation

**Base URL**: `http://localhost:8081`

**Version**: 1.0  
**Last Updated**: November 9, 2025

---

## Table of Contents

1. [Health Check](#health-check)
2. [Dataset Management](#dataset-management)
3. [Analytics Endpoints](#analytics-endpoints)
4. [Visualization Endpoints](#visualization-endpoints)
5. [Comparison Endpoints](#comparison-endpoints)

---

## Health Check

### GET /health

Check if the backend service is running.

**Response:**
```json
{
  "status": "ok"
}
```

**Status Codes:**
- `200 OK` - Service is healthy

---

## Dataset Management

### POST /api/v1/datasets/upload/csv

Upload a CSV file containing preprocessed scheduling blocks.

**Request:**
- Content-Type: `multipart/form-data`
- Body: File field named `file` with CSV data

**Response:**
```json
{
  "message": "Dataset loaded successfully",
  "metadata": {
    "filename": "schedule.csv",
    "num_blocks": 2647,
    "num_scheduled": 2131,
    "num_unscheduled": 516,
    "loaded_at": "2025-11-09T22:11:46.606613197Z"
  }
}
```

**Status Codes:**
- `200 OK` - File uploaded and processed successfully
- `400 Bad Request` - Invalid file format or missing data
- `500 Internal Server Error` - Server error during processing

---

### POST /api/v1/datasets/upload/json

Upload raw JSON schedule data with optional visibility periods.

**Request:**
- Content-Type: `multipart/form-data`
- Body: One or two files
  - Required: `schedule` (schedule.json)
  - Optional: `visibility` (possible_periods.json)

**Response:**
```json
{
  "message": "JSON dataset processed successfully",
  "metadata": {
    "filename": "schedule.json",
    "num_blocks": 2647,
    "num_scheduled": 0,
    "num_unscheduled": 2647,
    "loaded_at": "2025-11-09T22:11:46.606613197Z"
  }
}
```

**Status Codes:**
- `200 OK` - File uploaded and processed successfully
- `400 Bad Request` - Invalid JSON format or missing required fields
- `500 Internal Server Error` - Server error during processing

---

### POST /api/v1/datasets/sample

Load the bundled sample dataset.

**Response:**
```json
{
  "message": "Sample dataset loaded successfully",
  "metadata": {
    "filename": "schedule.csv (sample)",
    "num_blocks": 2647,
    "num_scheduled": 2131,
    "num_unscheduled": 516,
    "loaded_at": "2025-11-09T22:11:46.606613197Z"
  }
}
```

**Status Codes:**
- `200 OK` - Sample dataset loaded successfully
- `500 Internal Server Error` - Sample file not found or corrupted

---

### GET /api/v1/datasets/current/metadata

Get metadata for the currently loaded dataset.

**Response:**
```json
{
  "filename": "schedule.csv",
  "num_blocks": 2647,
  "num_scheduled": 2131,
  "num_unscheduled": 516,
  "loaded_at": "2025-11-09T22:11:46.606613197Z"
}
```

**Status Codes:**
- `200 OK` - Metadata retrieved successfully
- `404 Not Found` - No dataset currently loaded

---

### GET /api/v1/datasets/current

Get the entire currently loaded dataset (all scheduling blocks).

**Query Parameters:** None

**Response:**
```json
{
  "blocks": [
    {
      "scheduling_block_id": "1000004379",
      "priority": 19.2,
      "right_ascension_deg": 258.651,
      "declination_deg": -29.882,
      "scheduled_flag": true,
      ...
    }
  ],
  "metadata": { ... }
}
```

**Status Codes:**
- `200 OK` - Dataset retrieved successfully
- `404 Not Found` - No dataset currently loaded

---

### DELETE /api/v1/datasets/current

Clear the currently loaded dataset from memory.

**Response:**
```json
{
  "message": "Dataset cleared successfully"
}
```

**Status Codes:**
- `200 OK` - Dataset cleared successfully

---

## Analytics Endpoints

### GET /api/v1/analytics/metrics

Get overall metrics and statistics for the loaded dataset.

**Response:**
```json
{
  "total_blocks": 2647,
  "scheduled_blocks": 2131,
  "unscheduled_blocks": 516,
  "scheduling_rate": 80.51,
  "total_requested_hours": 1252.99,
  "total_scheduled_hours": 969.29,
  "total_visibility_hours": 746000.71,
  "utilization": 0.13,
  "priority_stats": {
    "mean": 12.65,
    "median": 12.5,
    "std": 5.23,
    "min": 0.0,
    "max": 28.2
  },
  "visibility_hours_stats": { ... },
  "requested_hours_stats": { ... }
}
```

**Status Codes:**
- `200 OK` - Metrics computed successfully
- `404 Not Found` - No dataset loaded

---

### GET /api/v1/analytics/correlations

Compute Spearman correlation between specified columns.

**Query Parameters:**
- `columns` (optional): Comma-separated list of column names. Default: `priority,total_visibility_hours,requested_hours`

**Example:** `/api/v1/analytics/correlations?columns=priority,visibility_hours`

**Response:**
```json
{
  "columns": ["priority", "total_visibility_hours"],
  "matrix": [
    [1.0, 0.142],
    [0.142, 1.0]
  ]
}
```

**Status Codes:**
- `200 OK` - Correlations computed successfully
- `400 Bad Request` - Invalid column names
- `404 Not Found` - No dataset loaded

---

### GET /api/v1/analytics/conflicts

Detect scheduling conflicts and impossible observations.

**Response:**
```json
{
  "impossible_observations": [
    {
      "scheduling_block_id": "1000005123",
      "priority": 15.5,
      "requested_hours": 8.0,
      "total_visibility_hours": 6.0,
      "deficit_hours": 2.0
    }
  ],
  "total_conflicts": 1
}
```

**Status Codes:**
- `200 OK` - Conflicts detected successfully
- `404 Not Found` - No dataset loaded

---

### GET /api/v1/analytics/top

Get top N observations by a specified metric.

**Query Parameters:**
- `by` (default: `priority`): Sort metric. Options: `priority`, `requested_hours`, `visibility_hours`
- `n` (default: `10`): Number of results to return
- `scheduled_only` (default: `false`): Filter to scheduled observations only

**Example:** `/api/v1/analytics/top?by=priority&n=20&scheduled_only=true`

**Response:**
```json
{
  "observations": [
    {
      "scheduling_block_id": "1000004968",
      "priority": 28.2,
      "right_ascension_deg": 297.096,
      "declination_deg": -11.056,
      "requested_hours": 5.282,
      "scheduled_flag": false
    }
  ],
  "count": 20,
  "sort_by": "priority"
}
```

**Status Codes:**
- `200 OK` - Top observations retrieved successfully
- `400 Bad Request` - Invalid parameters
- `404 Not Found` - No dataset loaded

---

### GET /api/v1/analytics/distribution

Compute histogram distribution for a specified column.

**Query Parameters:**
- `column` (required): Column name. Options: `priority`, `requested_hours`, `visibility_hours`, `elevation_range_deg`
- `bins` (default: `20`): Number of histogram bins

**Example:** `/api/v1/analytics/distribution?column=priority&bins=30`

**Response:**
```json
{
  "column": "priority",
  "bins": [
    {
      "lower": 0.0,
      "upper": 1.5,
      "count": 125,
      "frequency": 0.047
    }
  ],
  "stats": {
    "mean": 12.65,
    "median": 12.5,
    "std": 5.23,
    "min": 0.0,
    "max": 28.2,
    "q1": 8.5,
    "q3": 16.8
  }
}
```

**Status Codes:**
- `200 OK` - Distribution computed successfully
- `400 Bad Request` - Invalid column name or bins parameter
- `404 Not Found` - No dataset loaded

---

### GET /api/v1/analytics/trends

Compute time series trends for scheduling metrics.

**Query Parameters:**
- `metric` (default: `scheduling_rate`): Metric to track. Options: `scheduling_rate`, `utilization`, `avg_priority`
- `group_by` (default: `month`): Time grouping. Options: `month`, `week`, `day`

**Example:** `/api/v1/analytics/trends?metric=utilization&group_by=week`

**Response:**
```json
{
  "metric": "scheduling_rate",
  "group_by": "month",
  "data": [
    {
      "period": "2027-02",
      "value": 100.0,
      "count": 119
    },
    {
      "period": "2027-03",
      "value": 100.0,
      "count": 316
    }
  ]
}
```

**Status Codes:**
- `200 OK` - Trends computed successfully
- `400 Bad Request` - Invalid parameters
- `404 Not Found` - No dataset loaded

---

## Visualization Endpoints

### GET /api/v1/visualizations/visibility-map

Get detailed visibility information for a specific scheduling block.

**Query Parameters:**
- `block_id` (required): Scheduling block ID

**Example:** `/api/v1/visualizations/visibility-map?block_id=1000004968`

**Response:**
```json
{
  "scheduling_block_id": "1000004968",
  "right_ascension_deg": 297.096,
  "declination_deg": -11.056,
  "requested_hours": 5.282,
  "total_visibility_hours": 5.252,
  "priority": 28.2,
  "scheduled_flag": false,
  "visibility_periods": [
    {
      "start": 62035.017,
      "stop": 62035.235
    }
  ],
  "azimuth_min_deg": 0.0,
  "azimuth_max_deg": 360.0,
  "elevation_min_deg": 20.0,
  "elevation_max_deg": 90.0,
  "elevation_range_deg": 70.0
}
```

**Status Codes:**
- `200 OK` - Visibility data retrieved successfully
- `400 Bad Request` - Missing block_id parameter
- `404 Not Found` - No dataset loaded or block not found

---

### GET /api/v1/visualizations/timeline

Get all scheduled observations with optional time filtering.

**Query Parameters:**
- `month` (optional): Month number (1-12)
- `year` (optional): Year (e.g., 2028)

**Example:** `/api/v1/visualizations/timeline?month=1&year=2028`

**Response:**
```json
{
  "observations": [
    {
      "scheduling_block_id": "1000004379",
      "scheduled_time_mjd": 61771.0,
      "scheduled_time_iso": "2028-01-01T00:00:00Z",
      "scheduled_duration_hours": 0.5,
      "priority": 19.2,
      "priority_bin": "High (10+)",
      "right_ascension_deg": 258.651,
      "declination_deg": -29.882
    }
  ],
  "total_count": 2131,
  "month": 1,
  "year": 2028
}
```

**Status Codes:**
- `200 OK` - Timeline data retrieved successfully
- `400 Bad Request` - Invalid month or year
- `404 Not Found` - No dataset loaded

---

## Comparison Endpoints

### POST /api/v1/datasets/comparison/upload

Upload a comparison dataset for side-by-side analysis.

**Request:**
- Content-Type: `multipart/form-data`
- Body: File field named `file` with CSV data

**Response:**
```json
{
  "message": "Comparison dataset 'schedule2.csv' uploaded successfully",
  "metadata": {
    "filename": "schedule2.csv",
    "num_blocks": 2500,
    "num_scheduled": 2000,
    "num_unscheduled": 500,
    "loaded_at": "2025-11-09T22:30:00Z"
  }
}
```

**Status Codes:**
- `200 OK` - Comparison dataset uploaded successfully
- `400 Bad Request` - Invalid file format or missing data
- `500 Internal Server Error` - Server error during processing

**Note**: Large files (>10MB) may fail due to multipart body size limits. Consider splitting or compressing large datasets.

---

### GET /api/v1/analytics/compare

Compare primary and comparison datasets.

**Response:**
```json
{
  "primary": {
    "filename": "schedule1.csv",
    "total_blocks": 2647,
    "scheduled_blocks": 2131,
    "unscheduled_blocks": 516,
    "scheduling_rate": 80.51,
    "total_requested_hours": 1252.99,
    "total_scheduled_hours": 969.29,
    "total_visibility_hours": 746000.71,
    "utilization": 0.13,
    "avg_priority": 12.65,
    "avg_requested_hours": 0.473,
    "avg_visibility_hours": 281.83
  },
  "comparison": {
    "filename": "schedule2.csv",
    "total_blocks": 2500,
    "scheduled_blocks": 2000,
    "unscheduled_blocks": 500,
    "scheduling_rate": 80.0,
    "total_requested_hours": 1180.5,
    "total_scheduled_hours": 900.0,
    "total_visibility_hours": 720000.0,
    "utilization": 0.125,
    "avg_priority": 13.2,
    "avg_requested_hours": 0.472,
    "avg_visibility_hours": 288.0
  },
  "diff": {
    "blocks_added": 50,
    "blocks_removed": 197,
    "blocks_unchanged": 2400,
    "blocks_modified": 50,
    "newly_scheduled": 20,
    "newly_unscheduled": 15,
    "scheduling_rate_diff": -0.51,
    "utilization_diff": -0.005,
    "avg_priority_diff": 0.55
  },
  "changes": [
    {
      "scheduling_block_id": "1000003976",
      "change_type": "removed",
      "primary_scheduled": true,
      "comparison_scheduled": null,
      "primary_priority": 6.75,
      "comparison_priority": null
    },
    {
      "scheduling_block_id": "1000009999",
      "change_type": "added",
      "primary_scheduled": null,
      "comparison_scheduled": false,
      "primary_priority": null,
      "comparison_priority": 15.5
    },
    {
      "scheduling_block_id": "1000004379",
      "change_type": "modified",
      "primary_scheduled": true,
      "comparison_scheduled": false,
      "primary_priority": 19.2,
      "comparison_priority": 19.2
    }
  ]
}
```

**Change Types:**
- `added` - Block exists only in comparison dataset
- `removed` - Block exists only in primary dataset
- `modified` - Block exists in both but scheduling status or priority changed
- `unchanged` - Block exists in both with identical status and priority

**Status Codes:**
- `200 OK` - Comparison computed successfully
- `404 Not Found` - Primary or comparison dataset not loaded

---

### DELETE /api/v1/datasets/comparison

Clear the comparison dataset from memory.

**Response:**
```json
{
  "message": "Comparison dataset cleared successfully"
}
```

**Status Codes:**
- `200 OK` - Comparison dataset cleared successfully

---

## Error Responses

All endpoints may return errors in the following format:

```json
{
  "error": "Error message",
  "details": "Optional detailed error information"
}
```

**Common Status Codes:**
- `400 Bad Request` - Invalid request parameters or malformed data
- `404 Not Found` - Requested resource not found
- `500 Internal Server Error` - Server-side error during processing

---

## Data Models

### SchedulingBlock

```typescript
{
  scheduling_block_id: string,
  priority: number,
  min_observation_time_in_sec: number,
  requested_duration_sec: number,
  fixed_start_time: number | null,
  fixed_stop_time: number | null,
  dec_in_deg: number,
  ra_in_deg: number,
  min_azimuth_angle_in_deg: number,
  max_azimuth_angle_in_deg: number,
  min_elevation_angle_in_deg: number,
  max_elevation_angle_in_deg: number,
  scheduled_period_start: number | null,
  scheduled_period_stop: number | null,
  visibility: Array<{ start: number, stop: number }>,
  num_visibility_periods: number,
  total_visibility_hours: number,
  priority_bin: "No priority" | "Low (0-5)" | "Medium (5-8)" | "Medium (8-10)" | "High (10+)",
  scheduled_flag: boolean,
  requested_hours: number,
  elevation_range_deg: number
}
```

### DatasetMetadata

```typescript
{
  filename: string,
  num_blocks: number,
  num_scheduled: number,
  num_unscheduled: number,
  loaded_at: string  // ISO 8601 timestamp
}
```

---

## Rate Limiting

Currently, there is no rate limiting implemented. All endpoints can be called without restriction.

## Authentication

Currently, there is no authentication required. All endpoints are publicly accessible.

---

## CORS

CORS is enabled for all origins during development. For production deployment, restrict CORS to specific domains.

---

## Notes

- **MJD Format**: All times in the API use Modified Julian Date (MJD) format. Days since November 17, 1858.
- **ISO Timestamps**: Timeline endpoint converts MJD to ISO 8601 format for easier consumption.
- **Memory Management**: All datasets are stored in memory. Large datasets (>1GB) may cause performance issues.
- **File Uploads**: Multipart uploads have a body size limit. Files larger than 10MB may fail to upload.

---

## Example Usage

### Python

```python
import requests

BASE_URL = "http://localhost:8081"

# Load sample dataset
response = requests.post(f"{BASE_URL}/api/v1/datasets/sample")
print(response.json())

# Get metrics
metrics = requests.get(f"{BASE_URL}/api/v1/analytics/metrics").json()
print(f"Scheduling Rate: {metrics['scheduling_rate']}%")

# Upload comparison dataset
with open("schedule2.csv", "rb") as f:
    files = {"file": f}
    response = requests.post(
        f"{BASE_URL}/api/v1/datasets/comparison/upload",
        files=files
    )
    print(response.json())

# Get comparison
comparison = requests.get(f"{BASE_URL}/api/v1/analytics/compare").json()
print(f"Blocks Added: {comparison['diff']['blocks_added']}")
```

### JavaScript/TypeScript

```typescript
const BASE_URL = "http://localhost:8081";

// Load sample dataset
const loadSample = async () => {
  const response = await fetch(`${BASE_URL}/api/v1/datasets/sample`, {
    method: "POST",
  });
  const data = await response.json();
  console.log(data);
};

// Get trends
const getTrends = async () => {
  const response = await fetch(
    `${BASE_URL}/api/v1/analytics/trends?metric=scheduling_rate&group_by=month`
  );
  const data = await response.json();
  console.log(data);
};

// Upload comparison
const uploadComparison = async (file: File) => {
  const formData = new FormData();
  formData.append("file", file);

  const response = await fetch(`${BASE_URL}/api/v1/datasets/comparison/upload`, {
    method: "POST",
    body: formData,
  });
  const data = await response.json();
  console.log(data);
};
```

### curl

```bash
# Load sample dataset
curl -X POST http://localhost:8081/api/v1/datasets/sample

# Get metrics
curl http://localhost:8081/api/v1/analytics/metrics

# Upload CSV
curl -X POST -F "file=@schedule.csv" \
  http://localhost:8081/api/v1/datasets/upload/csv

# Get visibility map
curl "http://localhost:8081/api/v1/visualizations/visibility-map?block_id=1000004968"

# Get trends
curl "http://localhost:8081/api/v1/analytics/trends?metric=utilization&group_by=week"

# Upload comparison and compare
curl -X POST -F "file=@schedule2.csv" \
  http://localhost:8081/api/v1/datasets/comparison/upload

curl http://localhost:8081/api/v1/analytics/compare
```

---

**For issues or questions, please refer to the README.md or open an issue on GitHub.**
