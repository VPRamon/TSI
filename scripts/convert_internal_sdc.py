#!/usr/bin/env python3
"""
Convert internalSDC scheduler output JSON to TSI schedule.json and possible_periods.json.

Input: JSON file containing scheduler output with:
  - schedulingBlocks: list of ObservationTask entries
  - output.scheduledPeriods: scheduled periods per block UUID
  - output.possiblePeriodsMap: possible periods keyed by constraint+target string

Output:
  - schedule.json: TSI schedule schema (snake_case, MJD periods)
  - possible_periods.json: visibility periods keyed by block ID

Usage:
  # Basic conversion
  python scripts/convert_internal_sdc.py input.json -o output_dir/
  
  # With custom schedule name
  python scripts/convert_internal_sdc.py input.json -o output_dir/ -n "MySchedule"
  
Example:
  python scripts/convert_internal_sdc.py \\
    data/sensitive/sdc_cta_north_AGN_Spectra_N_wobble_Regular_internalSDC.json \\
    -o converted/ \\
    -n "CTA_North_AGN"
"""

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Tuple


# MJD epoch: Modified Julian Date 0 = November 17, 1858, 00:00:00 UTC
# Unix epoch: January 1, 1970, 00:00:00 UTC = MJD 40587.0
UNIX_EPOCH_MJD = 40587.0


def parse_sdc_datetime(dt_str: str) -> datetime:
    """
    Parse internalSDC datetime string to UTC datetime.
    
    Format: "YYYY-MM-DD HH:MM:SS.ffffff" (space-separated, no timezone)
    Treat as UTC per TSI convention.
    """
    return datetime.strptime(dt_str, "%Y-%m-%d %H:%M:%S.%f").replace(tzinfo=timezone.utc)


def datetime_to_mjd(dt: datetime) -> float:
    """Convert UTC datetime to Modified Julian Date."""
    # Convert to Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
    unix_timestamp = dt.timestamp()
    # Convert to MJD: MJD = Unix_days + UNIX_EPOCH_MJD
    mjd = (unix_timestamp / 86400.0) + UNIX_EPOCH_MJD
    return mjd


def sdc_datetime_to_mjd(dt_str: str) -> float:
    """Parse internalSDC datetime string and convert to MJD."""
    dt = parse_sdc_datetime(dt_str)
    return datetime_to_mjd(dt)


def extract_constraints_from_task(task_data: dict) -> dict:
    """
    Extract constraints from ObservationTask.
    
    Focus on AirmassAltitude; use flexible defaults for azimuth.
    """
    constraint_obj = task_data.get("constraint", {})
    
    # Default: full range for azimuth, conservative for altitude
    constraints = {
        "min_alt": 0.0,
        "max_alt": 90.0,
        "min_az": 0.0,
        "max_az": 360.0
    }
    
    # Extract AirmassAltitude if present
    if "constraints::AirmassAltitude" in constraint_obj:
        airmass_data = constraint_obj["constraints::AirmassAltitude"]
        alt_range = airmass_data.get("range", {})
        constraints["min_alt"] = alt_range.get("first", 0.0)
        constraints["max_alt"] = alt_range.get("second", 90.0)
    
    return constraints


def extract_coordinates(task_data: dict) -> Tuple[float, float]:
    """Extract RA and Dec in degrees from ObservationTask."""
    target = task_data.get("target", {})
    
    if "coordinates::ConstEquatorial" in target:
        coords = target["coordinates::ConstEquatorial"]
        ra = coords.get("ra", 0.0)
        dec = coords.get("dec", 0.0)
        return ra, dec
    
    # Fallback: return zeros (should warn in production)
    return 0.0, 0.0


def parse_ra_dec_from_key(key_str: str) -> Optional[Tuple[float, float]]:
    """
    Parse RA and Dec from possiblePeriodsMap key string.
    
    Expected format: "AirmassAltitude(...) 2h14m17.939209s, 51º44'51.942372""
    Returns: (ra_deg, dec_deg) or None if parsing fails
    """
    # Pattern: HhMmS.s, ±D[º°]M'S.s" (accept both º and ° symbols)
    # Allow optional spaces between components (e.g., "1º 5'23.8")
    pattern = r"(\d+)h\s*(\d+)m\s*([\d.]+)s,\s*([+-]?\d+)[º°]\s*(\d+)'\s*([\d.]+)\""
    match = re.search(pattern, key_str)
    
    if not match:
        return None
    
    h, m, s, d, arcmin, arcsec = match.groups()
    
    # Convert HMS to degrees
    ra_deg = (float(h) * 15.0) + (float(m) * 0.25) + (float(s) * 0.25 / 60.0)
    
    # Convert DMS to degrees
    dec_deg = abs(float(d)) + (float(arcmin) / 60.0) + (float(arcsec) / 3600.0)
    if d.startswith('-'):
        dec_deg = -dec_deg
    
    return ra_deg, dec_deg


def match_key_to_blocks(key_str: str, blocks: List[dict], tolerance_deg: float = 0.01) -> List[str]:
    """
    Match possiblePeriodsMap key to all block original_block_ids with matching coordinates.
    
    Returns list of original_block_ids with RA/Dec within tolerance.
    """
    coords = parse_ra_dec_from_key(key_str)
    if coords is None:
        return []
    
    key_ra, key_dec = coords
    matched_blocks = []
    
    for block in blocks:
        block_ra = block['target_ra']
        block_dec = block['target_dec']
        
        # Simple angular distance (adequate for matching, not great-circle)
        distance = ((key_ra - block_ra) ** 2 + (key_dec - block_dec) ** 2) ** 0.5
        
        if distance <= tolerance_deg:
            matched_blocks.append(block['original_block_id'])
    
    return matched_blocks


def convert_weighted_period_to_mjd(weighted_period: dict) -> dict:
    """Convert a WeightedPeriod (with begin/end datetime strings) to MJD Period."""
    begin_str = weighted_period.get("begin", "")
    end_str = weighted_period.get("end", "")
    
    return {
        "start": sdc_datetime_to_mjd(begin_str),
        "stop": sdc_datetime_to_mjd(end_str)
    }


def convert_sdc_to_tsi(input_path: Path, output_dir: Path, schedule_name: Optional[str] = None):
    """
    Convert internalSDC JSON to TSI schedule.json and possible_periods.json.
    """
    print(f"Reading input file: {input_path}")
    with open(input_path, 'r') as f:
        sdc_data = json.load(f)
    
    # Extract scheduling blocks
    scheduling_blocks_raw = sdc_data.get("schedulingBlocks", [])
    scheduled_periods_map = sdc_data.get("output", {}).get("scheduledPeriods", {})
    possible_periods_nested = sdc_data.get("output", {}).get("possiblePeriodsMap", {})
    
    # Extract the actual map from nested structure
    possible_periods_map = possible_periods_nested.get("constraints::PossiblePeriodsMap", {}).get("possiblePeriodsMap_", {})
    
    # Build schedule blocks
    blocks = []
    
    for sb_entry in scheduling_blocks_raw:
        if "scheduling_blocks::ObservationTask" not in sb_entry:
            continue
        
        task = sb_entry["scheduling_blocks::ObservationTask"]
        block_id = task.get("id", "")
        
        # Extract data
        priority = task.get("priority", 1.0)
        estimated_duration = task.get("estimated_duration", 0.0)
        ra, dec = extract_coordinates(task)
        constraints = extract_constraints_from_task(task)
        
        # Build block - use UUID as original_block_id, no internal id
        block = {
            "original_block_id": block_id,  # User-provided identifier (UUID string)
            "priority": priority,
            "target_ra": ra,
            "target_dec": dec,
            "constraints": constraints,
            "min_observation": estimated_duration,  # Use estimated as both min and requested
            "requested_duration": estimated_duration
        }
        
        # Add scheduled_period if available
        if block_id in scheduled_periods_map:
            scheduled_entry = scheduled_periods_map[block_id]
            if "time::WeightedPeriod" in scheduled_entry:
                weighted = scheduled_entry["time::WeightedPeriod"]
                block["scheduled_period"] = convert_weighted_period_to_mjd(weighted)
        
        blocks.append(block)
    
    # Build schedule.json
    schedule = {
        "name": schedule_name or input_path.stem,
        "blocks": blocks
    }
    
    # Build possible_periods.json
    possible_periods = {"blocks": {}}
    
    for key_str, weighted_periods in possible_periods_map.items():
        # Match key to all blocks with same coordinates
        block_ids = match_key_to_blocks(key_str, blocks)
        
        if not block_ids:
            # Skip unmatched entries
            continue
        
        # Convert weighted periods to plain MJD periods
        periods = []
        for entry in weighted_periods:
            if "time::WeightedPeriod" in entry:
                weighted = entry["time::WeightedPeriod"]
                periods.append(convert_weighted_period_to_mjd(weighted))
        
        # Assign to all matching blocks
        for block_id in block_ids:
            if block_id not in possible_periods["blocks"]:
                possible_periods["blocks"][block_id] = []
            
            possible_periods["blocks"][block_id].extend(periods)
    
    # Write outputs
    output_dir.mkdir(parents=True, exist_ok=True)
    
    schedule_path = output_dir / "schedule.json"
    print(f"Writing schedule to: {schedule_path}")
    with open(schedule_path, 'w') as f:
        json.dump(schedule, f, indent=2)
    
    possible_periods_path = output_dir / "possible_periods.json"
    print(f"Writing possible periods to: {possible_periods_path}")
    with open(possible_periods_path, 'w') as f:
        json.dump(possible_periods, f, indent=2)
    
    print(f"\nConversion complete!")
    print(f"  Blocks converted: {len(blocks)}")
    print(f"  Blocks with scheduled periods: {sum(1 for b in blocks if 'scheduled_period' in b)}")
    print(f"  Blocks with possible periods: {len(possible_periods['blocks'])}")


def main():
    parser = argparse.ArgumentParser(
        description="Convert internalSDC scheduler output to TSI schedule format"
    )
    parser.add_argument(
        "input",
        type=Path,
        help="Input internalSDC JSON file"
    )
    parser.add_argument(
        "-o", "--output-dir",
        type=Path,
        default=Path("./output"),
        help="Output directory for schedule.json and possible_periods.json (default: ./output)"
    )
    parser.add_argument(
        "-n", "--name",
        type=str,
        help="Schedule name (default: input filename)"
    )
    
    args = parser.parse_args()
    
    convert_sdc_to_tsi(args.input, args.output_dir, args.name)


if __name__ == "__main__":
    main()
