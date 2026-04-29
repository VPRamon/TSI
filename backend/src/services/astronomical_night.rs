// ! Astronomical night computation service.
//!
//! Computes astronomical night periods (Sun altitude < -18°) for a given
//! observer location and time period using the siderust astronomy library.

use siderust::bodies::solar_system::Moon;
use siderust::bodies::Sun;
use siderust::calculus::solar::Twilight;
use siderust::time::intersect_periods;
use siderust::{below_threshold, SearchOpts};

use crate::api::{GeographicLocation, Period};

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
    // `Period` is `siderust::time::Interval<ModifiedJulianDate>` — no conversion needed.
    below_threshold(
        &Sun,
        location,
        *time_period,
        Twilight::Astronomical.into(),
        SearchOpts::default(),
    )
}

/// Compute dark periods for a given observer location and time period.
///
/// Dark periods are defined as the intersection of astronomical nights
/// (Sun altitude < -18°) and Moon-below-horizon periods (Moon altitude < 0°).
///
/// # Arguments
///
/// * `location` - Geographic location of the observer
/// * `time_period` - Time window to search within (in MJD)
/// * `astronomical_nights` - Pre-computed astronomical night periods (Sun < -18°)
///
/// # Returns
///
/// Vector of dark periods (Moon below horizon AND Sun < -18°).
pub fn compute_dark_periods(
    location: &GeographicLocation,
    time_period: &Period,
    astronomical_nights: &[Period],
) -> Vec<Period> {
    let moon_below_horizon = below_threshold(
        &Moon,
        location,
        *time_period,
        Twilight::Horizon.into(),
        SearchOpts::default(),
    );
    intersect_periods(astronomical_nights, &moon_below_horizon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ModifiedJulianDate;

    #[test]
    fn test_compute_astronomical_nights_roque_de_los_muchachos() {
        use qtty::{Degrees, Meters};
        use siderust::coordinates::centers::Geodetic;
        use siderust::coordinates::frames::ECEF;

        // Roque de los Muchachos Observatory
        let location = Geodetic::<ECEF>::new(
            Degrees::new(-17.8892),
            Degrees::new(28.7624),
            Meters::new(2396.0),
        );

        // One week in January 2026
        let period = Period {
            start: ModifiedJulianDate::new(60694.0), // 2026-01-15
            end: ModifiedJulianDate::new(60701.0),   // 2026-01-22
        };

        let nights = compute_astronomical_nights(&location, &period);

        // Should find several astronomical night periods in a week
        assert!(!nights.is_empty(), "Expected to find astronomical nights");

        // Each night should be a valid period with start < stop
        for night in &nights {
            assert!(
                night.start.value() < night.end.value(),
                "Night period should have start < stop"
            );
            assert!(
                night.start.value() >= period.start.value(),
                "Night should start within search period"
            );
            assert!(
                night.end.value() <= period.end.value(),
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
        use qtty::{Degrees, Meters};
        use siderust::coordinates::centers::Geodetic;
        use siderust::coordinates::frames::ECEF;

        // Greenwich Observatory
        let location =
            Geodetic::<ECEF>::new(Degrees::new(0.0), Degrees::new(51.4769), Meters::new(0.0));

        // One day in January (winter)
        let period = Period {
            start: ModifiedJulianDate::new(60694.0), // 2026-01-15
            end: ModifiedJulianDate::new(60695.0),   // 2026-01-16
        };

        let nights = compute_astronomical_nights(&location, &period);

        // Should find at least one astronomical night in winter
        assert!(!nights.is_empty(), "Expected to find astronomical night");

        // The night should be several hours long in winter
        let duration_hours = (nights[0].end.value() - nights[0].start.value()) * 24.0;
        assert!(
            duration_hours > 4.0,
            "Expected night duration > 4 hours, got {:.1}",
            duration_hours
        );
    }
}
