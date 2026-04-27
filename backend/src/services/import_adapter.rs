//! Import adapter abstraction for schedule ingestion.
//!
//! Adapters normalize raw import payloads into TSI's canonical `Schedule`
//! model before persistence and analysis. The default implementation keeps
//! the existing native TSI JSON format unchanged.

use crate::api::Schedule;
use std::sync::Arc;

/// Backend extension point for normalizing imported payloads.
pub trait ScheduleImportAdapter: Send + Sync {
    /// Stable adapter identifier for logging and diagnostics.
    fn name(&self) -> &'static str;

    /// Validate the raw payload before any async processing starts.
    ///
    /// Implementations can enforce adapter-specific schema requirements.
    fn validate_schedule_payload(&self, _raw_payload: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// Parse and normalize a raw payload into the canonical schedule model.
    fn parse_schedule(&self, raw_payload: &str) -> anyhow::Result<Schedule>;

    /// Parse and normalize a JSON value into the canonical schedule model.
    ///
    /// The default implementation re-serialises the value and delegates to
    /// [`Self::parse_schedule`]; adapters that already know how to consume a
    /// `serde_json::Value` directly should override this to skip the
    /// intermediate string round-trip (this is on the hot path for bulk
    /// imports).
    fn parse_schedule_value(&self, value: &serde_json::Value) -> anyhow::Result<Schedule> {
        let raw = serde_json::to_string(value)?;
        self.parse_schedule(&raw)
    }

    /// Parse the payload but skip any expensive astronomy work whose result
    /// the caller intends to overwrite immediately (typically `dark_periods`,
    /// `astronomical_nights`, and per-block `visibility_periods`).
    ///
    /// The default implementation simply delegates to [`Self::parse_schedule_value`];
    /// adapters that compute these fields themselves (e.g. via solar/lunar
    /// sweeps per item) should override this to short-circuit those passes.
    /// This is called by the bulk-import handler after the first item has
    /// produced an environment-level preschedule cache, so subsequent items
    /// only need their structural fields populated.
    fn parse_schedule_value_structural(
        &self,
        value: &serde_json::Value,
    ) -> anyhow::Result<Schedule> {
        self.parse_schedule_value(value)
    }
}

/// Built-in adapter for TSI's native JSON format.
#[derive(Debug, Default)]
pub struct NativeScheduleImportAdapter;

impl ScheduleImportAdapter for NativeScheduleImportAdapter {
    fn name(&self) -> &'static str {
        "tsi-native-json"
    }

    fn validate_schedule_payload(&self, raw_payload: &str) -> anyhow::Result<()> {
        crate::models::schedule::validate_schedule_json_str(raw_payload)
    }

    fn parse_schedule(&self, raw_payload: &str) -> anyhow::Result<Schedule> {
        crate::models::schedule::parse_schedule_json_str(raw_payload)
    }
}

/// Build the default import adapter used by the HTTP server.
pub fn default_schedule_import_adapter() -> Arc<dyn ScheduleImportAdapter> {
    Arc::new(NativeScheduleImportAdapter)
}

#[cfg(test)]
mod tests {
    use super::{NativeScheduleImportAdapter, ScheduleImportAdapter};

    #[test]
    fn native_adapter_parses_valid_schedule_json() {
        let adapter = NativeScheduleImportAdapter;
        let schedule_json = r#"{
            "name": "native-fixture",
            "geographic_location": {
                "lat_deg": 28.7624,
                "lon_deg": -17.8892,
                "height": 2396.0
            },
            "blocks": [
                {
                    "id": 1,
                    "original_block_id": "block-1",
                    "target_ra": 158.03,
                    "target_dec": -68.03,
                    "constraints": {
                        "min_alt": 60.0,
                        "max_alt": 90.0,
                        "min_az": 0.0,
                        "max_az": 360.0,
                        "fixed_time": null
                    },
                    "priority": 8.5,
                    "min_observation": 3600.0,
                    "requested_duration": 7200.0,
                    "visibility_periods": [],
                    "scheduled_period": null
                }
            ]
        }"#;

        let parsed = adapter.parse_schedule(schedule_json).unwrap();

        assert_eq!(parsed.name, "native-fixture");
        assert_eq!(parsed.blocks.len(), 1);
    }

    #[test]
    fn native_adapter_validates_payload() {
        let adapter = NativeScheduleImportAdapter;

        adapter
            .validate_schedule_payload(
                r#"{
                    "name": "native-fixture",
                    "geographic_location": {
                        "lat_deg": 28.7624,
                        "lon_deg": -17.8892,
                        "height": 2396.0
                    },
                    "blocks": []
                }"#,
            )
            .expect("native payload validation should pass");
    }

    #[test]
    fn native_adapter_rejects_invalid_schedule_json() {
        let adapter = NativeScheduleImportAdapter;
        let err = adapter
            .validate_schedule_payload(r#"{"missing":"blocks"}"#)
            .unwrap_err();

        assert!(err.to_string().contains("Missing required 'blocks' field"));
    }
}
