#!/usr/bin/env python3
"""
CP-SAT Feasibility Checker CLI Tool

A command-line tool for analyzing observation schedule feasibility using
constraint programming (CP-SAT). Checks if a subset of observations can be
scheduled without conflicts.

Usage:
    python3 -m py_lab.cli --schedule <path> --possible-periods <path> --num-obs <N>
    
Example:
    python3 -m py_lab.cli \
        --schedule data/schedule.json \
        --possible-periods data/possible_periods.json \
        --num-obs 50 \
        --priority 7
"""

import sys
import argparse
from pathlib import Path
from typing import Optional

from py_lab import (
    load_schedule,
    filter_by_priority,
    ConflictAnalyzer,
    validate_schedule_dataframe,
    export_to_json
)


def filter_impossible_observations(df):
    """
    Filter out observations with no visibility or zero/negative duration.
    
    Args:
        df: DataFrame with observations
        
    Returns:
        Tuple of (filtered DataFrame, stats dict)
    """
    original_count = len(df)
    
    # Remove rows with zero or negative duration
    if 'requested_duration' in df.columns:
        df_filtered = df[df['requested_duration'] > 0].copy()
        duration_removed = original_count - len(df_filtered)
    else:
        df_filtered = df.copy()
        duration_removed = 0
    
    # Remove rows with missing or empty visibility_periods
    has_visibility = df_filtered['visibility_periods'].apply(
        lambda x: x is not None and len(x) > 0 if isinstance(x, list) else False
    )
    df_final = df_filtered[has_visibility].copy()
    visibility_removed = len(df_filtered) - len(df_final)
    
    return df_final, {
        'original': original_count,
        'zero_duration_removed': duration_removed,
        'no_visibility_removed': visibility_removed,
        'final': len(df_final)
    }


def analyze_feasibility(
    schedule_path: Path,
    possible_periods_path: Optional[Path],
    num_observations: int,
    min_priority: Optional[int] = None,
    max_iterations: int = 50,
    output_dir: Optional[Path] = None,
    verbose: bool = True
):
    """
    Run feasibility analysis on a schedule subset.
    
    Args:
        schedule_path: Path to schedule JSON file
        possible_periods_path: Optional path to possible periods JSON file
        num_observations: Number of observations to analyze
        min_priority: Minimum priority filter (optional)
        max_iterations: Maximum iterations for conflict detection
        output_dir: Optional directory for output files
        verbose: Whether to print detailed progress
        
    Returns:
        ConflictResult object
    """
    if verbose:
        print("=" * 70)
        print("CP-SAT Feasibility Checker")
        print("=" * 70)
    
    # Step 1: Load schedule
    if verbose:
        print("\n[1/5] Loading schedule...")
    
    if not schedule_path.exists():
        print(f"Error: Schedule file not found at {schedule_path}")
        sys.exit(1)
    
    if possible_periods_path and not possible_periods_path.exists():
        print(f"Error: Possible periods file not found at {possible_periods_path}")
        sys.exit(1)
    
    df = load_schedule(
        schedule_path=schedule_path,
        possible_periods_path=possible_periods_path
    )
    
    if verbose:
        print(f"  ✓ Loaded {len(df)} candidate observations")
    
    # Step 2: Validate DataFrame
    if verbose:
        print("\n[2/5] Validating data...")
    
    is_valid, missing = validate_schedule_dataframe(df)
    if not is_valid:
        print(f"  ✗ Missing required columns: {missing}")
        sys.exit(1)
    
    if verbose:
        print(f"  ✓ Data validated")
    
    # Step 3: Preprocess - filter impossible observations
    if verbose:
        print("\n[3/5] Preprocessing observations...")
    
    df_feasible, stats = filter_impossible_observations(df)
    
    if verbose:
        print(f"  ✓ Filtered to {stats['final']} possible observations")
        print(f"    - Zero duration removed: {stats['zero_duration_removed']}")
        print(f"    - No visibility removed: {stats['no_visibility_removed']}")
    
    # Optional: filter by priority
    if min_priority is not None and 'priority' in df_feasible.columns:
        df_filtered = filter_by_priority(df_feasible, min_priority=min_priority)
        if verbose:
            print(f"  ✓ Priority filter (≥{min_priority}): {len(df_filtered)} observations")
        df_to_analyze = df_filtered
    else:
        df_to_analyze = df_feasible
    
    # Select subset
    if num_observations > len(df_to_analyze):
        if verbose:
            print(f"  ⚠ Requested {num_observations} observations but only {len(df_to_analyze)} available")
        num_observations = len(df_to_analyze)
    
    df_to_analyze = df_to_analyze.head(num_observations)
    
    if verbose:
        print(f"\n  Analyzing subset of {len(df_to_analyze)} observations...")
    
    # Step 4: Convert to CP-SAT Tasks and check feasibility
    if verbose:
        print("\n[4/5] Checking scheduling feasibility with CP-SAT...")
    
    analyzer = ConflictAnalyzer()
    result = analyzer.check_feasibility(df_to_analyze)
    
    if result.feasible:
        if verbose:
            print(f"  ✓ FEASIBLE: All {len(df_to_analyze)} observations can be scheduled!")
    else:
        if verbose:
            print(f"  ✗ INFEASIBLE: Conflicts detected")
        
        # Step 5: Find minimal infeasible subset (conflicts)
        if verbose:
            print("\n[5/5] Finding minimal infeasible subset...")
        
        conflict_result = analyzer.find_conflicts(df_to_analyze, max_iterations=max_iterations)
        
        if conflict_result.infeasible_tasks:
            if verbose:
                print(f"  ✓ Found {len(conflict_result.infeasible_tasks)} conflicting observations:")
                for task_id in conflict_result.infeasible_tasks[:10]:
                    print(f"      - {task_id}")
                
                if len(conflict_result.infeasible_tasks) > 10:
                    print(f"      ... and {len(conflict_result.infeasible_tasks) - 10} more")
            
            # Export conflict report if output directory provided
            if output_dir:
                output_dir.mkdir(parents=True, exist_ok=True)
                output_path = output_dir / "conflict_report.json"
                export_to_json(conflict_result.details, output_path)
                
                if verbose:
                    print(f"\n  Report saved to: {output_path}")
            
            result = conflict_result
        else:
            if verbose:
                print("  Could not isolate minimal conflict set")
    
    if verbose:
        print("\n" + "=" * 70)
        print("Analysis complete!")
        print("=" * 70)
    
    return result


def main():
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Check observation schedule feasibility using CP-SAT constraint programming",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Basic usage
  %(prog)s --schedule data/schedule.json --possible-periods data/possible_periods.json --num-obs 50
  
  # With priority filter
  %(prog)s -s data/schedule.json -p data/possible_periods.json -n 100 --priority 7
  
  # Save conflict report
  %(prog)s -s data/schedule.json -p data/possible_periods.json -n 50 --output output/
  
  # Quiet mode (minimal output)
  %(prog)s -s data/schedule.json -p data/possible_periods.json -n 50 --quiet
        """
    )
    
    parser.add_argument(
        "-s", "--schedule",
        type=Path,
        required=True,
        help="Path to schedule JSON file"
    )
    
    parser.add_argument(
        "-p", "--possible-periods",
        type=Path,
        default=None,
        help="Path to possible periods JSON file (optional)"
    )
    
    parser.add_argument(
        "-n", "--num-obs",
        type=int,
        required=True,
        help="Number of observations to analyze"
    )
    
    parser.add_argument(
        "--priority",
        type=int,
        default=None,
        help="Minimum priority filter (e.g., 7 for high priority only)"
    )
    
    parser.add_argument(
        "--max-iterations",
        type=int,
        default=50,
        help="Maximum iterations for conflict detection (default: 50)"
    )
    
    parser.add_argument(
        "-o", "--output",
        type=Path,
        default=None,
        help="Output directory for conflict reports"
    )
    
    parser.add_argument(
        "-q", "--quiet",
        action="store_true",
        help="Quiet mode - minimal output"
    )
    
    args = parser.parse_args()
    
    # Run analysis
    result = analyze_feasibility(
        schedule_path=args.schedule,
        possible_periods_path=args.possible_periods,
        num_observations=args.num_obs,
        min_priority=args.priority,
        max_iterations=args.max_iterations,
        output_dir=args.output,
        verbose=not args.quiet
    )
    
    # Exit code based on feasibility
    sys.exit(0 if result.feasible else 1)


if __name__ == "__main__":
    main()
