# TSI Rust Backend Tests

## Overview

This directory contains comprehensive unit tests for the Telescope Scheduling Intelligence (TSI) Rust backend, implementing Product Assurance best practices to achieve 80%+ code coverage.

## Test Structure

```
backend/src/
├── parsing/
│   ├── json_parser_tests.rs          # 15 tests - JSON parsing & validation
│   ├── csv_parser_tests.rs           # 18 tests - CSV parsing & derived columns
│   └── dark_periods_parser_tests.rs  # 31 tests - Dark periods in multiple formats
├── io/
│   └── loaders_tests.rs               # 21 tests - File loaders & auto-detection
├── preprocessing/                      # TODO: Phase 2
├── algorithms/                         # TODO: Phase 3
├── transformations/                    # TODO: Phase 4
└── python/                             # TODO: Phase 5 (pytest)
```

## Current Status

### ✅ Phase 1 Complete: Parsing & IO (85 tests, ~2,600 LOC)

- **JSON Parser**: String/int ID handling, error context preservation, constraint validation
- **CSV Parser**: Optional columns, visibility strings, derived column generation
- **Dark Periods Parser**: 8+ JSON shapes, 6+ timestamp formats, timezone handling
- **IO Loaders**: Extension auto-detection, error propagation, DataFrame validation

### 🚧 Phase 2-6 In Progress

See `TEST_IMPLEMENTATION_GUIDE.md` for templates and patterns to complete:
- Phase 2: Preprocessing & Validation (~40 tests)
- Phase 3: Algorithms (~35 tests)
- Phase 4: Transformations (~30 tests)
- Phase 5: Python Bindings (~50 pytest tests)
- Phase 6: Integration & CI (~10 tests + tooling)

## Running Tests

### Quick Start

```bash
# Run all tests
./run_tests.sh

# Run specific module
./run_tests.sh --module json_parser_tests

# Run with coverage
./run_tests.sh --coverage
```

### Manual Execution

```bash
# Run all library tests
cargo test --lib

# Run specific test module
cargo test --lib parsing::json_parser_tests

# Run single test with output
cargo test --lib test_parse_string_ids -- --nocapture

# Run tests in specific file
cargo test --lib --test test_integration
```

### Coverage Analysis

```bash
# Install cargo-tarpaulin (once)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --ignore-tests --out Html --out Xml --output-dir coverage/

# View coverage
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

## Test Categories

### Table-Driven Tests

Each test module uses fixtures and variations to systematically cover edge cases:

```rust
#[test]
fn test_parse_string_ids() { /* ... */ }

#[test]
fn test_parse_integer_ids() { /* ... */ }

#[test]
fn test_parse_mixed_types() { /* ... */ }
```

### Error Context Validation

All error paths verify anyhow context is preserved:

```rust
let result = parse_function(invalid_input);
assert!(result.is_err());
let error_msg = result.unwrap_err().to_string();
assert!(error_msg.contains("expected context"), "Error: {}", error_msg);
```

### Realistic Fixtures

Tests use data from `data/` directory when possible:
- `data/schedule.json` for JSON parsing
- `data/dark_periods.json` for dark period parsing
- `data/schedule.csv` for CSV parsing

### Derived Values

DataFrame tests verify computed columns:
- `scheduled_flag`: Boolean indicating if block is scheduled
- `priority_bin`: Categorical priority grouping
- `total_visibility_hours`: Sum of visibility period durations
- `elevation_range_deg`: Max - Min elevation angles

## Dependencies

### Dev Dependencies

```toml
[dev-dependencies]
tempfile = "3.8"      # Temporary file creation for filesystem tests
criterion = "0.7"     # Benchmarking (existing)
proptest = "1.4"      # Property-based testing (existing)
```

## Code Coverage Targets

| Module | Target | Current (Phase 1) |
|--------|--------|-------------------|
| `parsing/json_parser.rs` | 85% | ~85% ✅ |
| `parsing/csv_parser.rs` | 80% | ~80% ✅ |
| `parsing/dark_periods_parser.rs` | 90% | ~90% ✅ |
| `io/loaders.rs` | 85% | ~85% ✅ |
| `preprocessing/pipeline.rs` | 80% | 0% 🚧 |
| `preprocessing/enricher.rs` | 80% | ~40% 🚧 |
| `preprocessing/validator.rs` | 80% | ~50% 🚧 |
| `algorithms/analysis.rs` | 75% | ~20% 🚧 |
| `algorithms/conflicts.rs` | 70% | ~15% 🚧 |
| `algorithms/optimization.rs` | 75% | ~30% 🚧 |
| `transformations/cleaning.rs` | 80% | ~25% 🚧 |
| `transformations/filtering.rs` | 80% | ~30% 🚧 |

**Overall Target:** 80% line coverage across all production code

## Known Issues

### Bug Fixed ✓

**File:** `transformations/cleaning.rs` (line ~60)
**Issue:** Median imputation strategy was incorrectly using `FillNullStrategy::Mean`

**Status:** FIXED - Median strategy now correctly computes the median value and fills nulls with it.

**Implementation:** Since Polars doesn't provide a built-in `FillNullStrategy::Median`, the fix manually computes the median and uses `zip_with` to replace null values.

```rust
// Fixed implementation
"median" => {
    let float_series = series.cast(&DataType::Float64)?;
    if let Some(median_val) = float_series.median() {
        let mask = float_series.is_null();
        let median_series = Series::from_vec(float_series.name().clone(), vec![median_val; float_series.len()]);
        let filled = float_series.zip_with(&mask, &median_series)?;
        Ok(filled)
    } else {
        Ok(series.clone())
    }
}
```

## Documentation

- **TEST_COVERAGE_REPORT.md**: Comprehensive report of implemented tests
- **TEST_IMPLEMENTATION_GUIDE.md**: Templates for implementing remaining tests
- **PYTHON_API.md**: Python binding documentation (existing)

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test-coverage.yml
name: Test & Coverage
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: cargo test --lib
      - name: Generate coverage
        run: cargo tarpaulin --ignore-tests --out Xml
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
      - name: Check 80% threshold
        run: ./scripts/check_coverage_threshold.sh
```

### Pre-commit Hooks

```bash
# Install pre-commit hook
echo "cargo test --lib" > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## Contributing

### Adding New Tests

1. Create test file in same directory as source: `<module>_tests.rs`
2. Add `#[cfg(test)]` module to parent `mod.rs`
3. Follow patterns in `TEST_IMPLEMENTATION_GUIDE.md`
4. Ensure tests cover:
   - Happy path with realistic data
   - Error cases with context verification
   - Edge cases (None, empty, invalid types)
   - Integration points (file I/O, cross-module calls)
5. Run `./run_tests.sh` locally before committing
6. Update this README if adding new test modules

### Test Review Checklist

- [ ] Tests use descriptive names (`test_parse_string_ids` not `test1`)
- [ ] Error messages are asserted (not just `is_err()`)
- [ ] Tempfiles are used for filesystem tests
- [ ] Realistic fixtures over handcrafted data
- [ ] Both success and failure paths covered
- [ ] No hardcoded paths or assumptions about environment
- [ ] Tests are independent (no shared mutable state)
- [ ] Documentation updated if needed

## Troubleshooting

### "cannot find module tempfile"

Add to `Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3.8"
```

### "test result: FAILED" with no details

Run with output flag:
```bash
cargo test --lib -- --nocapture
```

### Slow compilation

Tests only compile with `cargo test`, not `cargo build`. To speed up:
```bash
# Only compile, don't run
cargo test --lib --no-run

# Then run without recompiling
cargo test --lib
```

### Coverage tool errors

Ensure you're using latest stable Rust:
```bash
rustup update stable
cargo clean
cargo tarpaulin --version  # Should be >= 0.27
```

## Performance

Test execution times (approximate):
- **JSON Parser Tests**: ~200ms
- **CSV Parser Tests**: ~150ms  
- **Dark Periods Parser Tests**: ~300ms
- **IO Loaders Tests**: ~250ms
- **Total Phase 1**: ~1 second

Coverage generation: ~30-60 seconds (includes recompilation)

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Proptest Book](https://proptest-rs.github.io/proptest/)
- [tempfile crate](https://docs.rs/tempfile/)

## Questions?

For test-related questions:
1. Check `TEST_IMPLEMENTATION_GUIDE.md` for patterns
2. Look at existing tests in `*_tests.rs` files
3. Review `TEST_COVERAGE_REPORT.md` for detailed documentation
4. Contact Product Assurance team

---

**Last Updated:** November 25, 2025  
**Test Count:** 85 implemented, ~210 remaining  
**Coverage:** Phase 1 complete, Phases 2-6 in progress
