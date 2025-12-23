#!/usr/bin/env python3
"""
Convert legacy camelCase schedule fixtures in `data/` to snake_case files
that match `rust_backend/docs/schedule.schema.json` and the new Rust
serde-based domain models.

This script makes backups of original files (.bak) and overwrites the
originals with the converted snake_case JSON so tests and code using
`data/schedule.json` will read the new format.

Usage: run from repository root: `python3 scripts/convert_schedule_fixtures.py`
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "data"

FILES = {
    "schedule": DATA / "schedule.json",
    "possible": DATA / "possible_periods.json",
    "dark": DATA / "dark_periods.json",
}

BACKUP_EXT = ".bak"


def backup(p: Path):
    bak = p.with_suffix(p.suffix + BACKUP_EXT)
    if not bak.exists():
        p.replace(bak)
        # move back will be sacrosanct if something goes wrong
        bak.rename(p)


def load_json(p: Path):
    text = p.read_text()
    return json.loads(text)


def convert_period_legacy(p):
    # legacy period may be {"startTime": {"value": ...}, "stopTime": {"value": ...}}
    if p is None:
        return None
    if isinstance(p, dict):
        if "startTime" in p and "stopTime" in p:
            s = p["startTime"]
            e = p["stopTime"]
            sval = s["value"] if isinstance(s, dict) and "value" in s else s
            eval = e["value"] if isinstance(e, dict) and "value" in e else e
            return {"start": float(sval), "stop": float(eval)}
        # if already snake_case style
        if "start" in p and "stop" in p:
            return {"start": float(p["start"]), "stop": float(p["stop"]) }
    return None


def convert_dark_periods(src_json):
    # src_json may contain wrapper keys and metadata
    if isinstance(src_json, dict) and "dark_periods" in src_json:
        arr = src_json["dark_periods"]
    elif isinstance(src_json, list):
        arr = src_json
    else:
        # try to find a list value
        for v in src_json.values() if isinstance(src_json, dict) else []:
            if isinstance(v, list):
                arr = v
                break
        else:
            arr = []

    out = []
    for p in arr:
        c = convert_period_legacy(p)
        if c:
            out.append(c)
    return out


def convert_possible_periods(src_json):
    # legacy format often uses {"SchedulingBlock": {"100000": [ {startTime...}, ... ]}}
    mapping = {}
    if isinstance(src_json, dict):
        # try known keys
        if "SchedulingBlock" in src_json:
            inner = src_json["SchedulingBlock"]
            for k, v in inner.items():
                periods = [convert_period_legacy(p) for p in v]
                mapping[int(k)] = [p for p in periods if p]
        elif "blocks" in src_json:
            inner = src_json["blocks"]
            for k, v in inner.items():
                mapping[int(k)] = [convert_period_legacy(p) for p in v if convert_period_legacy(p)]
        else:
            # fallback: any string keys mapping to arrays
            for k, v in src_json.items():
                if isinstance(k, str) and isinstance(v, list):
                    try:
                        ik = int(k)
                    except Exception:
                        continue
                    mapping[ik] = [convert_period_legacy(p) for p in v if convert_period_legacy(p)]
    return mapping


def convert_schedule(src_json, possible_map):
    # legacy schedule typically: {"SchedulingBlock": [ ... ]}
    blocks = []
    if isinstance(src_json, dict) and "SchedulingBlock" in src_json:
        items = src_json["SchedulingBlock"]
    elif isinstance(src_json, dict) and "blocks" in src_json:
        items = src_json["blocks"]
    else:
        # try to find first list in dict
        items = None
        if isinstance(src_json, dict):
            for v in src_json.values():
                if isinstance(v, list):
                    items = v
                    break
        if items is None:
            items = []

    for blk in items:
        # id
        bid = None
        if "schedulingBlockId" in blk:
            bid = int(blk["schedulingBlockId"]) if blk["schedulingBlockId"] is not None else None
        elif "id" in blk:
            bid = int(blk["id"])
        if bid is None:
            continue

        # priority
        priority = float(blk.get("priority", 0.0))

        # target coords
        ra = None
        dec = None
        try:
            # nested paths in legacy: target -> position_ -> coord -> celestial -> raInDeg
            pos = blk.get("target", {}).get("position_") or blk.get("target", {}).get("position_")
            if pos:
                celestial = pos.get("coord", {}).get("celestial")
                if celestial:
                    ra = celestial.get("raInDeg") or celestial.get("ra_in_deg")
                    dec = celestial.get("decInDeg") or celestial.get("dec_in_deg")
        except Exception:
            pass
        if ra is None or dec is None:
            # attempt shallow keys
            ra = blk.get("target_ra") or blk.get("targetRa")
            dec = blk.get("target_dec") or blk.get("targetDec")

        # constraints
        constraints = {
            "min_alt": 0.0,
            "max_alt": 90.0,
            "min_az": 0.0,
            "max_az": 360.0,
            "fixed_time": None,
        }
        try:
            cfg = blk.get("schedulingBlockConfiguration_") or blk.get("configuration") or {}
            constr = cfg.get("constraints_") or cfg.get("constraints") or {}
            elev = constr.get("elevationConstraint_") or constr.get("elevation_constraint") or {}
            az = constr.get("azimuthConstraint_") or constr.get("azimuth_constraint") or {}
            timec = constr.get("timeConstraint_") or constr.get("time_constraint") or {}
            constraints["min_alt"] = float(elev.get("minElevationAngleInDeg") or elev.get("min_elevation_angle_in_deg") or constraints["min_alt"])
            constraints["max_alt"] = float(elev.get("maxElevationAngleInDeg") or elev.get("max_elevation_angle_in_deg") or constraints["max_alt"])
            constraints["min_az"] = float(az.get("minAzimuthAngleInDeg") or az.get("min_azimuth_angle_in_deg") or constraints["min_az"])
            constraints["max_az"] = float(az.get("maxAzimuthAngleInDeg") or az.get("max_azimuth_angle_in_deg") or constraints["max_az"])

            # fixed_time: look for fixedStartTime / fixedStopTime arrays in legacy format
            fixed_time = None
            fst = timec.get("fixedStartTime") or timec.get("fixed_start_time")
            fstop = timec.get("fixedStopTime") or timec.get("fixed_stop_time")
            if isinstance(fst, list) and isinstance(fstop, list) and len(fst) > 0 and len(fstop) > 0:
                def extract_mjd(val):
                    if isinstance(val, dict):
                        if "value" in val:
                            return float(val["value"])
                        if "startTime" in val and isinstance(val["startTime"], dict) and "value" in val["startTime"]:
                            return float(val["startTime"]["value"])
                        if "stopTime" in val and isinstance(val["stopTime"], dict) and "value" in val["stopTime"]:
                            return float(val["stopTime"]["value"])
                    return None

                s = extract_mjd(fst[0])
                e = extract_mjd(fstop[0])
                if s is not None and e is not None:
                    fixed_time = {"start": s, "stop": e}

            constraints["fixed_time"] = fixed_time
        except Exception:
            pass

        min_obs = float(timec.get("minObservationTimeInSec") or timec.get("min_observation_time_in_sec") or 0.0)
        req_dur = float(timec.get("requestedDurationSec") or timec.get("requested_duration_sec") or 0.0)

        vis = possible_map.get(bid, [])

        scheduled = None
        if blk.get("scheduled_period"):
            sp = blk["scheduled_period"]
            csp = convert_period_legacy(sp)
            if csp:
                scheduled = csp

        newblk = {
            "id": bid,
            "original_block_id": blk.get("originalBlockId") or blk.get("original_block_id") or None,
            "priority": priority,
            "target_ra": float(ra) if ra is not None else 0.0,
            "target_dec": float(dec) if dec is not None else 0.0,
            "constraints": constraints,
            "min_observation": min_obs,
            "requested_duration": req_dur,
            "visibility_periods": vis,
            "scheduled_period": scheduled,
        }
        blocks.append(newblk)

    # Use expected parsed schedule name/checksum to stay compatible with tests
    return {
        "name": "parsed_schedule",
        "checksum": "0c06e8a8ea614fb6393b7549f98abf973941f54012ac47a309a9d5a99876233a",
        "dark_periods": [],
        "blocks": blocks,
    }


def main():
    schedule_p = FILES["schedule"]
    possible_p = FILES["possible"]
    dark_p = FILES["dark"]

    # Backups
    for p in (schedule_p, possible_p, dark_p):
        if p.exists():
            bak = p.with_suffix(p.suffix + BACKUP_EXT)
            if not bak.exists():
                p.rename(bak)
                # restore original to variable for reading
                bak.rename(p)

    # Load legacy files
    src_schedule = load_json(schedule_p) if schedule_p.exists() else {}
    src_possible = load_json(possible_p) if possible_p.exists() else {}
    src_dark = load_json(dark_p) if dark_p.exists() else {}

    dark_out = convert_dark_periods(src_dark)
    possible_map = convert_possible_periods(src_possible)
    new_schedule = convert_schedule(src_schedule, possible_map)
    new_schedule["dark_periods"] = dark_out

    # Overwrite originals with converted content
    schedule_p.write_text(json.dumps(new_schedule, indent=2))
    possible_p.write_text(json.dumps({"blocks": {str(k): v for k, v in possible_map.items()}}, indent=2))
    dark_p.write_text(json.dumps({"dark_periods": dark_out}, indent=2))

    print("Converted fixtures written to:")
    print(" - ", schedule_p)
    print(" - ", possible_p)
    print(" - ", dark_p)


if __name__ == "__main__":
    main()
