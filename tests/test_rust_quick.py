#!/usr/bin/env python3
"""
Quick test of the Rust backend - FASE 1A
"""
import sys
sys.path.insert(0, '/tmp/tsi_test')

import tsi_rust
import time
from datetime import datetime, timezone

print("=" * 70)
print("🦀 TSI Rust Backend - FASE 1A Tests")
print("=" * 70)

# Test 1: MJD to datetime
print("\n✅ Test 1: MJD to datetime conversion")
mjd = 59580.0
dt = tsi_rust.mjd_to_datetime(mjd)
print(f"   MJD {mjd} → {dt}")
assert dt.year == 2022, f"Expected year 2022, got {dt.year}"
print("   ✓ Passed")

# Test 2: Datetime to MJD
print("\n✅ Test 2: Datetime to MJD conversion")
test_dt = datetime(2022, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
result_mjd = tsi_rust.datetime_to_mjd(test_dt)
print(f"   {test_dt} → MJD {result_mjd}")
assert abs(result_mjd - 59580.0) < 1.0, f"Expected ~59580.0, got {result_mjd}"
print("   ✓ Passed")

# Test 3: Roundtrip
print("\n✅ Test 3: Roundtrip MJD → datetime → MJD")
original_mjd = 59580.123456
dt_temp = tsi_rust.mjd_to_datetime(original_mjd)
back_to_mjd = tsi_rust.datetime_to_mjd(dt_temp)
error = abs(original_mjd - back_to_mjd)
print(f"   Original: {original_mjd}")
print(f"   Roundtrip: {back_to_mjd}")
print(f"   Error: {error:.10f}")
assert error < 1e-6, f"Roundtrip error too large: {error}"
print("   ✓ Passed")

# Test 4: Parse empty visibility
print("\n✅ Test 4: Parse empty visibility periods")
result = tsi_rust.parse_visibility_periods("[]")
assert len(result) == 0, f"Expected 0 periods, got {len(result)}"
print("   ✓ Passed")

# Test 5: Parse single visibility period
print("\n✅ Test 5: Parse single visibility period")
input_str = "[(59580.0, 59581.0)]"
result = tsi_rust.parse_visibility_periods(input_str)
assert len(result) == 1, f"Expected 1 period, got {len(result)}"
start, stop = result[0]
print(f"   Parsed: {start} → {stop}")
assert start.year == 2022, f"Expected year 2022, got {start.year}"
print("   ✓ Passed")

# Test 6: Parse multiple visibility periods
print("\n✅ Test 6: Parse multiple visibility periods")
input_str = "[(59580.0, 59580.5), (59581.0, 59581.25), (59582.0, 59582.1)]"
result = tsi_rust.parse_visibility_periods(input_str)
assert len(result) == 3, f"Expected 3 periods, got {len(result)}"
print(f"   Parsed {len(result)} periods:")
for i, (start, stop) in enumerate(result, 1):
    duration_hours = (stop - start).total_seconds() / 3600
    print(f"     {i}. {start} → {stop} ({duration_hours:.1f}h)")
print("   ✓ Passed")

# Test 7: Performance - batch MJD conversions
print("\n✅ Test 7: Performance - Batch MJD conversions")
n = 10000
mjd_values = [59580.0 + i * 0.01 for i in range(n)]

start_time = time.time()
for mjd_val in mjd_values:
    _ = tsi_rust.mjd_to_datetime(mjd_val)
elapsed = time.time() - start_time

conversions_per_sec = n / elapsed if elapsed > 0 else float('inf')
print(f"   Converted {n:,} MJD values in {elapsed:.3f}s")
print(f"   Performance: {conversions_per_sec:,.0f} conversions/sec")
assert elapsed < 1.0, f"Too slow: {elapsed:.3f}s for {n} conversions"
print("   ✓ Passed")

# Test 8: Performance - batch visibility parsing
print("\n✅ Test 8: Performance - Batch visibility parsing")
n_parse = 1000
vis_strings = ["[(59580.0, 59580.5), (59581.0, 59581.25)]"] * n_parse

start_time = time.time()
for vis_str in vis_strings:
    _ = tsi_rust.parse_visibility_periods(vis_str)
elapsed = time.time() - start_time

parses_per_sec = n_parse / elapsed if elapsed > 0 else float('inf')
print(f"   Parsed {n_parse:,} visibility strings in {elapsed:.3f}s")
print(f"   Performance: {parses_per_sec:,.0f} parses/sec")
print("   ✓ Passed")

print("\n" + "=" * 70)
print("🎉 All tests passed successfully!")
print("=" * 70)
print("\n📊 FASE 1A Summary:")
print("  ✅ MJD conversions working")
print("  ✅ Visibility parsing working")
print("  ✅ Performance acceptable")
print("  ✅ Ready for integration with Python codebase")
