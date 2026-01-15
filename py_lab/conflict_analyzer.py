"""
Conflict Analyzer Module for py_lab

Integrates the CP-SAT constraint solver with schedule DataFrames to detect
scheduling conflicts, find minimal infeasible subsets, and analyze observation
proposal feasibility.
"""

from typing import List, Tuple, Dict, Optional, Set, Any
from dataclasses import dataclass
import pandas as pd

from .cp_sat import (
    Task,
    can_schedule,
    find_minimal_infeasible_subset,
    find_max_schedulable_from_group
)


@dataclass
class ConflictResult:
    """Result from conflict analysis."""
    feasible: bool
    infeasible_tasks: Optional[List[str]] = None
    message: str = ""
    details: Optional[Dict[str, Any]] = None


class ConflictAnalyzer:
    """
    Analyzer for detecting scheduling conflicts using CP-SAT solver.
    
    Converts schedule DataFrame rows into Task objects and applies
    constraint programming to find conflicts and maximal schedulable sets.
    """
    
    def __init__(self, time_unit: float = 60.0):
        """
        Initialize the conflict analyzer.
        
        Args:
            time_unit: Conversion factor from schedule time units to solver units.
                      Default 60.0 converts minutes to seconds (if schedule uses minutes).
                      For MJD (days), use 86400.0 to convert to seconds.
        """
        self.time_unit = time_unit
    
    def df_to_tasks(
        self, 
        df: pd.DataFrame,
        time_scale: float = 86400.0  # MJD days to seconds
    ) -> List[Task]:
        """
        Convert DataFrame rows to Task objects for CP-SAT solver.
        
        Args:
            df: DataFrame with columns: id, requested_duration, visibility_periods
            time_scale: Multiplier to convert time values to integer units
                       (e.g., 86400 converts MJD fractional days to seconds)
            
        Returns:
            List of Task objects suitable for CP-SAT solving
            
        Note:
            visibility_periods should be a list of dicts with 'start' and 'stop' keys,
            or a list of tuples (start, stop), in MJD format.
        """
        tasks = []
        
        for idx, row in df.iterrows():
            # Extract required fields
            task_id = str(row.get('id', row.get('original_block_id', idx)))
            duration_raw = row.get('requested_duration', 0)
            visibility = row.get('visibility_periods', [])
            
            # Skip if no visibility or zero duration
            if not visibility or duration_raw <= 0:
                continue
            
            # Convert duration to integer seconds
            duration_int = int(duration_raw)
            
            # Convert visibility periods to integer time windows
            periods = []
            for period in visibility:
                if isinstance(period, dict):
                    start = period.get('start', 0)
                    stop = period.get('stop', 0)
                elif isinstance(period, (tuple, list)) and len(period) >= 2:
                    start, stop = period[0], period[1]
                else:
                    continue
                
                # Convert MJD to integer seconds (scale up for precision)
                start_int = int(start * time_scale)
                stop_int = int(stop * time_scale)
                
                if stop_int > start_int:
                    periods.append((start_int, stop_int))
            
            if periods:
                tasks.append(Task(
                    id=task_id,
                    duration=duration_int,
                    periods=periods
                ))
        
        return tasks
    
    def check_feasibility(
        self, 
        df: pd.DataFrame,
        time_scale: float = 86400.0
    ) -> ConflictResult:
        """
        Check if all observations in the DataFrame can be scheduled without conflicts.
        
        Args:
            df: DataFrame with scheduling blocks
            time_scale: Time conversion scale factor
            
        Returns:
            ConflictResult indicating feasibility
        """
        tasks = self.df_to_tasks(df, time_scale)
        
        if not tasks:
            return ConflictResult(
                feasible=True,
                message="No valid tasks to schedule (zero duration or no visibility)"
            )
        
        feasible = can_schedule(tasks)
        
        if feasible:
            return ConflictResult(
                feasible=True,
                message=f"All {len(tasks)} tasks can be scheduled without conflicts"
            )
        else:
            return ConflictResult(
                feasible=False,
                message=f"Conflicts detected among {len(tasks)} tasks",
                details={"task_count": len(tasks)}
            )
    
    def find_conflicts(
        self,
        df: pd.DataFrame,
        time_scale: float = 86400.0,
        max_iterations: int = 100
    ) -> ConflictResult:
        """
        Find minimal infeasible subset (MIS) of conflicting observations.
        
        Args:
            df: DataFrame with scheduling blocks
            time_scale: Time conversion scale factor
            max_iterations: Maximum deletion iterations for MIS search
            
        Returns:
            ConflictResult with infeasible task IDs
        """
        tasks = self.df_to_tasks(df, time_scale)
        
        if not tasks:
            return ConflictResult(
                feasible=True,
                message="No valid tasks to analyze"
            )
        
        # Check if feasible first
        if can_schedule(tasks):
            return ConflictResult(
                feasible=True,
                message=f"All {len(tasks)} tasks are schedulable (no conflicts)"
            )
        
        # Find minimal infeasible subset
        mis = find_minimal_infeasible_subset(tasks, max_iterations)
        
        if mis is None:
            return ConflictResult(
                feasible=False,
                message="Infeasible but could not isolate minimal conflict set",
                details={"task_count": len(tasks)}
            )
        
        conflict_ids = [t.id for t in mis]
        
        return ConflictResult(
            feasible=False,
            infeasible_tasks=conflict_ids,
            message=f"Found minimal infeasible subset: {len(conflict_ids)} conflicting tasks",
            details={
                "total_tasks": len(tasks),
                "conflict_size": len(conflict_ids),
                "conflict_ids": conflict_ids
            }
        )
    
    def find_max_schedulable(
        self,
        df: pd.DataFrame,
        k: int = 1,
        time_scale: float = 86400.0
    ) -> Dict[str, Any]:
        """
        Find maximum number of observations schedulable from a group with "at most k" constraint.
        
        Args:
            df: DataFrame with scheduling blocks (typically from same group/priority)
            k: At most k tasks from this group can be scheduled
            time_scale: Time conversion scale factor
            
        Returns:
            Dictionary with max schedulable count and selected task IDs
        """
        tasks = self.df_to_tasks(df, time_scale)
        
        if not tasks:
            return {
                "max_schedulable": 0,
                "selected_tasks": [],
                "message": "No valid tasks to analyze"
            }
        
        max_count, selected = find_max_schedulable_from_group(tasks, k)
        selected_ids = [t.id for t in selected] if selected else []
        
        return {
            "max_schedulable": max_count,
            "selected_tasks": selected_ids,
            "total_tasks": len(tasks),
            "constraint_k": k,
            "message": f"Maximum {max_count} out of {len(tasks)} tasks can be scheduled (constraint: at most {k})"
        }
    
    def analyze_priority_groups(
        self,
        df: pd.DataFrame,
        time_scale: float = 86400.0,
        priority_bins: Optional[List[float]] = None
    ) -> pd.DataFrame:
        """
        Analyze scheduling feasibility by priority groups.
        
        Args:
            df: DataFrame with scheduling blocks
            time_scale: Time conversion scale factor
            priority_bins: Custom priority bin edges (default: [0, 5, 7, 9, 10])
            
        Returns:
            DataFrame with conflict analysis per priority group
        """
        if priority_bins is None:
            priority_bins = [0, 5, 7, 9, 10]
        
        if 'priority' not in df.columns:
            raise ValueError("DataFrame must have 'priority' column")
        
        results = []
        
        for i in range(len(priority_bins) - 1):
            low, high = priority_bins[i], priority_bins[i + 1]
            group_df = df[(df['priority'] >= low) & (df['priority'] < high)]
            
            if len(group_df) == 0:
                continue
            
            # Check feasibility for this group
            feasibility = self.check_feasibility(group_df, time_scale)
            
            # If infeasible, find conflicts
            conflict_info = None
            if not feasibility.feasible:
                conflict_info = self.find_conflicts(group_df, time_scale)
            
            results.append({
                'priority_range': f"{low}-{high}",
                'total_blocks': len(group_df),
                'feasible': feasibility.feasible,
                'conflict_count': len(conflict_info.infeasible_tasks) if conflict_info and conflict_info.infeasible_tasks else 0,
                'message': feasibility.message if feasibility.feasible else conflict_info.message
            })
        
        return pd.DataFrame(results)


def detect_conflicts(
    df: pd.DataFrame,
    time_scale: float = 86400.0,
    max_iterations: int = 100
) -> ConflictResult:
    """
    Convenience function to detect conflicts in a schedule DataFrame.
    
    Args:
        df: DataFrame with scheduling blocks
        time_scale: Time conversion scale (default: MJD days to seconds)
        max_iterations: Maximum iterations for MIS search
        
    Returns:
        ConflictResult with conflict information
        
    Example:
        >>> from py_lab.data_loader import load_schedule
        >>> from py_lab.conflict_analyzer import detect_conflicts
        >>> 
        >>> df, _ = load_schedule("schedule.json")
        >>> result = detect_conflicts(df)
        >>> 
        >>> if not result.feasible:
        ...     print(f"Conflicts found: {result.infeasible_tasks}")
    """
    analyzer = ConflictAnalyzer()
    return analyzer.find_conflicts(df, time_scale, max_iterations)


def get_schedulable_subset(
    df: pd.DataFrame,
    k: int = 1,
    time_scale: float = 86400.0
) -> List[str]:
    """
    Get IDs of maximum schedulable observations from a group.
    
    Args:
        df: DataFrame with scheduling blocks
        k: At most k tasks constraint
        time_scale: Time conversion scale
        
    Returns:
        List of task IDs that can be scheduled
    """
    analyzer = ConflictAnalyzer()
    result = analyzer.find_max_schedulable(df, k, time_scale)
    return result["selected_tasks"]


if __name__ == "__main__":
    """Example usage and testing"""
    import sys
    from pathlib import Path
    
    # Add parent to path for imports
    sys.path.insert(0, str(Path(__file__).parent.parent))
    
    from py_lab.data_loader import load_schedule
    
    print("=" * 60)
    print("Conflict Analyzer - py_lab")
    print("=" * 60)
    
    try:
        # Load a test schedule
        print("\nLoading schedule...")
        df, _ = load_schedule("schedule_test.json")
        
        print(f"Loaded {len(df)} scheduling blocks")
        
        # Initialize analyzer
        analyzer = ConflictAnalyzer()
        
        # Test 1: Check overall feasibility
        print("\n" + "=" * 60)
        print("Test 1: Overall Feasibility Check")
        print("=" * 60)
        
        result = analyzer.check_feasibility(df)
        print(f"Feasible: {result.feasible}")
        print(f"Message: {result.message}")
        
        # Test 2: Find conflicts (if any)
        if not result.feasible:
            print("\n" + "=" * 60)
            print("Test 2: Finding Minimal Infeasible Subset")
            print("=" * 60)
            
            conflict = analyzer.find_conflicts(df)
            print(f"Conflicts found: {len(conflict.infeasible_tasks) if conflict.infeasible_tasks else 0}")
            if conflict.infeasible_tasks:
                print(f"Conflicting task IDs: {conflict.infeasible_tasks[:10]}...")  # Show first 10
        
        # Test 3: Analyze by priority groups
        if 'priority' in df.columns:
            print("\n" + "=" * 60)
            print("Test 3: Priority Group Analysis")
            print("=" * 60)
            
            group_analysis = analyzer.analyze_priority_groups(df)
            print(group_analysis.to_string(index=False))
        
        # Test 4: High priority subset
        print("\n" + "=" * 60)
        print("Test 4: High Priority Subset (priority >= 8)")
        print("=" * 60)
        
        if 'priority' in df.columns:
            high_priority = df[df['priority'] >= 8].head(20)  # Limit to 20 for testing
            print(f"Testing {len(high_priority)} high-priority blocks...")
            
            hp_result = analyzer.check_feasibility(high_priority)
            print(f"Feasible: {hp_result.feasible}")
            print(f"Message: {hp_result.message}")
        
        print("\n" + "=" * 60)
        print("Conflict analyzer module is ready!")
        print("=" * 60)
        
    except Exception as e:
        print(f"\nError: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)
