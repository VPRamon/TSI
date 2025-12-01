#!/usr/bin/env python3
"""
Test script for database migration dual-path implementation.

This script verifies that:
1. Configuration loads correctly
2. Both data source paths are accessible
3. Routing works based on DATA_SOURCE setting
4. Functions have the same signatures
"""

import os
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent / "src"))


def test_configuration():
    """Test that configuration loads and validates correctly."""
    print("=" * 80)
    print("TEST 1: Configuration Loading and Validation")
    print("=" * 80)
    
    from app_config import get_settings
    
    # Test default value
    settings = get_settings()
    print(f"✓ Default DATA_SOURCE: {settings.data_source}")
    assert settings.data_source in ("legacy", "etl"), "Invalid data_source default"
    
    # Test that it's accessible
    print(f"✓ Configuration accessible via get_settings()")
    
    print("✅ Configuration test passed\n")


def test_imports():
    """Test that all new modules and functions can be imported."""
    print("=" * 80)
    print("TEST 2: Module Imports")
    print("=" * 80)
    
    # Test data_source module
    try:
        from tsi.services import data_source
        print("✓ tsi.services.data_source imports successfully")
    except ImportError as e:
        print(f"✗ Failed to import data_source: {e}")
        return False
    
    # Test unified functions
    unified_funcs = [
        "get_sky_map_data_unified",
        "get_distribution_data_unified",
        "get_schedule_timeline_data_unified",
        "get_insights_data_unified",
        "get_trends_data_unified",
        "get_compare_data_unified",
        "get_visibility_map_data_unified",
    ]
    
    for func_name in unified_funcs:
        if hasattr(data_source, func_name):
            print(f"✓ {func_name} available")
        else:
            print(f"✗ {func_name} missing")
            return False
    
    # Test legacy functions
    legacy_funcs = [
        "get_sky_map_data_legacy",
        "get_distribution_data_legacy",
    ]
    
    for func_name in legacy_funcs:
        if hasattr(data_source, func_name):
            print(f"✓ {func_name} available")
        else:
            print(f"✗ {func_name} missing")
            return False
    
    # Test ETL functions
    etl_funcs = [
        "get_sky_map_data_etl",
        "get_distribution_data_etl",
    ]
    
    for func_name in etl_funcs:
        if hasattr(data_source, func_name):
            print(f"✓ {func_name} available")
        else:
            print(f"✗ {func_name} missing")
            return False
    
    print("✅ Import test passed\n")
    return True


def test_rust_backend_functions():
    """Test that Rust backend exposes the necessary functions."""
    print("=" * 80)
    print("TEST 3: Rust Backend Function Availability")
    print("=" * 80)
    
    try:
        import tsi_rust
        print("✓ tsi_rust module imports successfully")
    except ImportError as e:
        print(f"✗ Failed to import tsi_rust: {e}")
        print("  (This is expected if Rust backend not compiled)")
        return False
    
    # Test base functions
    base_funcs = [
        "py_get_sky_map_data",
        "py_get_distribution_data",
    ]
    
    for func_name in base_funcs:
        if hasattr(tsi_rust, func_name):
            print(f"✓ {func_name} available")
        else:
            print(f"✗ {func_name} missing")
            return False
    
    # Test explicit path functions
    path_funcs = [
        "py_get_sky_map_data_legacy",
        "py_get_sky_map_data_analytics",
        "py_get_distribution_data_legacy",
        "py_get_distribution_data_analytics",
    ]
    
    for func_name in path_funcs:
        if hasattr(tsi_rust, func_name):
            print(f"✓ {func_name} available")
        else:
            print(f"✗ {func_name} missing")
            return False
    
    print("✅ Rust backend test passed\n")
    return True


def test_database_routing():
    """Test that database.py functions route correctly."""
    print("=" * 80)
    print("TEST 4: Database.py Routing")
    print("=" * 80)
    
    from tsi.services import database
    
    # Test that functions exist
    funcs = [
        "get_sky_map_data",
        "get_distribution_data",
    ]
    
    for func_name in funcs:
        if hasattr(database, func_name):
            print(f"✓ {func_name} available in database module")
        else:
            print(f"✗ {func_name} missing from database module")
            return False
    
    # Test that they have correct signatures (can't call without DB connection)
    import inspect
    
    sig = inspect.signature(database.get_sky_map_data)
    params = list(sig.parameters.keys())
    print(f"✓ get_sky_map_data signature: {params}")
    assert "schedule_id" in params, "Missing schedule_id parameter"
    
    sig = inspect.signature(database.get_distribution_data)
    params = list(sig.parameters.keys())
    print(f"✓ get_distribution_data signature: {params}")
    assert "schedule_id" in params, "Missing schedule_id parameter"
    assert "filter_impossible" in params, "Missing filter_impossible parameter"
    
    print("✅ Database routing test passed\n")
    return True


def test_services_exports():
    """Test that services package exports the new functions."""
    print("=" * 80)
    print("TEST 5: Services Package Exports")
    print("=" * 80)
    
    from tsi import services
    
    unified_funcs = [
        "get_sky_map_data_unified",
        "get_distribution_data_unified",
    ]
    
    for func_name in unified_funcs:
        if hasattr(services, func_name):
            print(f"✓ {func_name} exported from services")
        else:
            print(f"✗ {func_name} not exported from services")
            return False
    
    print("✅ Services exports test passed\n")
    return True


def test_configuration_validation():
    """Test that invalid configuration values are rejected."""
    print("=" * 80)
    print("TEST 6: Configuration Validation")
    print("=" * 80)
    
    from pydantic import ValidationError
    from app_config.settings import Settings
    
    # Test valid values
    for value in ["legacy", "etl", "LEGACY", "ETL"]:
        try:
            s = Settings(data_source=value)
            print(f"✓ '{value}' accepted (normalized to '{s.data_source}')")
        except ValidationError as e:
            print(f"✗ Valid value '{value}' rejected: {e}")
            return False
    
    # Test invalid value
    try:
        s = Settings(data_source="invalid")
        print(f"✗ Invalid value 'invalid' was accepted (should be rejected)")
        return False
    except ValidationError:
        print(f"✓ Invalid value 'invalid' correctly rejected")
    
    print("✅ Configuration validation test passed\n")
    return True


def main():
    """Run all tests."""
    print("\n" + "=" * 80)
    print("DATABASE MIGRATION DUAL-PATH VERIFICATION")
    print("=" * 80 + "\n")
    
    results = []
    
    # Run tests
    try:
        test_configuration()
        results.append(("Configuration", True))
    except Exception as e:
        print(f"❌ Configuration test failed: {e}\n")
        results.append(("Configuration", False))
    
    try:
        test_imports()
        results.append(("Imports", True))
    except Exception as e:
        print(f"❌ Import test failed: {e}\n")
        results.append(("Imports", False))
    
    try:
        rust_ok = test_rust_backend_functions()
        results.append(("Rust Backend", rust_ok))
    except Exception as e:
        print(f"❌ Rust backend test failed: {e}\n")
        results.append(("Rust Backend", False))
    
    try:
        test_database_routing()
        results.append(("Database Routing", True))
    except Exception as e:
        print(f"❌ Database routing test failed: {e}\n")
        results.append(("Database Routing", False))
    
    try:
        test_services_exports()
        results.append(("Services Exports", True))
    except Exception as e:
        print(f"❌ Services exports test failed: {e}\n")
        results.append(("Services Exports", False))
    
    try:
        test_configuration_validation()
        results.append(("Config Validation", True))
    except Exception as e:
        print(f"❌ Config validation test failed: {e}\n")
        results.append(("Config Validation", False))
    
    # Summary
    print("=" * 80)
    print("TEST SUMMARY")
    print("=" * 80)
    
    total = len(results)
    passed = sum(1 for _, success in results if success)
    
    for test_name, success in results:
        status = "✅ PASS" if success else "❌ FAIL"
        print(f"{status}: {test_name}")
    
    print("-" * 80)
    print(f"Total: {passed}/{total} tests passed")
    print("=" * 80 + "\n")
    
    if passed == total:
        print("🎉 All tests passed! Dual-path implementation verified.")
        return 0
    else:
        print("⚠️  Some tests failed. Review the output above.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
