/// Modified Julian Date — re-exported from the upstream time library.
///
/// Serialises as a raw `f64` (days since the MJD epoch, 1858-11-17).
pub use siderust::time::ModifiedJulianDate;

#[cfg(test)]
mod tests {
    use super::ModifiedJulianDate;

    #[test]
    fn test_mjd_new_and_value() {
        let mjd = ModifiedJulianDate::new(50000.0);
        assert_eq!(mjd.value(), 50000.0);
    }

    #[test]
    fn test_mjd_equality() {
        let mjd1 = ModifiedJulianDate::new(52000.0);
        let mjd2 = ModifiedJulianDate::new(52000.0);
        let mjd3 = ModifiedJulianDate::new(52001.0);
        assert_eq!(mjd1, mjd2);
        assert_ne!(mjd1, mjd3);
    }

    #[test]
    fn test_mjd_ordering() {
        let mjd1 = ModifiedJulianDate::new(50000.0);
        let mjd2 = ModifiedJulianDate::new(51000.0);
        assert!(mjd1 < mjd2);
        assert!(mjd2 > mjd1);
    }

    #[test]
    fn test_mjd_negative_and_zero() {
        assert_eq!(ModifiedJulianDate::new(-1000.0).value(), -1000.0);
        assert_eq!(ModifiedJulianDate::new(0.0).value(), 0.0);
    }
}
