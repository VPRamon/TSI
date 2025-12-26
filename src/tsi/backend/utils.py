"""Small helpers: Python-side implementations of legacy filter functions.

These replace the previous Rust-exported `py_filter_by_range` and
`py_filter_by_scheduled` functions and operate on JSON record lists.
"""
from __future__ import annotations

import json
from typing import Any


def py_filter_by_range(json_str: str, column: str, min_val: float, max_val: float) -> str:
    records = json.loads(json_str)
    filtered = []
    for r in records:
        v = r.get(column)
        try:
            fv = float(v) if v is not None else None
        except (TypeError, ValueError):
            fv = None
        if fv is not None and min_val <= fv <= max_val:
            filtered.append(r)
    return json.dumps(filtered)


def py_filter_by_scheduled(json_str: str, filter_type: str) -> str:
    records = json.loads(json_str)
    if filter_type == "All":
        filtered = records
    elif filter_type == "Scheduled":
        filtered = [r for r in records if r.get("wasScheduled") is True]
    elif filter_type == "Unscheduled":
        filtered = [r for r in records if not r.get("wasScheduled", False)]
    else:
        filtered = records
    return json.dumps(filtered)


def py_filter_dataframe(
    json_str: str,
    priority_min: float,
    priority_max: float,
    scheduled_filter: str,
    priority_bins: list[str] | None = None,
    block_ids: list[str] | None = None,
) -> str:
    records = json.loads(json_str)

    # Priority range
    filtered = []
    for r in records:
        try:
            p = float(r.get("priority")) if r.get("priority") is not None else None
        except (TypeError, ValueError):
            p = None
        if p is not None and priority_min <= p <= priority_max:
            filtered.append(r)

    # Scheduled filter
    if scheduled_filter == "Scheduled":
        filtered = [r for r in filtered if r.get("wasScheduled") is True]
    elif scheduled_filter == "Unscheduled":
        filtered = [r for r in filtered if not r.get("wasScheduled", False)]

    # Priority bins
    if priority_bins:
        filtered = [r for r in filtered if r.get("priorityBin") in priority_bins]

    # Block IDs
    if block_ids:
        filtered = [r for r in filtered if r.get("schedulingBlockId") in block_ids]

    return json.dumps(filtered)


def py_remove_duplicates(json_str: str, subset: list[str] | None = None, keep: str | None = "first") -> str:
    records = json.loads(json_str)
    keep = keep or "first"

    def key_of(r: dict[str, Any]) -> str:
        if subset:
            parts = [str(r.get(c)) for c in subset]
            return "|".join(parts)
        return json.dumps(r, sort_keys=True)

    if keep == "first":
        seen = set()
        out = []
        for r in records:
            k = key_of(r)
            if k not in seen:
                seen.add(k)
                out.append(r)
        return json.dumps(out)

    if keep == "last":
        seen = set()
        out_rev = []
        for r in reversed(records):
            k = key_of(r)
            if k not in seen:
                seen.add(k)
                out_rev.append(r)
        return json.dumps(list(reversed(out_rev)))

    # keep == "none"
    counts: dict[str, int] = {}
    for r in records:
        counts[key_of(r)] = counts.get(key_of(r), 0) + 1
    out = [r for r in records if counts[key_of(r)] == 1]
    return json.dumps(out)


def py_remove_missing_coordinates(json_str: str) -> str:
    records = json.loads(json_str)
    filtered = []
    for r in records:
        try:
            has_ra = r.get("raDeg") is not None and float(r.get("raDeg")) is not None
            has_dec = r.get("decDeg") is not None and float(r.get("decDeg")) is not None
        except (TypeError, ValueError):
            has_ra = False
            has_dec = False
        if has_ra and has_dec:
            filtered.append(r)
    return json.dumps(filtered)


def py_validate_dataframe(json_str: str) -> tuple[bool, list[str]]:
    records = json.loads(json_str)
    issues: list[str] = []

    missing_coords = 0
    for r in records:
        try:
            has_ra = r.get("raDeg") is not None and float(r.get("raDeg")) is not None
            has_dec = r.get("decDeg") is not None and float(r.get("decDeg")) is not None
        except (TypeError, ValueError):
            has_ra = False
            has_dec = False
        if not (has_ra and has_dec):
            missing_coords += 1

    if missing_coords > 0:
        issues.append(f"{missing_coords} observations with missing coordinates")

    invalid_priorities = 0
    for r in records:
        try:
            p = float(r.get("priority"))
            if p < 0.0 or p > 100.0:
                invalid_priorities += 1
        except (TypeError, ValueError):
            invalid_priorities += 1

    if invalid_priorities > 0:
        issues.append(f"{invalid_priorities} observations with invalid priorities")

    return (len(issues) == 0, issues)
