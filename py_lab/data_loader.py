"""
Data Loader Module for py_lab

Simplified loader using direct JSON parsing with pandas.
Analytics use the backend (see analytics.py).
"""

import json
from pathlib import Path
from typing import Optional, Dict, Any, List
import pandas as pd


class ScheduleLoader:
    """
    Loader for schedule JSON files.
    
    Parses JSON directly into DataFrames for Python analysis.
    For advanced analytics, use the analytics module which leverages the Rust backend.
    """
    
    def __init__(self, schedule_path: Optional[Path] = None, possible_periods_path: Optional[Path] = None):
        """
        Initialize the schedule loader with explicit file paths.

        Args:
            schedule_path: Path to the schedule JSON file. Defaults to workspace/data/schedule.json
            possible_periods_path: Path to possible_periods JSON file. Defaults to workspace/data/possible_periods.json
        """
        base_data = Path(__file__).parent.parent / "data"
        self.schedule_path = Path(schedule_path) if schedule_path is not None else base_data / "schedule.json"
        self.possible_periods_path = Path(possible_periods_path) if possible_periods_path is not None else base_data / "possible_periods.json"

        if not self.schedule_path.exists():
            raise FileNotFoundError(f"Schedule file not found: {self.schedule_path}")
    
    def load_schedule(self, validate: bool = True) -> pd.DataFrame:
        """
        Load the configured schedule file into a DataFrame.

        Args:
            validate: Whether to perform basic validation

        Returns:
            DataFrame with scheduling blocks
        """
        schedule_path = self.schedule_path
        visibility_path = self.possible_periods_path if self.possible_periods_path.exists() else None

        # Try to use the Rust backend if available, otherwise fall back to JSON parsing
        try:
            try:
                import tsi_rust_api as tsi  # type: ignore
            except ImportError:
                import tsi_rust as tsi  # type: ignore

            schedule_json = schedule_path.read_text()
            visibility_json = visibility_path.read_text() if visibility_path is not None else None

            schedule_id = tsi.store_schedule(schedule_path.name, schedule_json, visibility_json)
            # `get_schedule` should return an object with a `blocks` attribute
            sched = tsi.get_schedule(tsi.ScheduleId(int(schedule_id)))

            def unwrap(val):
                if val is None:
                    return None
                if hasattr(val, "value") and callable(getattr(val, "value")):
                    return val.value()
                if hasattr(val, "start_mjd") or hasattr(val, "stop_mjd"):
                    return {
                        "start_mjd": getattr(val, "start_mjd", None),
                        "stop_mjd": getattr(val, "stop_mjd", None),
                    }
                return val

            blocks_out = []
            for b in getattr(sched, "blocks", []):
                bb = {}
                for fld in (
                    "id",
                    "original_block_id",
                    "priority",
                    "min_observation",
                    "requested_duration",
                    "target_ra",
                    "target_dec",
                    "scheduled_period",
                    "visibility_periods",
                ):
                    v = getattr(b, fld, None)
                    if fld == "visibility_periods":
                        bb["visibility_periods"] = [unwrap(p) for p in (v or [])]
                    else:
                        bb[fld] = unwrap(v)

                blocks_out.append(bb)

            df = pd.DataFrame(blocks_out)
            return df
        except Exception:
            # fall through to JSON parsing
            pass

        with open(schedule_path, "r") as f:
            data = json.load(f)

        blocks = data.get("blocks", [])
        if validate and not blocks:
            raise ValueError("No blocks found in schedule JSON")

        df = pd.DataFrame(blocks)

        # Ensure an `id` column exists (prefer `id`, then `original_block_id`, then index)
        if 'id' not in df.columns:
            if 'original_block_id' in df.columns:
                df['id'] = df['original_block_id']
            else:
                df['id'] = df.index.astype(str)

        # Ensure `visibility_periods` column exists. Prefer external possible_periods file
        if 'visibility_periods' not in df.columns:
            vis_map: Dict[str, Any] = {}
            if visibility_path is not None and visibility_path.exists():
                try:
                    with open(visibility_path, 'r') as vf:
                        vis_data = json.load(vf)

                    # possible_periods.json may be a dict with a 'blocks' mapping
                    if isinstance(vis_data, dict) and 'blocks' in vis_data:
                        vis_map = vis_data['blocks']
                    elif isinstance(vis_data, dict):
                        vis_map = vis_data
                    elif isinstance(vis_data, list):
                        # convert list of records to a mapping if necessary
                        for item in vis_data:
                            key = item.get('original_block_id') or item.get('id')
                            if key:
                                vis_map[str(key)] = item.get('visibility_periods') or item.get('periods') or []
                except Exception:
                    vis_map = {}

            def _lookup_vis(row: Dict[str, Any]) -> List[Any]:
                key = None
                if isinstance(row, dict):
                    key = row.get('original_block_id') or row.get('id')
                else:
                    # pandas Series
                    key = row.get('original_block_id') if 'original_block_id' in row else row.get('id')

                return vis_map.get(str(key), []) if key is not None else []

            df['visibility_periods'] = df.apply(_lookup_vis, axis=1)
        else:
            # Normalize non-list entries to empty lists
            df['visibility_periods'] = df['visibility_periods'].apply(
                lambda x: x if isinstance(x, list) else (x if pd.isna(x) is False and x is not None else [])
            )

        return df
    
    def list_available_schedules(self) -> List[str]:
        """
        List all JSON files in the data directory.
        
        Returns:
            List of JSON filenames
        """
        data_dir = self.schedule_path.parent
        return sorted([f.name for f in data_dir.glob("*.json")])


def load_schedule(
    schedule_path: Optional[Path] = None,
    possible_periods_path: Optional[Path] = None,
    validate: bool = True,
) -> pd.DataFrame:
    """
    Convenience function to load a schedule in one call.
    
    Args:
        schedule_file: Name of the schedule JSON file
        visibility_file: Optional separate visibility periods JSON file
        data_dir: Path to data directory (defaults to workspace/data/)
        validate: Whether to perform basic validation
        
    Returns:
        DataFrame with scheduling blocks
        
    Example:
        >>> df = load_schedule("schedule.json")
        >>> print(f"Loaded {len(df)} scheduling blocks")
        >>> print(df[['id', 'priority', 'target_ra', 'target_dec']].head())
    """
    loader = ScheduleLoader(schedule_path=schedule_path, possible_periods_path=possible_periods_path)
    return loader.load_schedule(validate=validate)
