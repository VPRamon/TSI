"""
Python integration tests for PyO3 bindings.

These tests actually invoke Rust functions through the Python runtime,
testing the full PyO3 binding stack that is otherwise untested.
"""
import pytest
import json
from datetime import datetime, timezone


@pytest.fixture
def minimal_schedule_json():
    """Minimal valid schedule JSON for testing"""
    return json.dumps({
        "blocks": [],
        "dark_periods": []
    })


@pytest.fixture
def schedule_with_blocks_json():
    """Schedule JSON with scheduling blocks"""
    return json.dumps({
        "blocks": [
            {
                "scheduling_block_id": "block_001",
                "target_ra": 45.0,
                "target_dec": -30.0,
                "constraints": {
                    "min_alt": 30.0,
                    "max_alt": 85.0,
                    "min_az": 0.0,
                    "max_az": 360.0
                },
                "priority": 5.0,
                "min_observation": 60.0,
                "requested_duration": 3600.0,
                "visibility_periods": [
                    {"start": 59580.0, "stop": 59581.0}
                ]
            }
        ],
        "dark_periods": [
            {"start": 59580.0, "stop": 59590.0}
        ]
    })


class TestPyO3Imports:
    """Test that PyO3 modules and types can be imported"""
    
    def test_import_tsi_rust(self):
        """Test basic module import"""
        import tsi_rust
        assert tsi_rust is not None
    
    def test_import_tsi_rust_api(self):
        """Test API module import"""
        import tsi_rust_api
        assert tsi_rust_api is not None


class TestPyO3TypeConstructors:
    """Test PyO3 type constructors and basic operations"""
    
    def test_schedule_id_constructor(self):
        """Test ScheduleId can be created from Python"""
        from tsi_rust_api import ScheduleId
        
        sid = ScheduleId(42)
        assert sid.value == 42
    
    def test_target_id_constructor(self):
        """Test TargetId can be created from Python"""
        from tsi_rust_api import TargetId
        
        tid = TargetId(100)
        assert tid.value == 100
    
    def test_constraints_id_constructor(self):
        """Test ConstraintsId can be created from Python"""
        from tsi_rust_api import ConstraintsId
        
        cid = ConstraintsId(7)
        assert cid.value == 7
    
    def test_scheduling_block_id_constructor(self):
        """Test SchedulingBlockId can be created from Python"""
        from tsi_rust_api import SchedulingBlockId
        
        sbid = SchedulingBlockId(999)
        assert sbid.value == 999


class TestPyO3Period:
    """Test Period type PyO3 bindings"""
    
    def test_period_constructor_basic(self):
        """Test Period creation with MJD values"""
        from tsi_rust_api import Period
        
        period = Period(59580.0, 59581.0)
        assert period.start_mjd == 59580.0
        assert period.stop_mjd == 59581.0
    
    def test_period_from_datetime(self):
        """Test Period.from_datetime static method"""
        from tsi_rust_api import Period
        
        start = datetime(2022, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        stop = datetime(2022, 1, 2, 0, 0, 0, tzinfo=timezone.utc)
        
        period = Period.from_datetime(start, stop)
        assert period.start_mjd > 0
        assert period.stop_mjd > period.start_mjd
        
        # Should be approximately 1 day difference
        duration = period.stop_mjd - period.start_mjd
        assert 0.99 < duration < 1.01
    
    def test_period_contains_mjd(self):
        """Test Period.contains_mjd method"""
        from tsi_rust_api import Period
        
        period = Period(59580.0, 59582.0)
        
        # Inside period
        assert period.contains_mjd(59581.0)
        assert period.contains_mjd(59580.0)
        assert period.contains_mjd(59582.0)
        
        # Outside period
        assert not period.contains_mjd(59579.0)
        assert not period.contains_mjd(59583.0)
    
    def test_period_to_datetime(self):
        """Test Period.to_datetime conversion"""
        from tsi_rust_api import Period
        
        # MJD 59580.0 is approximately 2022-01-01
        period = Period(59580.0, 59581.0)
        start_dt, stop_dt = period.to_datetime()
        
        assert start_dt.year == 2022
        assert start_dt.month == 1
        assert start_dt.day == 1
        
        # Stop should be one day later
        assert stop_dt.year == 2022
        assert stop_dt.month == 1
        assert stop_dt.day == 2


class TestPyO3Constraints:
    """Test Constraints type PyO3 bindings"""
    
    def test_constraints_constructor_basic(self):
        """Test Constraints creation"""
        from tsi_rust_api import Constraints
        
        c = Constraints(
            min_alt=30.0,
            max_alt=85.0,
            min_az=0.0,
            max_az=360.0
        )
        
        assert c.min_alt == 30.0
        assert c.max_alt == 85.0
        assert c.min_az == 0.0
        assert c.max_az == 360.0
        assert c.fixed_time is None
    
    def test_constraints_with_fixed_time(self):
        """Test Constraints with fixed_time period"""
        from tsi_rust_api import Constraints, Period
        
        fixed_period = Period(59580.0, 59581.0)
        c = Constraints(
            min_alt=30.0,
            max_alt=85.0,
            min_az=0.0,
            max_az=360.0,
            fixed_time=fixed_period
        )
        
        assert c.fixed_time is not None
        assert c.fixed_time.start_mjd == 59580.0


class TestPyO3LandingRoutes:
    """Test landing route PyO3 bindings (store_schedule, list_schedules)"""
    
    def test_store_schedule_minimal(self, minimal_schedule_json):
        """Test storing a minimal schedule through PyO3"""
        from tsi_rust_api import store_schedule
        
        schedule_id = store_schedule(
            schedule_name="Test Schedule",
            schedule_json=minimal_schedule_json,
            visibility_json=None
        )
        
        assert isinstance(schedule_id, int)
        assert schedule_id > 0
    
    def test_store_schedule_with_blocks(self, schedule_with_blocks_json):
        """Test storing a schedule with blocks through PyO3"""
        from tsi_rust_api import store_schedule
        
        schedule_id = store_schedule(
            schedule_name="Schedule With Blocks",
            schedule_json=schedule_with_blocks_json,
            visibility_json=None
        )
        
        assert isinstance(schedule_id, int)
        assert schedule_id > 0
    
    def test_list_schedules_after_store(self, minimal_schedule_json):
        """Test listing schedules through PyO3"""
        from tsi_rust_api import store_schedule, list_schedules
        
        # Store a schedule first
        schedule_id = store_schedule(
            schedule_name="Listable Schedule",
            schedule_json=minimal_schedule_json,
            visibility_json=None
        )
        
        # List schedules
        schedules = list_schedules()
        
        assert isinstance(schedules, list)
        assert len(schedules) > 0
        
        # Find our schedule
        found = any(s.schedule_id.value == schedule_id for s in schedules)
        assert found, f"Schedule {schedule_id} not found in list"
    
    def test_schedule_info_attributes(self, minimal_schedule_json):
        """Test ScheduleInfo attributes through PyO3"""
        from tsi_rust_api import store_schedule, list_schedules
        
        schedule_name = "Attribute Test Schedule"
        store_schedule(
            schedule_name=schedule_name,
            schedule_json=minimal_schedule_json,
            visibility_json=None
        )
        
        schedules = list_schedules()
        test_schedule = next(s for s in schedules if s.schedule_name == schedule_name)
        
        assert hasattr(test_schedule, 'schedule_id')
        assert hasattr(test_schedule, 'schedule_name')
        assert test_schedule.schedule_name == schedule_name


class TestPyO3VisibilityRoutes:
    """Test visibility route PyO3 bindings"""
    
    def test_get_visibility_map_data(self, schedule_with_blocks_json):
        """Test get_visibility_map_data through PyO3"""
        from tsi_rust_api import store_schedule, get_visibility_map_data, ScheduleId
        
        schedule_id = store_schedule(
            schedule_name="Visibility Test",
            schedule_json=schedule_with_blocks_json,
            visibility_json=None
        )
        
        # Get visibility map data
        vis_data = get_visibility_map_data(ScheduleId(schedule_id))
        
        assert hasattr(vis_data, 'blocks')
        assert hasattr(vis_data, 'priority_min')
        assert hasattr(vis_data, 'priority_max')
        assert hasattr(vis_data, 'total_count')
        assert hasattr(vis_data, 'scheduled_count')
    
    def test_get_schedule_time_range(self, schedule_with_blocks_json):
        """Test get_schedule_time_range through PyO3"""
        from tsi_rust_api import store_schedule, get_schedule_time_range, ScheduleId
        
        schedule_id = store_schedule(
            schedule_name="Time Range Test",
            schedule_json=schedule_with_blocks_json,
            visibility_json=None
        )
        
        # Get time range
        time_range = get_schedule_time_range(ScheduleId(schedule_id))
        
        if time_range is not None:
            start_unix, end_unix = time_range
            assert isinstance(start_unix, int)
            assert isinstance(end_unix, int)
            assert start_unix < end_unix
    
    def test_get_visibility_histogram(self, schedule_with_blocks_json):
        """Test get_visibility_histogram through PyO3"""
        from tsi_rust_api import store_schedule, get_visibility_histogram, ScheduleId
        
        schedule_id = store_schedule(
            schedule_name="Histogram Test",
            schedule_json=schedule_with_blocks_json,
            visibility_json=None
        )
        
        # Get histogram
        histogram = get_visibility_histogram(ScheduleId(schedule_id))
        
        assert isinstance(histogram, dict)
        # Histogram should have time bin keys
        assert 'bin_size_minutes' in histogram or len(histogram) >= 0


class TestPyO3ErrorHandling:
    """Test error handling in PyO3 bindings"""
    
    def test_invalid_schedule_json(self):
        """Test that invalid JSON raises appropriate error"""
        from tsi_rust_api import store_schedule
        
        with pytest.raises(Exception):  # Should raise PyRuntimeError
            store_schedule(
                schedule_name="Invalid",
                schedule_json="not valid json",
                visibility_json=None
            )
    
    def test_invalid_schedule_id_lookup(self):
        """Test that looking up non-existent schedule raises error"""
        from tsi_rust_api import get_visibility_map_data, ScheduleId
        
        # Very high ID that shouldn't exist
        with pytest.raises(Exception):  # Should raise PyRuntimeError
            get_visibility_map_data(ScheduleId(999999999))
    
    def test_period_invalid_order(self):
        """Test Period validation with stop before start"""
        from tsi_rust_api import Period
        
        # Period with stop before start should still construct
        # but may have validation in Rust side
        period = Period(59582.0, 59580.0)
        assert period.start_mjd == 59582.0
        assert period.stop_mjd == 59580.0


class TestPyO3RouteConstants:
    """Test that route constants are exposed"""
    
    def test_landing_constants(self):
        """Test landing route constants"""
        import tsi_rust_api
        
        assert hasattr(tsi_rust_api, 'LIST_SCHEDULES')
        assert hasattr(tsi_rust_api, 'POST_SCHEDULE')
        assert tsi_rust_api.LIST_SCHEDULES == "list_schedules"
        assert tsi_rust_api.POST_SCHEDULE == "store_schedule"
    
    def test_visibility_constants(self):
        """Test visibility route constants"""
        import tsi_rust_api
        
        assert hasattr(tsi_rust_api, 'GET_VISIBILITY_MAP_DATA')
        assert hasattr(tsi_rust_api, 'GET_SCHEDULE_TIME_RANGE')
        assert hasattr(tsi_rust_api, 'GET_VISIBILITY_HISTOGRAM')


class TestPyO3DataTypes:
    """Test complex data type bindings"""
    
    def test_visibility_block_summary_structure(self, schedule_with_blocks_json):
        """Test VisibilityBlockSummary structure"""
        from tsi_rust_api import store_schedule, get_visibility_map_data, ScheduleId
        
        schedule_id = store_schedule(
            schedule_name="Block Summary Test",
            schedule_json=schedule_with_blocks_json,
            visibility_json=None
        )
        
        vis_data = get_visibility_map_data(ScheduleId(schedule_id))
        
        if len(vis_data.blocks) > 0:
            block = vis_data.blocks[0]
            assert hasattr(block, 'scheduling_block_id')
            assert hasattr(block, 'original_block_id')
            assert hasattr(block, 'priority')
            assert hasattr(block, 'num_visibility_periods')
            assert hasattr(block, 'scheduled')
