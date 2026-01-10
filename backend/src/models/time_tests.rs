#[cfg(test)]
mod tests {
    use crate::api::ModifiedJulianDate;

    #[test]
    fn test_mjd_new() {
        let mjd = ModifiedJulianDate::new(50000.0);
        assert_eq!(mjd.value(), 50000.0);
    }

    #[test]
    fn test_mjd_from_f64() {
        let mjd: ModifiedJulianDate = 58849.0.into();
        assert_eq!(mjd.value(), 58849.0);
    }

    #[test]
    fn test_mjd_value() {
        let mjd = ModifiedJulianDate::new(59000.5);
        assert_eq!(mjd.value(), 59000.5);
    }

    #[test]
    fn test_mjd_py_new() {
        let mjd = ModifiedJulianDate::py_new(60000.123);
        assert_eq!(mjd.value(), 60000.123);
    }

    #[test]
    fn test_mjd_get_value() {
        let mjd = ModifiedJulianDate::new(55555.5);
        assert_eq!(mjd.get_value(), 55555.5);
    }

    #[test]
    fn test_mjd_float() {
        let mjd = ModifiedJulianDate::new(57000.25);
        assert_eq!(mjd.__float__(), 57000.25);
    }

    #[test]
    fn test_mjd_clone() {
        let mjd1 = ModifiedJulianDate::new(51544.0);
        let mjd2 = mjd1.clone();
        assert_eq!(mjd1.value(), mjd2.value());
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
    fn test_mjd_negative_values() {
        let mjd = ModifiedJulianDate::new(-1000.0);
        assert_eq!(mjd.value(), -1000.0);
    }

    #[test]
    fn test_mjd_zero() {
        let mjd = ModifiedJulianDate::new(0.0);
        assert_eq!(mjd.value(), 0.0);
    }

    #[test]
    fn test_mjd_large_values() {
        let mjd = ModifiedJulianDate::new(100000.999);
        assert_eq!(mjd.value(), 100000.999);
    }
}
