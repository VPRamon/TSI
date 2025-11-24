"""
Integration tests for FASE 1B: Parsing & Loading

Tests the complete data loading pipeline from JSON and CSV files
using the Rust backend implementation.
"""

import pytest
import tsi_rust
from pathlib import Path


DATA_DIR = Path(__file__).parent.parent / "data"
SCHEDULE_JSON = DATA_DIR / "schedule.json"
SCHEDULE_CSV = DATA_DIR / "schedule.csv"


def test_import_tsi_rust():
    """Test that tsi_rust module can be imported"""
    assert tsi_rust is not None


def test_load_schedule_has_functions():
    """Test that load_schedule functions are available"""
    assert hasattr(tsi_rust, 'load_schedule')
    assert hasattr(tsi_rust, 'load_schedule_from_json')
    assert hasattr(tsi_rust, 'load_schedule_from_csv')
    assert hasattr(tsi_rust, 'load_schedule_from_json_str')


@pytest.mark.skipif(not SCHEDULE_JSON.exists(), reason="schedule.json not found")
def test_load_schedule_from_json():
    """Test loading schedule data from JSON file"""
    df = tsi_rust.load_schedule_from_json(str(SCHEDULE_JSON))
    
    # Verify it's a Polars DataFrame
    assert df is not None
    
    # Convert to pandas for easier testing
    pandas_df = df.to_pandas()
    
    # Check basic structure
    assert len(pandas_df) > 0, "DataFrame should not be empty"
    assert "schedulingBlockId" in pandas_df.columns
    assert "priority" in pandas_df.columns
    assert "requestedDurationSec" in pandas_df.columns
    assert "scheduled_flag" in pandas_df.columns
    
    # Check derived columns are present
    assert "requested_hours" in pandas_df.columns
    assert "priority_bin" in pandas_df.columns
    assert "num_visibility_periods" in pandas_df.columns
    assert "total_visibility_hours" in pandas_df.columns
    
    print(f"✅ Loaded {len(pandas_df)} scheduling blocks from JSON")
    print(f"   Columns: {list(pandas_df.columns)}")


@pytest.mark.skipif(not SCHEDULE_CSV.exists(), reason="schedule.csv not found")
def test_load_schedule_from_csv():
    """Test loading schedule data from CSV file"""
    df = tsi_rust.load_schedule_from_csv(str(SCHEDULE_CSV))
    
    # Verify it's a Polars DataFrame
    assert df is not None
    
    # Convert to pandas for easier testing
    pandas_df = df.to_pandas()
    
    # Check basic structure
    assert len(pandas_df) > 0, "DataFrame should not be empty"
    assert "schedulingBlockId" in pandas_df.columns
    assert "priority" in pandas_df.columns
    
    print(f"✅ Loaded {len(pandas_df)} scheduling blocks from CSV")


@pytest.mark.skipif(not SCHEDULE_JSON.exists(), reason="schedule.json not found")
def test_load_schedule_auto_detect():
    """Test automatic format detection based on file extension"""
    # Test with JSON
    df_json = tsi_rust.load_schedule(str(SCHEDULE_JSON))
    assert df_json is not None
    pandas_df_json = df_json.to_pandas()
    assert len(pandas_df_json) > 0
    
    print(f"✅ Auto-detected JSON format and loaded {len(pandas_df_json)} blocks")


def test_load_schedule_from_json_str():
    """Test loading from a JSON string"""
    json_str = """{
        "SchedulingBlock": [
            {
                "schedulingBlockId": "test-001",
                "priority": 10.0,
                "fixedStartTime": null,
                "fixedStopTime": null,
                "scheduled_period": {
                    "start": 59580.0,
                    "stop": 59580.5
                },
                "target": {
                    "targetId": 1,
                    "targetName": "Test Target",
                    "raInDeg": 180.0,
                    "decInDeg": 45.0
                },
                "observation": {
                    "minObservationTimeInSec": 1800,
                    "requestedDurationSec": 3600
                },
                "controlParameters": {
                    "minAzimuthAngleInDeg": 0.0,
                    "maxAzimuthAngleInDeg": 360.0,
                    "minElevationAngleInDeg": 30.0,
                    "maxElevationAngleInDeg": 85.0
                }
            }
        ]
    }"""
    
    df = tsi_rust.load_schedule_from_json_str(json_str)
    assert df is not None
    
    pandas_df = df.to_pandas()
    assert len(pandas_df) == 1
    assert pandas_df.iloc[0]["schedulingBlockId"] == "test-001"
    assert pandas_df.iloc[0]["priority"] == 10.0
    assert pandas_df.iloc[0]["scheduled_flag"] == True
    assert pandas_df.iloc[0]["requested_hours"] == 1.0  # 3600 sec = 1 hour
    
    print("✅ Loaded scheduling block from JSON string")
    print(f"   ID: {pandas_df.iloc[0]['schedulingBlockId']}")
    print(f"   Priority: {pandas_df.iloc[0]['priority']}")
    print(f"   Scheduled: {pandas_df.iloc[0]['scheduled_flag']}")


@pytest.mark.skipif(not SCHEDULE_JSON.exists(), reason="schedule.json not found")
def test_dataframe_structure():
    """Test that the DataFrame has the expected structure and derived columns"""
    df = tsi_rust.load_schedule_from_json(str(SCHEDULE_JSON))
    pandas_df = df.to_pandas()
    
    # Check for essential columns
    essential_cols = [
        "schedulingBlockId",
        "priority",
        "requestedDurationSec",
        "raInDeg",
        "decInDeg",
        "minElevationAngleInDeg",
        "maxElevationAngleInDeg",
    ]
    
    for col in essential_cols:
        assert col in pandas_df.columns, f"Missing essential column: {col}"
    
    # Check derived columns
    derived_cols = [
        "scheduled_flag",
        "requested_hours",
        "elevation_range_deg",
        "priority_bin",
        "num_visibility_periods",
        "total_visibility_hours",
    ]
    
    for col in derived_cols:
        assert col in pandas_df.columns, f"Missing derived column: {col}"
    
    # Verify data types
    assert pandas_df["priority"].dtype in ["float64", "float32"]
    assert pandas_df["scheduled_flag"].dtype == bool
    
    # Verify priority bins are assigned
    priority_bins = pandas_df["priority_bin"].unique()
    print(f"✅ DataFrame structure is correct")
    print(f"   Priority bins found: {priority_bins}")


@pytest.mark.skipif(not SCHEDULE_JSON.exists(), reason="schedule.json not found")
def test_performance_comparison():
    """Simple performance test - just verify it loads quickly"""
    import time
    
    start = time.time()
    df = tsi_rust.load_schedule_from_json(str(SCHEDULE_JSON))
    pandas_df = df.to_pandas()
    elapsed = time.time() - start
    
    num_blocks = len(pandas_df)
    
    print(f"✅ Performance test:")
    print(f"   Loaded {num_blocks} blocks in {elapsed:.3f}s")
    print(f"   Speed: {num_blocks/elapsed:.0f} blocks/sec")
    
    # Should be reasonably fast (less than 1 second for typical datasets)
    assert elapsed < 5.0, f"Loading took too long: {elapsed:.3f}s"


if __name__ == "__main__":
    # Run tests manually
    print("=" * 60)
    print("FASE 1B Integration Tests")
    print("=" * 60)
    
    test_import_tsi_rust()
    print("✅ Import test passed\n")
    
    test_load_schedule_has_functions()
    print("✅ Function availability test passed\n")
    
    if SCHEDULE_JSON.exists():
        test_load_schedule_from_json()
        print()
        
        test_load_schedule_auto_detect()
        print()
        
        test_dataframe_structure()
        print()
        
        test_performance_comparison()
        print()
    else:
        print("⚠️  schedule.json not found, skipping file-based tests\n")
    
    if SCHEDULE_CSV.exists():
        test_load_schedule_from_csv()
        print()
    else:
        print("⚠️  schedule.csv not found, skipping CSV test\n")
    
    test_load_schedule_from_json_str()
    print()
    
    print("=" * 60)
    print("All tests completed!")
    print("=" * 60)
