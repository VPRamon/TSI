use serde::*;

/// Modified Julian Date representation.
/// MJD 0 = 1858-11-17 00:00:00 UTC
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModifiedJulianDate(qtty::Days);

impl ModifiedJulianDate {
    /// Create a new MJD value.
    pub fn new<V: Into<qtty::Days>>(v: V) -> Self {
        Self(v.into())
    }

    /// Raw MJD value as f64.
    pub fn value(&self) -> f64 {
        self.0.value()
    }

    /// Convert to Unix timestamp (seconds since 1970-01-01 00:00:00 UTC).
    pub fn to_unix_timestamp(&self) -> f64 {
        (self.value() - 40587.0) * 86400.0
    }

    /// Create from Unix timestamp (seconds since 1970-01-01 00:00:00 UTC).
    pub fn from_unix_timestamp(timestamp: f64) -> Self {
        Self::new(timestamp / 86400.0 + 40587.0)
    }

    /// Convert to chrono DateTime<Utc>.
    pub fn to_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        let secs = self.to_unix_timestamp();
        let secs_i64 = secs.floor() as i64;
        let nanos = ((secs - secs.floor()) * 1e9) as u32;
        chrono::DateTime::from_timestamp(secs_i64, nanos)
            .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH)
    }

    /// Create from chrono DateTime<Utc>.
    pub fn from_datetime(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self::from_unix_timestamp(dt.timestamp() as f64 + dt.timestamp_subsec_nanos() as f64 / 1e9)
    }
}

impl From<f64> for ModifiedJulianDate {
    fn from(v: f64) -> Self {
        ModifiedJulianDate::new(v)
    }
}

#[cfg(test)]
mod tests {
    use super::ModifiedJulianDate;

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
    fn test_mjd_clone() {
        let mjd1 = ModifiedJulianDate::new(51544.0);
        let mjd2 = mjd1;
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

    #[test]
    fn test_mjd_to_unix_timestamp() {
        // MJD 40587.0 corresponds to Unix epoch (1970-01-01)
        let mjd = ModifiedJulianDate::new(40587.0);
        assert!((mjd.to_unix_timestamp()).abs() < 1.0);
    }

    #[test]
    fn test_mjd_roundtrip_unix() {
        let original = ModifiedJulianDate::new(59000.5);
        let timestamp = original.to_unix_timestamp();
        let roundtrip = ModifiedJulianDate::from_unix_timestamp(timestamp);
        assert!((original.value() - roundtrip.value()).abs() < 1e-9);
    }
}
