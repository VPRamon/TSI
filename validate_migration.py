#!/usr/bin/env python
"""
Validation script for the new consolidated backend API.

This script demonstrates that the migration from tsi.backend to tsi_rust_api
is complete and working correctly.
"""

import sys
from pathlib import Path


def validate_imports():
    """Validate that new imports work and old imports fail."""
    print("1. Testing imports...")
    
    # Test new imports work
    try:
        from tsi_rust_api import (
            TSIBackend,
            load_schedule,
            load_schedule_file,
            get_top_observations,
            find_conflicts,
            filter_by_priority,
        )
        print("   ✓ tsi_rust_api imports successful")
    except ImportError as e:
        print(f"   ✗ Failed to import from tsi_rust_api: {e}")
        return False
    
    # Test old imports fail
    try:
        from tsi.backend import TSIBackend as OldBackend  # noqa: F401
        print("   ✗ Old tsi.backend still accessible (should be removed!)")
        return False
    except ImportError:
        print("   ✓ Old tsi.backend correctly removed")
    
    # Test services integration
    try:
        from tsi.services import BACKEND
        from tsi.services.rust_backend import load_schedule_from_any
        print("   ✓ Services layer imports successful")
    except ImportError as e:
        print(f"   ✗ Failed to import from services: {e}")
        return False
    
    return True


def validate_backend_class():
    """Validate TSIBackend class functionality."""
    print("\n2. Testing TSIBackend class...")
    
    from tsi_rust_api import TSIBackend
    
    # Instantiate
    backend = TSIBackend()
    print(f"   ✓ Backend instantiated: {repr(backend)}")
    
    # Check methods exist
    required_methods = [
        'load_schedule',
        'load_schedule_from_string',
        'filter_by_priority',
        'filter_by_scheduled',
        'get_top_observations',
        'find_conflicts',
        'validate_dataframe',
    ]
    
    for method in required_methods:
        if not hasattr(backend, method):
            print(f"   ✗ Missing method: {method}")
            return False
    
    print(f"   ✓ All {len(required_methods)} required methods present")
    return True


def validate_functional_api():
    """Validate functional API."""
    print("\n3. Testing functional API...")
    
    import tsi_rust_api
    
    required_functions = [
        'load_schedule',
        'load_schedule_file',
        'load_schedule_from_string',
        'load_dark_periods',
        'filter_by_priority',
        'get_top_observations',
        'find_conflicts',
    ]
    
    for func in required_functions:
        if not hasattr(tsi_rust_api, func):
            print(f"   ✗ Missing function: {func}")
            return False
    
    print(f"   ✓ All {len(required_functions)} functions available")
    return True


def validate_services_layer():
    """Validate services layer integration."""
    print("\n4. Testing services layer...")
    
    from tsi.services import BACKEND
    from tsi.services.rust_backend import load_schedule_from_any
    
    # Check BACKEND type
    backend_type = type(BACKEND).__name__
    if backend_type != "TSIBackend":
        print(f"   ✗ BACKEND has wrong type: {backend_type}")
        return False
    
    print(f"   ✓ BACKEND is TSIBackend instance")
    print(f"   ✓ load_schedule_from_any available")
    
    return True


def validate_no_legacy_code():
    """Validate that legacy backend code is removed."""
    print("\n5. Checking legacy code removal...")
    
    backend_path = Path("src/tsi/backend")
    if backend_path.exists():
        print(f"   ✗ Legacy backend directory still exists: {backend_path}")
        return False
    
    print("   ✓ Legacy backend directory removed")
    
    # Check for any remaining imports in source code
    import subprocess
    result = subprocess.run(
        ["grep", "-r", "from tsi.backend", "src/tsi", "--exclude-dir=__pycache__"],
        capture_output=True,
        text=True,
    )
    
    if result.returncode == 0:  # Found matches
        print("   ✗ Found references to tsi.backend in source code:")
        print(result.stdout)
        return False
    
    print("   ✓ No tsi.backend imports in source code")
    return True


def validate_data_operations():
    """Validate basic data operations work."""
    print("\n6. Testing data operations...")
    
    import json
    import tempfile
    import pandas as pd
    from tsi_rust_api import TSIBackend
    
    # Create test data
    test_data = {
        "schedulingBlocks": [
            {"schedulingBlockId": "1", "priority": 8.5, "raDeg": 10.0, "decDeg": 20.0},
            {"schedulingBlockId": "2", "priority": 6.2, "raDeg": 15.0, "decDeg": 25.0},
            {"schedulingBlockId": "3", "priority": 9.1, "raDeg": 20.0, "decDeg": 30.0},
        ]
    }
    
    # Test loading from file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(test_data, f)
        temp_path = f.name
    
    try:
        backend = TSIBackend()
        df = backend.load_schedule(temp_path)
        
        if len(df) != 3:
            print(f"   ✗ Expected 3 rows, got {len(df)}")
            return False
        print("   ✓ Load from file works")
        
        # Test filtering
        high_priority = backend.filter_by_priority(df, min_priority=8.0)
        if len(high_priority) != 2:
            print(f"   ✗ Expected 2 high priority rows, got {len(high_priority)}")
            return False
        print("   ✓ Priority filtering works")
        
        # Test top observations
        top_2 = backend.get_top_observations(df, n=2, by="priority")
        if len(top_2) != 2:
            print(f"   ✗ Expected 2 top observations, got {len(top_2)}")
            return False
        if top_2.iloc[0]["priority"] != 9.1:
            print(f"   ✗ Top observation should have priority 9.1, got {top_2.iloc[0]['priority']}")
            return False
        print("   ✓ Top observations works")
        
    finally:
        Path(temp_path).unlink()
    
    return True


def main():
    """Run all validation checks."""
    print("=" * 60)
    print("TSI Backend Migration Validation")
    print("=" * 60)
    
    checks = [
        ("Import checks", validate_imports),
        ("Backend class", validate_backend_class),
        ("Functional API", validate_functional_api),
        ("Services layer", validate_services_layer),
        ("Legacy code removal", validate_no_legacy_code),
        ("Data operations", validate_data_operations),
    ]
    
    results = []
    for name, check_func in checks:
        try:
            success = check_func()
            results.append((name, success))
        except Exception as e:
            print(f"\n✗ {name} raised exception: {e}")
            import traceback
            traceback.print_exc()
            results.append((name, False))
    
    # Summary
    print("\n" + "=" * 60)
    print("VALIDATION SUMMARY")
    print("=" * 60)
    
    passed = sum(1 for _, success in results if success)
    total = len(results)
    
    for name, success in results:
        status = "✓ PASS" if success else "✗ FAIL"
        print(f"{status} - {name}")
    
    print("=" * 60)
    print(f"Result: {passed}/{total} checks passed")
    print("=" * 60)
    
    if passed == total:
        print("\n🎉 MIGRATION SUCCESSFUL! All validation checks passed.")
        return 0
    else:
        print(f"\n❌ MIGRATION INCOMPLETE: {total - passed} checks failed.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
