#!/usr/bin/env python3
"""
Convert legacy scheduling JSON (e.g., AP.json) into schedule.schema.json format.

Usage:
  python3 scripts/convert_ap_schedule.py path/to/AP.json -o out.json
  python3 scripts/convert_ap_schedule.py data/sensitive/*.json -o out_dir
"""
import argparse
import json
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def find_blocks(src: Any) -> List[Dict[str, Any]]:
    if isinstance(src, dict):
        if "SchedulingBlock" in src and isinstance(src["SchedulingBlock"], list):
            return src["SchedulingBlock"]
        if "blocks" in src and isinstance(src["blocks"], list):
            return src["blocks"]
        for v in src.values():
            if isinstance(v, list):
                return v
        return []
    if isinstance(src, list):
        return src
    return []


def extract_mjd(val: Any) -> Optional[float]:
    if isinstance(val, dict):
        if "value" in val:
            return float(val["value"])
        if "startTime" in val and isinstance(val["startTime"], dict) and "value" in val["startTime"]:
            return float(val["startTime"]["value"])
        if "stopTime" in val and isinstance(val["stopTime"], dict) and "value" in val["stopTime"]:
            return float(val["stopTime"]["value"])
    if isinstance(val, (int, float)):
        return float(val)
    return None


def convert_period_legacy(p: Any) -> Optional[Dict[str, float]]:
    if isinstance(p, dict):
        if "startTime" in p and "stopTime" in p:
            s = extract_mjd(p["startTime"])
            e = extract_mjd(p["stopTime"])
            if s is not None and e is not None:
                return {"start": s, "stop": e}
        if "start" in p and "stop" in p:
            return {"start": float(p["start"]), "stop": float(p["stop"])}
    return None


def pick_first_coord(candidates: Iterable[Tuple[Optional[float], Optional[float]]]) -> Tuple[float, float]:
    for ra, dec in candidates:
        if ra is not None and dec is not None:
            return float(ra), float(dec)
    return 0.0, 0.0


def extract_target_coords(blk: Dict[str, Any]) -> Tuple[float, float]:
    target = blk.get("target") or {}
    pos = target.get("position_") or target.get("position") or {}
    celestial = (pos.get("coord") or {}).get("celestial") or {}
    t_ra = celestial.get("raInDeg") or celestial.get("ra_in_deg")
    t_dec = celestial.get("decInDeg") or celestial.get("dec_in_deg")

    cfg = blk.get("schedulingBlockConfiguration_") or blk.get("configuration") or {}
    omode = cfg.get("observingMode_") or cfg.get("observing_mode") or {}
    custom = omode.get("custom_") or omode.get("custom") or {}
    coords = custom.get("coordinates") or []
    c0 = coords[0] if isinstance(coords, list) and coords else {}
    ccel = c0.get("celestial") or {}
    c_ra = ccel.get("raInDeg")
    c_dec = ccel.get("decInDeg")

    wobble = omode.get("wobble_") or omode.get("wobble") or {}
    wcel = (wobble.get("centralCoordinate") or {}).get("celestial") or {}
    w_ra = wcel.get("raInDeg")
    w_dec = wcel.get("decInDeg")

    return pick_first_coord(
        [
            (t_ra, t_dec),
            (c_ra, c_dec),
            (w_ra, w_dec),
            (blk.get("target_ra") or blk.get("targetRa"), blk.get("target_dec") or blk.get("targetDec")),
        ]
    )


def extract_constraints(blk: Dict[str, Any]) -> Tuple[Dict[str, Any], Dict[str, Any]]:
    cfg = blk.get("schedulingBlockConfiguration_") or blk.get("configuration") or {}
    constr = cfg.get("constraints_") or cfg.get("constraints") or {}
    elev = constr.get("elevationConstraint_") or constr.get("elevation_constraint") or {}
    az = constr.get("azimuthConstraint_") or constr.get("azimuth_constraint") or {}
    timec = constr.get("timeConstraint_") or constr.get("time_constraint") or {}

    constraints: Dict[str, Any] = {
        "min_alt": float(elev.get("minElevationAngleInDeg") or elev.get("min_elevation_angle_in_deg") or 0.0),
        "max_alt": float(elev.get("maxElevationAngleInDeg") or elev.get("max_elevation_angle_in_deg") or 90.0),
        "min_az": float(az.get("minAzimuthAngleInDeg") or az.get("min_azimuth_angle_in_deg") or 0.0),
        "max_az": float(az.get("maxAzimuthAngleInDeg") or az.get("max_azimuth_angle_in_deg") or 360.0),
    }

    fst = timec.get("fixedStartTime") or timec.get("fixed_start_time")
    fstop = timec.get("fixedStopTime") or timec.get("fixed_stop_time")
    if isinstance(fst, list) and isinstance(fstop, list) and fst and fstop:
        s = extract_mjd(fst[0])
        e = extract_mjd(fstop[0])
        if s is not None and e is not None:
            constraints["fixed_time"] = {"start": s, "stop": e}

    return constraints, timec


def extract_visibility_periods(blk: Dict[str, Any]) -> Optional[List[Dict[str, float]]]:
    raw = blk.get("visibility_periods") or blk.get("visibilityPeriods")
    if not isinstance(raw, list):
        return None
    periods = []
    for p in raw:
        c = convert_period_legacy(p)
        if c:
            periods.append(c)
    return periods if periods else None


def convert_block(blk: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    bid = blk.get("schedulingBlockId") or blk.get("id")
    if bid is None:
        return None

    ra, dec = extract_target_coords(blk)
    constraints, timec = extract_constraints(blk)

    min_obs = timec.get("minObservationTimeInSec") or timec.get("min_observation_time_in_sec")
    req_dur = timec.get("requestedDurationSec") or timec.get("requested_duration_sec")
    if min_obs is None or req_dur is None:
        sched = blk.get("scheduled_period") or blk.get("scheduledPeriod")
        if isinstance(sched, dict) and "durationInSec" in sched:
            dur = float(sched["durationInSec"])
            min_obs = min_obs if min_obs is not None else dur
            req_dur = req_dur if req_dur is not None else dur

    out: Dict[str, Any] = {
        "id": int(bid),
        "priority": float(blk.get("priority") or 0.0),
        "target_ra": float(ra),
        "target_dec": float(dec),
        "constraints": constraints,
        "min_observation": float(min_obs or 0.0),
        "requested_duration": float(req_dur or 0.0),
    }

    original = blk.get("originalBlockId") or blk.get("original_block_id")
    if original is not None:
        out["original_block_id"] = original

    # Only add `scheduled_period` to output if the original block actually had it.
    if "scheduled_period" in blk or "scheduledPeriod" in blk:
        scheduled = blk.get("scheduled_period") or blk.get("scheduledPeriod")
        if isinstance(scheduled, dict):
            csp = convert_period_legacy(scheduled)
            if csp:
                out["scheduled_period"] = csp

    vis = extract_visibility_periods(blk)
    if vis is not None:
        out["visibility_periods"] = vis

    return out


def convert_schedule(src: Any, name: str) -> Dict[str, Any]:
    blocks = []
    for blk in find_blocks(src):
        if isinstance(blk, dict):
            converted = convert_block(blk)
            if converted:
                blocks.append(converted)

    return {
        "name": name,
        "blocks": blocks,
    }


def resolve_outputs(inputs: List[Path], output: Optional[Path]) -> List[Path]:
    if not inputs:
        raise ValueError("No input files provided.")
    if output is None:
        if len(inputs) > 1:
            raise ValueError("Output must be a directory when converting multiple inputs.")
        return [inputs[0].with_suffix(".schedule.json")]
    if len(inputs) == 1 and (not output.exists() or output.is_file()):
        return [output]
    if output.exists() and output.is_dir():
        return [output / f"{p.stem}.schedule.json" for p in inputs]
    if len(inputs) > 1:
        raise ValueError("Output must be a directory when converting multiple inputs.")
    return [output]


def main() -> int:
    parser = argparse.ArgumentParser(description="Convert AP-style schedules to schedule.schema.json.")
    parser.add_argument("inputs", nargs="+", help="Input JSON files.")
    parser.add_argument("-o", "--output", help="Output file or directory.")
    args = parser.parse_args()

    inputs = [Path(p) for p in args.inputs]
    output = Path(args.output) if args.output else None
    outputs = resolve_outputs(inputs, output)

    for in_path, out_path in zip(inputs, outputs):
        src = load_json(in_path)
        schedule = convert_schedule(src, in_path.stem)
        out_path.write_text(json.dumps(schedule, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
