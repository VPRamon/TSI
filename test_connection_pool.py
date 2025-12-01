#!/usr/bin/env python3
"""Test script to verify connection pool improvements."""

import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

try:
    from src.tsi.services.database import list_schedules_db
except ImportError:
    print("Failed to import list_schedules_db")
    sys.exit(1)


def test_single_call():
    """Test a single call to list_schedules_db."""
    print("Testing single call to list_schedules_db...")
    try:
        start = time.time()
        schedules = list_schedules_db()
        elapsed = time.time() - start
        print(f"✅ Success! Retrieved {len(schedules)} schedules in {elapsed:.2f}s")
        return True
    except Exception as e:
        print(f"❌ Failed: {e}")
        return False


def test_concurrent_calls(num_threads=10):
    """Test concurrent calls to list_schedules_db."""
    print(f"\nTesting {num_threads} concurrent calls to list_schedules_db...")
    
    def call_api(i):
        start = time.time()
        try:
            schedules = list_schedules_db()
            elapsed = time.time() - start
            return (i, True, len(schedules), elapsed, None)
        except Exception as e:
            elapsed = time.time() - start
            return (i, False, 0, elapsed, str(e))
    
    start_time = time.time()
    with ThreadPoolExecutor(max_workers=num_threads) as executor:
        futures = [executor.submit(call_api, i) for i in range(num_threads)]
        results = [f.result() for f in as_completed(futures)]
    
    total_time = time.time() - start_time
    
    successes = [r for r in results if r[1]]
    failures = [r for r in results if not r[1]]
    
    print(f"\n{'='*60}")
    print(f"Results for {num_threads} concurrent calls:")
    print(f"  ✅ Successful: {len(successes)}")
    print(f"  ❌ Failed: {len(failures)}")
    print(f"  ⏱️  Total time: {total_time:.2f}s")
    
    if successes:
        avg_time = sum(r[3] for r in successes) / len(successes)
        print(f"  📊 Average response time: {avg_time:.2f}s")
    
    if failures:
        print(f"\n❌ Failures:")
        for i, _, _, elapsed, error in failures:
            print(f"  Thread {i}: {error} (after {elapsed:.2f}s)")
    
    print(f"{'='*60}\n")
    
    return len(failures) == 0


if __name__ == "__main__":
    print("Connection Pool Test\n" + "="*60 + "\n")
    
    # Test single call
    single_success = test_single_call()
    
    if not single_success:
        print("\n⚠️  Single call failed, skipping concurrent test")
        sys.exit(1)
    
    # Test concurrent calls
    concurrent_success = test_concurrent_calls(10)
    
    if concurrent_success:
        print("✅ All tests passed! Connection pool is working correctly.")
        sys.exit(0)
    else:
        print("❌ Some concurrent calls failed. Check the output above.")
        sys.exit(1)
