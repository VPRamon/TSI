#!/usr/bin/env python3
"""
Schedule Preprocessing CLI

⚠️ LEGACY SCRIPT - Uses deprecated API

This script used the old core.loaders API which has been replaced by the Rust backend.
For current usage, see:
- tsi_rust_api.py (TSIBackend class)
- Rust backend functions in tsi_rust module

This script is kept for reference but may require updates to work with current codebase.

Original Usage:
    # Process a single schedule JSON file
    python preprocess_schedules.py --schedule data/schedule.json --output data/schedule.csv
    
    # Process with visibility/possible periods data
    python preprocess_schedules.py --schedule data/schedule.json --visibility data/possible_periods.json --output data/schedule.csv
    
    # Batch process multiple schedule files in a directory
    python preprocess_schedules.py --batch-dir data/schedules --output-dir data/preprocessed
    
    # Process with validation disabled (faster but not recommended)
    python preprocess_schedules.py --schedule data/schedule.json --output data/schedule.csv --no-validate
    
    # Batch process with pattern matching
    python preprocess_schedules.py --batch-dir data/schedules --pattern "schedule_*.json" --output-dir data/preprocessed
"""

import argparse
import logging
import sys
from pathlib import Path
from typing import List, Optional

import pandas as pd

# Add project root to path for imports
PROJECT_ROOT = Path(__file__).parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

# LEGACY: core.loaders no longer exists - use tsi_rust_api.TSIBackend instead
# from src.core.loaders import load_schedule_from_json
from src.tsi_rust_api import TSIBackend

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def process_single_schedule(
    schedule_path: Path,
    output_path: Path,
    visibility_path: Optional[Path] = None,
    validate: bool = True,
    verbose: bool = False,
) -> bool:
    """
    Process a single schedule JSON file.
    
    Args:
        schedule_path: Path to schedule JSON file
        output_path: Path for output CSV
        visibility_path: Optional path to visibility/possible periods JSON
        validate: Whether to validate data (note: validation is now handled by Rust)
        verbose: Whether to print detailed stats
        
    Returns:
        True if successful, False otherwise
    """
    try:
        logger.info(f"Processing {schedule_path.name}...")
        
        # Load data using Rust backend
        backend = TSIBackend()
        result_dict = backend.load_schedule(
            str(schedule_path), 
            str(visibility_path) if visibility_path else None,
        )
        
        # Convert to DataFrame
        df = pd.DataFrame(result_dict.get("blocks", []))
        
        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Export to CSV (stringify visibility column if present)
        df_export = df.copy()
        if "visibility" in df_export.columns:
            df_export["visibility"] = df_export["visibility"].apply(str)
        
        df_export.to_csv(output_path, index=False)
        
        # Show stats if verbose
        if verbose:
            logger.info(f"✓ Successfully processed {schedule_path.name}")
            logger.info(f"  Output: {output_path}")
            logger.info(f"  Total blocks: {len(df_export)}")
            logger.info(f"  Columns: {list(df_export.columns)}")
            
            if 'visibility' in df_export.columns:
                blocks_with_vis = df_export['visibility'].notna().sum()
                logger.info(f"  Blocks with visibility: {blocks_with_vis}")
        else:
            logger.info(f"✓ Successfully processed {schedule_path.name} -> {output_path}")
        
        return True
        
    except Exception as e:
        logger.error(f"✗ Failed to process {schedule_path.name}: {e}", exc_info=True)
        return False


def batch_process(
    batch_dir: Path,
    output_dir: Path,
    pattern: str = "schedule*.json",
    visibility_pattern: Optional[str] = None,
    validate: bool = True,
    verbose: bool = False,
) -> tuple[int, int]:
    """
    Batch process multiple schedule JSON files.
    
    Args:
        batch_dir: Directory containing schedule JSON files
        output_dir: Output directory for CSVs
        pattern: Glob pattern for schedule JSON files
        visibility_pattern: Optional glob pattern for visibility JSON files
        validate: Whether to validate data
        verbose: Whether to print detailed stats
        
    Returns:
        Tuple of (successful_count, failed_count)
    """
    if not batch_dir.exists():
        logger.error(f"Batch directory not found: {batch_dir}")
        return 0, 0
    
    # Find schedule JSON files
    schedule_files = list(batch_dir.glob(pattern))
    
    if not schedule_files:
        logger.warning(f"No schedule files found matching pattern '{pattern}' in {batch_dir}")
        return 0, 0
    
    logger.info(f"Found {len(schedule_files)} schedule files to process")
    
    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Build visibility file map if pattern provided
    visibility_map = {}
    if visibility_pattern:
        visibility_files = list(batch_dir.glob(visibility_pattern))
        for vis_file in visibility_files:
            # Match by similar base name
            visibility_map[vis_file.stem] = vis_file
    
    # Process each schedule file
    successful = 0
    failed = 0
    
    for schedule_file in sorted(schedule_files):
        # Determine output path
        output_name = schedule_file.stem + '.csv'
        output_path = output_dir / output_name
        
        # Try to find matching visibility file
        visibility_file = None
        if visibility_pattern:
            # Try exact match first
            if schedule_file.stem in visibility_map:
                visibility_file = visibility_map[schedule_file.stem]
            # Try possible_periods or similar naming
            else:
                vis_candidates = [
                    batch_dir / 'possible_periods.json',
                    batch_dir / f'{schedule_file.stem}_possible_periods.json',
                    batch_dir / f'{schedule_file.stem}_visibility.json',
                ]
                for candidate in vis_candidates:
                    if candidate.exists():
                        visibility_file = candidate
                        break
        
        if process_single_schedule(
            schedule_file, 
            output_path, 
            visibility_file,
            validate, 
            verbose
        ):
            successful += 1
        else:
            failed += 1
    
    return successful, failed


def main():
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Preprocess scheduling JSON files to CSV format",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    
    # Input options
    input_group = parser.add_mutually_exclusive_group(required=True)
    input_group.add_argument(
        '--schedule',
        type=Path,
        help='Path to schedule JSON file'
    )
    input_group.add_argument(
        '--batch-dir',
        type=Path,
        help='Path to directory containing multiple schedule JSON files'
    )
    
    # Output options
    parser.add_argument(
        '--output',
        type=Path,
        help='Output CSV path (required for --schedule)'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        help='Output directory for batch processing (required for --batch-dir)'
    )
    
    # Additional input options
    parser.add_argument(
        '--visibility',
        type=Path,
        help='Path to visibility/possible periods JSON file (optional, for --schedule)'
    )
    
    # Processing options
    parser.add_argument(
        '--pattern',
        default='schedule*.json',
        help='Glob pattern for batch processing schedule files (default: schedule*.json)'
    )
    parser.add_argument(
        '--visibility-pattern',
        help='Glob pattern for batch processing visibility files (optional)'
    )
    parser.add_argument(
        '--no-validate',
        action='store_true',
        help='Disable data validation (faster but not recommended)'
    )
    parser.add_argument(
        '--verbose',
        '-v',
        action='store_true',
        help='Print detailed statistics for each file'
    )
    
    args = parser.parse_args()
    
    # Validate arguments
    if args.schedule and not args.output:
        parser.error('--output is required when using --schedule')
    
    if args.batch_dir and not args.output_dir:
        parser.error('--output-dir is required when using --batch-dir')
    
    if args.visibility and args.batch_dir:
        parser.error('--visibility can only be used with --schedule, not --batch-dir')
    
    validate = not args.no_validate
    
    # Process
    if args.schedule:
        # Single schedule file
        if not args.schedule.exists():
            logger.error(f"Schedule file not found: {args.schedule}")
            sys.exit(1)
        
        # Check visibility file if provided
        visibility_path = None
        if args.visibility:
            if not args.visibility.exists():
                logger.error(f"Visibility file not found: {args.visibility}")
                sys.exit(1)
            visibility_path = args.visibility
        
        success = process_single_schedule(
            args.schedule,
            args.output,
            visibility_path=visibility_path,
            validate=validate,
            verbose=args.verbose
        )
        
        sys.exit(0 if success else 1)
    
    else:
        # Batch processing
        successful, failed = batch_process(
            args.batch_dir,
            args.output_dir,
            pattern=args.pattern,
            visibility_pattern=args.visibility_pattern,
            validate=validate,
            verbose=args.verbose
        )
        
        logger.info("=" * 60)
        logger.info(f"Batch processing complete:")
        logger.info(f"  Successful: {successful}")
        logger.info(f"  Failed: {failed}")
        logger.info(f"  Total: {successful + failed}")
        
        sys.exit(0 if failed == 0 else 1)


if __name__ == '__main__':
    main()
