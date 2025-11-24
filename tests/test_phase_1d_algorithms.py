"""
Integration tests for FASE 1D: Algorithms Core
Tests compute_metrics, find_conflicts, greedy_schedule
"""

import pytest
import pandas as pd
import tsi_rust


def test_compute_metrics_basic():
    """Test basic metrics computation"""
    # Load schedule data
    df = tsi_rust.load_schedule_from_csv("data/schedule.csv")
    
    # Compute metrics
    metrics = tsi_rust.py_compute_metrics(df)
    
    # Check attributes
    assert hasattr(metrics, 'total_observations')
    assert hasattr(metrics, 'scheduled_count')
    assert hasattr(metrics, 'unscheduled_count')
    assert hasattr(metrics, 'scheduling_rate')
    assert hasattr(metrics, 'mean_priority')
    
    print(f"\n✓ Metrics computed:")
    print(f"  - Total observations: {metrics.total_observations}")
    print(f"  - Scheduled: {metrics.scheduled_count}")
    print(f"  - Unscheduled: {metrics.unscheduled_count}")
    print(f"  - Scheduling rate: {metrics.scheduling_rate:.1%}")
    print(f"  - Mean priority: {metrics.mean_priority:.2f}")
    
    # Validate values
    assert metrics.total_observations > 0
    assert metrics.scheduled_count + metrics.unscheduled_count == metrics.total_observations
    assert 0 <= metrics.scheduling_rate <= 1.0


def test_compute_metrics_to_dict():
    """Test metrics conversion to dictionary"""
    df = tsi_rust.load_schedule_from_csv("data/schedule.csv")
    metrics = tsi_rust.py_compute_metrics(df)
    
    # Convert to dict
    metrics_dict = metrics.to_dict()
    
    assert isinstance(metrics_dict, dict)
    assert 'total_observations' in metrics_dict
    assert 'scheduling_rate' in metrics_dict
    
    print(f"\n✓ Metrics dictionary:")
    for key, value in metrics_dict.items():
        print(f"  - {key}: {value}")


def test_get_top_observations():
    """Test getting top N observations by priority"""
    df = tsi_rust.load_schedule_from_csv("data/schedule.csv")
    
    # Get top 10 by priority
    top_df = tsi_rust.py_get_top_observations(df, "priority", 10)
    
    # Convert to pandas
    top_pandas = top_df.to_pandas()
    
    assert len(top_pandas) <= 10
    assert "priority" in top_pandas.columns
    assert "schedulingBlockId" in top_pandas.columns
    
    print(f"\n✓ Top 10 observations by priority:")
    print(f"  - Count: {len(top_pandas)}")
    if len(top_pandas) > 0:
        print(f"  - Highest priority: {top_pandas['priority'].iloc[0]}")
        print(f"  - Lowest in top 10: {top_pandas['priority'].iloc[-1]}")
        
        # Check they're actually sorted descending
        priorities = top_pandas['priority'].tolist()
        assert priorities == sorted(priorities, reverse=True), "Should be sorted descending"


def test_find_conflicts():
    """Test conflict detection"""
    df = tsi_rust.load_schedule_from_csv("data/schedule.csv")
    
    # Find conflicts
    conflicts = tsi_rust.py_find_conflicts(df)
    
    # Should return a list (may be empty)
    assert isinstance(conflicts, list)
    
    print(f"\n✓ Conflict detection:")
    print(f"  - Conflicts found: {len(conflicts)}")
    
    if len(conflicts) > 0:
        first = conflicts[0]
        print(f"  - First conflict ID: {first.scheduling_block_id}")
        print(f"  - Priority: {first.priority}")
        print(f"  - Reasons: {first.conflict_reasons}")


def test_greedy_schedule_basic():
    """Test greedy scheduling algorithm"""
    # Create simple priority list
    priorities = [5.0, 3.0, 8.0, 1.0, 6.0]
    
    # Run greedy scheduler
    result = tsi_rust.py_greedy_schedule(priorities)
    
    # Check result
    assert hasattr(result, 'solution')
    assert hasattr(result, 'objective_value')
    assert hasattr(result, 'iterations')
    assert hasattr(result, 'converged')
    
    print(f"\n✓ Greedy schedule:")
    print(f"  - Selected: {len(result.solution)} observations")
    print(f"  - Indices: {result.solution}")
    print(f"  - Objective value: {result.objective_value:.2f}")
    print(f"  - Iterations: {result.iterations}")
    print(f"  - Converged: {result.converged}")
    
    # With no constraints, should select all
    assert len(result.solution) == len(priorities)
    assert result.converged


def test_greedy_schedule_with_real_data():
    """Test greedy scheduling with real dataset"""
    df = tsi_rust.load_schedule_from_csv("data/schedule.csv")
    df_pandas = df.to_pandas()
    
    # Get priorities
    priorities = df_pandas['priority'].tolist()[:100]  # Use first 100 for speed
    
    # Run optimization
    result = tsi_rust.py_greedy_schedule(priorities, max_iterations=200)
    
    assert len(result.solution) > 0
    assert result.objective_value > 0
    
    print(f"\n✓ Greedy schedule on real data:")
    print(f"  - Input size: {len(priorities)}")
    print(f"  - Selected: {len(result.solution)}")
    print(f"  - Objective: {result.objective_value:.2f}")
    print(f"  - Selection rate: {len(result.solution) / len(priorities):.1%}")


def test_compute_correlations():
    """Test correlation computation"""
    df = tsi_rust.load_schedule_from_csv("data/schedule.csv")
    
    # Compute correlations for selected columns
    columns = ["priority", "requested_hours", "total_visibility_hours"]
    corr_df = tsi_rust.py_compute_correlations(df, columns)
    
    # Convert to pandas
    corr_pandas = corr_df.to_pandas()
    
    print(f"\n✓ Correlation computation:")
    print(f"  - Result shape: {corr_pandas.shape}")
    print(f"  - Result type: {type(corr_pandas)}")
    
    # Note: Current implementation returns empty DataFrame (placeholder)
    # Real implementation would return correlation matrix


def test_analytics_snapshot_repr():
    """Test AnalyticsSnapshot string representation"""
    df = tsi_rust.load_schedule_from_csv("data/schedule.csv")
    metrics = tsi_rust.py_compute_metrics(df)
    
    # Get string representation
    repr_str = repr(metrics)
    
    assert "AnalyticsSnapshot" in repr_str
    assert "total=" in repr_str
    assert "scheduled=" in repr_str
    
    print(f"\n✓ Snapshot repr: {repr_str}")


def test_optimization_result_repr():
    """Test OptimizationResult string representation"""
    priorities = [5.0, 3.0, 8.0]
    result = tsi_rust.py_greedy_schedule(priorities)
    
    repr_str = repr(result)
    
    assert "OptimizationResult" in repr_str
    assert "selected=" in repr_str
    assert "objective=" in repr_str
    
    print(f"\n✓ Result repr: {repr_str}")


if __name__ == "__main__":
    # Run tests with verbose output
    pytest.main([__file__, "-v", "-s"])
