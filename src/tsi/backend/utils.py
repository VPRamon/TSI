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
