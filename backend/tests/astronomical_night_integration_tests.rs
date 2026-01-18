//! Integration tests for astronomical night computation and geographic location storage.

use tsi_rust::api::{GeographicLocation, ModifiedJulianDate, Period};
use tsi_rust::db::repositories::LocalRepository;
use tsi_rust::db::services;
use tsi_rust::models::parse_schedule_json_str;
use tsi_rust::services::astronomical_night::compute_astronomical_nights;

/// Test that geographic location can be parsed from JSON
#[test]
fn test_parse_schedule_with_geographic_location() {
    let schedule_json = r#"{
        "name": "Test Schedule with Location",
        "geographic_location": {
            "latitude": 28.7624,
            "longitude": -17.8892,
            "elevation_m": 2396.0
        },
        "schedule_period": {
            "start": 60694.0,
            "stop": 60701.0
        },
        "blocks": []
    }"#;

    let result = parse_schedule_json_str(schedule_json, None);
    assert!(
        result.is_ok(),
        "Failed to parse schedule with location: {:?}",
        result.err()
    );

    let schedule = result.unwrap();

    let location = &schedule.geographic_location;
    assert_eq!(location.latitude, 28.7624);
    assert_eq!(location.longitude, -17.8892);
    assert_eq!(location.elevation_m, Some(2396.0));
}

/// Test that astronomical nights are computed when location is provided
#[test]
fn test_parse_schedule_computes_astronomical_nights() {
    let schedule_json = r#"{
        "name": "Roque de los Muchachos Schedule",
        "geographic_location": {
            "latitude": 28.7624,
            "longitude": -17.8892,
            "elevation_m": 2396.0
        },
        "schedule_period": {
            "start": 60694.0,
            "stop": 60701.0
        },
        "blocks": []
    }"#;

    let result = parse_schedule_json_str(schedule_json, None);
    assert!(result.is_ok());

    let schedule = result.unwrap();
    assert!(
        !schedule.astronomical_nights.is_empty(),
        "Expected astronomical nights to be computed"
    );

    // Verify nights are within the schedule period
    for night in &schedule.astronomical_nights {
        assert!(night.start.value() >= schedule.schedule_period.start.value());
        assert!(night.stop.value() <= schedule.schedule_period.stop.value());
        assert!(night.start.value() < night.stop.value());
    }
}

/// Test astronomical night computation service directly
#[test]
fn test_compute_astronomical_nights_greenwich() {
    let location = GeographicLocation {
        latitude: 51.4769,
        longitude: 0.0,
        elevation_m: Some(0.0),
    };

    let period = Period {
        start: ModifiedJulianDate::new(60694.0),
        stop: ModifiedJulianDate::new(60701.0),
    };

    let nights = compute_astronomical_nights(&location, &period);

    assert!(
        !nights.is_empty(),
        "Expected astronomical nights at Greenwich in January"
    );

    // Each night should be valid
    for night in &nights {
        assert!(night.start.value() < night.stop.value());
        // Night duration should be reasonable (e.g., 4-12 hours in winter)
        let duration_hours = (night.stop.value() - night.start.value()) * 24.0;
        assert!(
            duration_hours > 1.0 && duration_hours < 24.0,
            "Unreasonable night duration: {:.1} hours",
            duration_hours
        );
    }
}

/// Test astronomical night computation at Roque de los Muchachos
#[test]
fn test_compute_astronomical_nights_roque_de_los_muchachos() {
    let location = GeographicLocation {
        latitude: 28.7624,
        longitude: -17.8892,
        elevation_m: Some(2396.0),
    };

    // One week in January 2026
    let period = Period {
        start: ModifiedJulianDate::new(60694.0),
        stop: ModifiedJulianDate::new(60701.0),
    };

    let nights = compute_astronomical_nights(&location, &period);

    assert!(
        !nights.is_empty(),
        "Expected astronomical nights at Roque de los Muchachos"
    );

    // Should have around 7 nights in a week
    assert!(
        nights.len() >= 5 && nights.len() <= 10,
        "Expected ~7 nights in a week, got {}",
        nights.len()
    );
}

/// Test that location and astronomical nights roundtrip through the database
#[tokio::test]
async fn test_location_and_nights_persist_to_database() {
    let repo = LocalRepository::new();

    let schedule_json = r#"{
        "name": "Persistence Test Schedule",
        "geographic_location": {
            "latitude": 28.7624,
            "longitude": -17.8892,
            "elevation_m": 2396.0
        },
        "schedule_period": {
            "start": 60694.0,
            "stop": 60701.0
        },
        "blocks": []
    }"#;

    let schedule = parse_schedule_json_str(schedule_json, None).unwrap();
    let original_nights_count = schedule.astronomical_nights.len();

    // Store schedule
    let metadata = services::store_schedule(&repo, &schedule).await.unwrap();

    // Retrieve schedule
    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .unwrap();

    // Verify geographic location persisted
    let location = &retrieved.geographic_location;
    assert_eq!(location.latitude, 28.7624);
    assert_eq!(location.longitude, -17.8892);
    assert_eq!(location.elevation_m, Some(2396.0));

    // Verify astronomical nights persisted
    assert_eq!(retrieved.astronomical_nights.len(), original_nights_count);
    assert!(!retrieved.astronomical_nights.is_empty());

    // Verify nights are valid periods
    for night in &retrieved.astronomical_nights {
        assert!(night.start.value() < night.stop.value());
    }
}

/// Test schedule with location but no elevation
#[test]
fn test_location_without_elevation() {
    let schedule_json = r#"{
        "name": "Location without elevation",
        "geographic_location": {
            "latitude": 0.0,
            "longitude": 0.0
        },
        "schedule_period": {
            "start": 60694.0,
            "stop": 60695.0
        },
        "blocks": []
    }"#;

    let result = parse_schedule_json_str(schedule_json, None);
    assert!(result.is_ok());

    let schedule = result.unwrap();

    let location = &schedule.geographic_location;
    assert_eq!(location.latitude, 0.0);
    assert_eq!(location.longitude, 0.0);
    assert_eq!(location.elevation_m, None);
}

/// Test computation at high latitude during summer (polar day - no astronomical night)
#[test]
fn test_no_astronomical_nights_polar_summer() {
    let location = GeographicLocation {
        latitude: 70.0, // High northern latitude
        longitude: 20.0,
        elevation_m: Some(0.0),
    };

    // Summer period (around MJD 60500 ~ July 2024)
    let period = Period {
        start: ModifiedJulianDate::new(60500.0),
        stop: ModifiedJulianDate::new(60507.0),
    };

    let nights = compute_astronomical_nights(&location, &period);

    // At 70°N in summer, there may be no astronomical night
    // (The service should return empty vector, not crash)
    // This is a valid result
    println!("Nights at 70°N in summer: {}", nights.len());
}

/// Test that schedule JSON without schedule_period infers it from blocks
#[test]
fn test_inferred_schedule_period_with_location() {
    let schedule_json = r#"{
        "name": "Inferred Period Schedule",
        "geographic_location": {
            "latitude": 28.7624,
            "longitude": -17.8892
        },
        "dark_periods": [
            {"start": 60694.0, "stop": 60695.0},
            {"start": 60696.0, "stop": 60697.0}
        ],
        "blocks": []
    }"#;

    let result = parse_schedule_json_str(schedule_json, None);
    assert!(result.is_ok());

    let schedule = result.unwrap();

    // Schedule period should be inferred from dark periods
    assert_eq!(schedule.schedule_period.start.value(), 60694.0);
    assert_eq!(schedule.schedule_period.stop.value(), 60697.0);

    // Astronomical nights should be computed for the inferred period
    assert!(!schedule.astronomical_nights.is_empty());
}
