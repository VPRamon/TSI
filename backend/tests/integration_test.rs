use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Basic integration test: spawn the server, hit /health and /api/v1/compute.
/// NOTE: This test expects the server to be already running on localhost:8080,
/// or you can modify it to spawn `cargo run` in the background (brittle for CI).
#[tokio::test]
#[ignore] // Mark as ignored by default since it requires a running server
async fn test_compute_endpoint() {
    // Wait briefly for server startup if running in parallel
    sleep(Duration::from_millis(500)).await;

    let client = Client::new();
    
    // Health check
    let health_resp = client.get("http://127.0.0.1:8080/health")
        .send()
        .await
        .expect("health check failed");
    assert!(health_resp.status().is_success());

    // Compute endpoint
    let compute_resp = client.post("http://127.0.0.1:8080/api/v1/compute")
        .json(&json!({"values": [1.0, 2.0, 3.0, 4.0]}))
        .send()
        .await
        .expect("compute request failed");

    assert!(compute_resp.status().is_success());
    let body: serde_json::Value = compute_resp.json().await.expect("invalid json");
    
    assert!(body.get("mean").is_some());
    assert!(body.get("std").is_some());
    
    let mean = body["mean"].as_f64().unwrap();
    let std = body["std"].as_f64().unwrap();
    
    // Verify correctness
    assert!((mean - 2.5).abs() < 1e-9, "mean={}", mean);
    assert!((std - 1.2909944487358056).abs() < 1e-9, "std={}", std);
}
