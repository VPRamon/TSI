"""Test to verify validation logic for time constraints."""

# Test case from the JSON: Block 1000004990
# scheduled_period: start=61894.19429606479, stop=61894.20818495378
# durationInSec: 1200.0
# requestedDurationSec: 1200
# fixedStartTime: [] (empty - no constraint)
# fixedStopTime: [] (empty - no constraint)

# Calculate durations
scheduled_start_mjd = 61894.19429606479
scheduled_stop_mjd = 61894.20818495378
requested_duration_sec = 1200

# Scheduled duration
scheduled_duration_days = scheduled_stop_mjd - scheduled_start_mjd
scheduled_duration_hours = scheduled_duration_days * 24.0
scheduled_duration_sec = scheduled_duration_hours * 3600

# Requested duration
requested_hours = requested_duration_sec / 3600.0

print("=== Scheduled Period Validation ===")
print(f"Scheduled start: {scheduled_start_mjd} MJD")
print(f"Scheduled stop: {scheduled_stop_mjd} MJD")
print(f"Scheduled duration: {scheduled_duration_days:.10f} days = {scheduled_duration_hours:.6f} hours = {scheduled_duration_sec:.2f} seconds")
print(f"Requested duration: {requested_duration_sec} seconds = {requested_hours:.6f} hours")
print()

# Check 16: Scheduled duration exceeds requested duration
tolerance = 1.01
if scheduled_duration_hours > requested_hours * tolerance:
    print(f"❌ ISSUE: Scheduled duration ({scheduled_duration_hours:.6f}h) exceeds requested ({requested_hours:.6f}h * {tolerance})")
else:
    print(f"✓ OK: Scheduled duration ({scheduled_duration_hours:.6f}h) <= requested ({requested_hours:.6f}h * {tolerance})")

difference_sec = scheduled_duration_sec - requested_duration_sec
print(f"   Difference: {difference_sec:.6f} seconds ({difference_sec / requested_duration_sec * 100:.4f}%)")
print()

# Check 17: Scheduled period outside time constraint
# Since fixedStartTime and fixedStopTime are empty, constraint should be None
print("=== Time Constraint Validation ===")
print("Fixed time constraints: None (empty arrays in JSON)")
print("✓ No constraint check should be performed")
print()

# Now let's test with a hypothetical constraint that would fail
print("=== Hypothetical Constraint Test ===")
# If there WAS a constraint like: 61894.0 to 61894.15
constraint_start = 61894.0
constraint_stop = 61894.15

print(f"Hypothetical constraint: [{constraint_start}, {constraint_stop}] MJD")
print(f"Scheduled period: [{scheduled_start_mjd}, {scheduled_stop_mjd}] MJD")

if scheduled_start_mjd < constraint_start or scheduled_stop_mjd > constraint_stop:
    print(f"❌ Would fail: Scheduled period is outside constraint")
    if scheduled_start_mjd < constraint_start:
        print(f"   Start {scheduled_start_mjd} < constraint start {constraint_start}")
    if scheduled_stop_mjd > constraint_stop:
        print(f"   Stop {scheduled_stop_mjd} > constraint stop {constraint_stop}")
else:
    print(f"✓ Would pass: Scheduled period is within constraint")
print()

# Check 14: Time constraint duration less than requested duration
constraint_duration_days = constraint_stop - constraint_start
constraint_duration_hours = constraint_duration_days * 24.0

print("=== Constraint Duration Test ===")
print(f"Constraint duration: {constraint_duration_hours:.6f} hours")
print(f"Requested duration: {requested_hours:.6f} hours")

if constraint_duration_hours < requested_hours:
    print(f"❌ Would fail: Constraint ({constraint_duration_hours:.2f}h) < requested ({requested_hours:.2f}h)")
else:
    print(f"✓ Would pass: Constraint ({constraint_duration_hours:.2f}h) >= requested ({requested_hours:.2f}h)")
