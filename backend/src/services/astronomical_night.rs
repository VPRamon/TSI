// ! Astronomical night computation service.
//!
//! Computes astronomical night periods (Sun altitude < -18°) for a given
//! observer location and time period using the siderust astronomy library.

use qtty::{Degrees, Meter, Quantity};
use siderust::astro::ModifiedJulianDate as SiderustMJD;
use siderust::calculus::solar::altitude_periods::{find_night_periods, twilight};
use siderust::coordinates::centers::ObserverSite;
use siderust::time::Period as SiderustPeriod;

use crate::api::{GeographicLocation, Period};
use crate::models::ModifiedJulianDate;

/// Compute astronomical night periods for a given observer location and time period.
///
/// Astronomical night is defined as the period when the Sun's center is more than
/// 18° below the horizon (altitude < -18°).
///
/// # Arguments
///
/// * `location` - Geographic location of the observer (latitude, longitude, elevation) - REQUIRED
/// * `time_period` - Time window to search for astronomical nights (in MJD)
///
/// # Returns
///
/// Vector of astronomical night periods.
pub fn compute_astronomical_nights(
    location: &GeographicLocation,
    time_period: &Period,
) -> Vec<Period> {
    // Create observer site from geographic location
    let site = ObserverSite::new(
        Degrees::new(location.longitude),
        Degrees::new(location.latitude),
        Quantity::<Meter>::new(location.elevation_m.unwrap_or(0.0)),
    );

    // Convert our Period to siderust Period
    let start_mjd = SiderustMJD::new(time_period.start.value());
    let stop_mjd = SiderustMJD::new(time_period.stop.value());
    let search_period = SiderustPeriod::new(start_mjd, stop_mjd);

    // Find astronomical night periods
    let nights = find_night_periods(site, search_period, twilight::ASTRONOMICAL);

    // Convert siderust Periods back to our Period type
    nights
        .unwrap_or_default()
        .into_iter()
        .map(|p| Period {
            start: ModifiedJulianDate::new(p.start.value()),
            stop: ModifiedJulianDate::new(p.end.value()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_astronomical_nights_roque_de_los_muchachos() {
        // Roque de los Muchachos Observatory
        let location = GeographicLocation {
            latitude: 28.7624,
            longitude: -17.8892,
            elevation_m: Some(2396.0),
        };

        // One week in January 2026
        let period = Period {
            start: ModifiedJulianDate::new(60694.0), // 2026-01-15
            stop: ModifiedJulianDate::new(60701.0),  // 2026-01-22
        };

        let nights = compute_astronomical_nights(&location, &period);

        // Should find several astronomical night periods in a week
        assert!(!nights.is_empty(), "Expected to find astronomical nights");

        // Each night should be a valid period with start < stop
        for night in &nights {
            assert!(
                night.start.value() < night.stop.value(),
                "Night period should have start < stop"
            );
            assert!(
                night.start.value() >= period.start.value(),
                "Night should start within search period"
            );
            assert!(
                night.stop.value() <= period.stop.value(),
                "Night should end within search period"
            );
        }

        // Typical astronomical night in winter at this latitude is ~8-10 hours
        // So in a week we should have around 7 nights
        assert!(
            nights.len() >= 5 && nights.len() <= 10,
            "Expected around 7 nights in a week, got {}",
            nights.len()
        );
    }

    #[test]
    fn test_compute_astronomical_nights_greenwich() {
        // Greenwich Observatory
        let location = GeographicLocation {
            latitude: 51.4769,
            longitude: 0.0,
            elevation_m: Some(0.0),
        };

        // One day in January (winter)
        let period = Period {
            start: ModifiedJulianDate::new(60694.0), // 2026-01-15
            stop: ModifiedJulianDate::new(60695.0),  // 2026-01-16
        };

        let nights = compute_astronomical_nights(&location, &period);

        // Should find at least one astronomical night in winter
        assert!(!nights.is_empty(), "Expected to find astronomical night");

        // The night should be several hours long in winter
        let duration_hours = (nights[0].stop.value() - nights[0].start.value()) * 24.0;
        assert!(
            duration_hours > 4.0,
            "Expected night duration > 4 hours, got {:.1}",
            duration_hours
        );
    }
}
