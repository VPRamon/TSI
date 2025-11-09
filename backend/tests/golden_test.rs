// Golden test: verify numerical parity between Rust (Polars) and Python (Pandas)
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct GoldenFixture {
    input: Vec<f64>,
    expected_mean: f64,
    expected_std: f64,
}

#[test]
fn golden_test_compute_parity() {
    // Load golden fixture (could be generated from Python Pandas output)
    let fixture_json = r#"{
        "input": [1.0, 2.0, 3.0, 4.0, 5.0],
        "expected_mean": 3.0,
        "expected_std": 1.5811388300841898
    }"#;
    
    let fixture: GoldenFixture = serde_json::from_str(fixture_json).unwrap();
    
    let (mean, std) = tsi_backend::compute::analyze_values(&fixture.input).unwrap();
    
    // Verify numerical parity within tolerance (±1e-9 relative error)
    let mean_rel_error = (mean - fixture.expected_mean).abs() / fixture.expected_mean.abs();
    let std_rel_error = (std - fixture.expected_std).abs() / fixture.expected_std.abs();
    
    assert!(
        mean_rel_error < 1e-9,
        "Mean mismatch: got {}, expected {}, rel_error={}",
        mean, fixture.expected_mean, mean_rel_error
    );
    
    assert!(
        std_rel_error < 1e-9,
        "Std mismatch: got {}, expected {}, rel_error={}",
        std, fixture.expected_std, std_rel_error
    );
}

#[test]
fn golden_test_larger_dataset() {
    // Simulate a larger dataset (e.g., 1000 points)
    let input: Vec<f64> = (1..=1000).map(|i| i as f64).collect();
    
    // Expected values computed with Python:
    // import numpy as np
    // data = np.arange(1, 1001, dtype=float)
    // mean = data.mean()  # 500.5
    // std = data.std(ddof=1)  # 288.8194360957494
    
    let expected_mean = 500.5;
    let expected_std = 288.8194360957494;
    
    let (mean, std) = tsi_backend::compute::analyze_values(&input).unwrap();
    
    let mean_rel_error = (mean - expected_mean).abs() / expected_mean.abs();
    let std_rel_error = (std - expected_std).abs() / expected_std.abs();
    
    assert!(
        mean_rel_error < 1e-9,
        "Mean mismatch: got {}, expected {}, rel_error={}",
        mean, expected_mean, mean_rel_error
    );
    
    assert!(
        std_rel_error < 1e-9,
        "Std mismatch: got {}, expected {}, rel_error={}",
        std, expected_std, std_rel_error
    );
}
