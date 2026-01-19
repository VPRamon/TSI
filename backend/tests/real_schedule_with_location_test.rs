//! End-to-end test with real schedule data from sensitive/ directory

#![cfg(test)]

use std::fs;
use tsi_rust::models::parse_schedule_json_str;

#[test]
fn test_parse_real_ap_schedule_with_location() {
    // Read the real AP schedule (first 5000 lines for faster testing)
    let schedule_path = "/workspace/data/sensitive/ap/schedule.json";

    if !std::path::Path::new(schedule_path).exists() {
        println!("Skipping test - sensitive schedule file not available");
        return;
    }

    let schedule_json = fs::read_to_string(schedule_path).expect("Failed to read AP schedule");

    let result = parse_schedule_json_str(&schedule_json);
    assert!(
        result.is_ok(),
        "Failed to parse AP schedule: {:?}",
        result.err()
    );

    let schedule = result.unwrap();

    // Verify geographic location was parsed
    let location = &schedule.geographic_location;
    assert_eq!(
        location.latitude, 28.7624,
        "Wrong latitude for Roque de los Muchachos"
    );
    assert_eq!(
        location.longitude, -17.8892,
        "Wrong longitude for Roque de los Muchachos"
    );
    assert_eq!(
        location.elevation_m,
        Some(2396.0),
        "Wrong elevation for Roque de los Muchachos"
    );

    // Verify astronomical nights were computed
    assert!(
        !schedule.astronomical_nights.is_empty(),
        "Expected astronomical nights to be computed for AP schedule"
    );

    println!("✅ AP Schedule parsed successfully");
    println!("   - Blocks: {}", schedule.blocks.len());
    println!(
        "   - Location: {:.4}°N, {:.4}°W, {} m",
        location.latitude,
        -location.longitude,
        location.elevation_m.unwrap_or(0.0)
    );
    println!(
        "   - Astronomical nights: {} periods",
        schedule.astronomical_nights.len()
    );
    println!(
        "   - Schedule period: {:.1} to {:.1} MJD",
        schedule.schedule_period.start.value(),
        schedule.schedule_period.stop.value()
    );

    // Verify nights are within schedule period
    for (i, night) in schedule.astronomical_nights.iter().enumerate() {
        assert!(
            night.start.value() >= schedule.schedule_period.start.value(),
            "Night {} starts before schedule period",
            i
        );
        assert!(
            night.stop.value() <= schedule.schedule_period.stop.value(),
            "Night {} ends after schedule period",
            i
        );
        assert!(
            night.start.value() < night.stop.value(),
            "Night {} has invalid period",
            i
        );

        // Log first few nights
        if i < 3 {
            let duration_hours = (night.stop.value() - night.start.value()) * 24.0;
            println!(
                "   - Night {}: {:.1} to {:.1} MJD ({:.1} hours)",
                i + 1,
                night.start.value(),
                night.stop.value(),
                duration_hours
            );
        }
    }
}

#[test]
fn test_parse_real_est_schedule_with_location() {
    let schedule_path = "/workspace/data/sensitive/est/schedule.json";

    if !std::path::Path::new(schedule_path).exists() {
        println!("Skipping test - sensitive schedule file not available");
        return;
    }

    let schedule_json = fs::read_to_string(schedule_path).expect("Failed to read EST schedule");

    let result = parse_schedule_json_str(&schedule_json);
    assert!(
        result.is_ok(),
        "Failed to parse EST schedule: {:?}",
        result.err()
    );

    let schedule = result.unwrap();

    // Verify geographic location
    let location = &schedule.geographic_location;
    assert_eq!(location.latitude, 28.7624);
    assert_eq!(location.longitude, -17.8892);

    // Verify astronomical nights
    assert!(!schedule.astronomical_nights.is_empty());

    println!("✅ EST Schedule parsed successfully");
    println!("   - Blocks: {}", schedule.blocks.len());
    println!(
        "   - Astronomical nights: {} periods",
        schedule.astronomical_nights.len()
    );
}
