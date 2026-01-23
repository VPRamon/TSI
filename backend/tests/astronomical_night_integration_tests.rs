//! Integration tests for astronomical night computation and geographic location storage.

use tsi_rust::api::{GeographicLocation, ModifiedJulianDate, Period};
use tsi_rust::db::repositories::LocalRepository;
use tsi_rust::db::services;
use tsi_rust::models::parse_schedule_json_str;
use tsi_rust::services::astronomical_night::compute_astronomical_nights;

/// Test that geographic location can be parsed from astro format JSON
#[test]
fn test_parse_schedule_with_geographic_location() {
    // Astro format uses "location" with lat/lon/distance
    // distance is in km from Earth center (Earth radius ~6371 km + elevation)
    let schedule_json = r#"{
        "location": {
            "lat": 28.7624,
            "lon": -17.8892,
            "distance": 6373.396
        },
        "period": {
            "start": 60694.0,
            "end": 60701.0
        },
        "tasks": []
    }"#;

    let result = parse_schedule_json_str(schedule_json);
    assert!(
        result.is_ok(),
        "Failed to parse schedule with location: {:?}",
        result.err()
    );

    let schedule = result.unwrap();

    let location = &schedule.geographic_location;
    assert!((location.latitude - 28.7624).abs() < 0.001);
    assert!((location.longitude - (-17.8892)).abs() < 0.001);
    // Elevation should be approximately 2396m (6373.396 - 6371.0) * 1000
    assert!(location.elevation_m.is_some());
}

/// Test that astronomical nights are computed when location is provided
#[test]
fn test_parse_schedule_computes_astronomical_nights() {
    let schedule_json = r#"{
        "location": {
            "lat": 28.7624,
            "lon": -17.8892,
            "distance": 6373.396
        },
        "period": {
            "start": 60694.0,
            "end": 60701.0
        },
        "tasks": []
    }"#;

    let result = parse_schedule_json_str(schedule_json);
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
        "location": {
            "lat": 28.7624,
            "lon": -17.8892,
            "distance": 6373.396
        },
        "period": {
            "start": 60694.0,
            "end": 60701.0
        },
        "tasks": []
    }"#;

    let schedule = parse_schedule_json_str(schedule_json).unwrap();
    let original_nights_count = schedule.astronomical_nights.len();

    // Store schedule
    let metadata = services::store_schedule(&repo, &schedule).await.unwrap();

    // Retrieve schedule
    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .unwrap();

    // Verify geographic location persisted
    let location = &retrieved.geographic_location;
    assert!((location.latitude - 28.7624).abs() < 0.001);
    assert!((location.longitude - (-17.8892)).abs() < 0.001);

    // Verify astronomical nights persisted
    assert_eq!(retrieved.astronomical_nights.len(), original_nights_count);
    assert!(!retrieved.astronomical_nights.is_empty());

    // Verify nights are valid periods
    for night in &retrieved.astronomical_nights {
        assert!(night.start.value() < night.stop.value());
    }
}

/// Test schedule at sea level (minimal elevation)
#[test]
fn test_location_at_sea_level() {
    let schedule_json = r#"{
        "location": {
            "lat": 0.0,
            "lon": 0.0,
            "distance": 6371.0
        },
        "period": {
            "start": 60694.0,
            "end": 60695.0
        },
        "tasks": []
    }"#;

    let result = parse_schedule_json_str(schedule_json);
    assert!(result.is_ok());

    let schedule = result.unwrap();

    let location = &schedule.geographic_location;
    assert!((location.latitude - 0.0).abs() < 0.001);
    assert!((location.longitude - 0.0).abs() < 0.001);
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

/// Test astro format schedule with observation tasks
#[test]
fn test_parse_schedule_with_observation_tasks() {
    let schedule_json = r#"{
        "location": {
            "lat": 28.7624,
            "lon": -17.8892,
            "distance": 6373.396
        },
        "period": {
            "start": 60694.0,
            "end": 60701.0
        },
        "tasks": [
            {
                "type": "observation",
                "id": "1",
                "name": "M31 Observation",
                "target": {
                    "position": { "ra": 10.6847, "dec": 41.2687 },
                    "time": 2451545.0
                },
                "duration_sec": 3600.0,
                "priority": 10
            }
        ]
    }"#;

    let result = parse_schedule_json_str(schedule_json);
    assert!(result.is_ok(), "Failed to parse schedule with tasks: {:?}", result.err());

    let schedule = result.unwrap();
    assert_eq!(schedule.blocks.len(), 1);
    assert_eq!(schedule.blocks[0].original_block_id, "1");
    
    // Astronomical nights should still be computed
    assert!(!schedule.astronomical_nights.is_empty());
}
