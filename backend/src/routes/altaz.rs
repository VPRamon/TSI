use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltAzTargetRequest {
    pub original_block_id: String,
    pub block_name: String,
    pub priority: f64,
    pub target_ra_deg: f64,
    pub target_dec_deg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltAzObservatoryRequest {
    pub lon_deg: f64,
    pub lat_deg: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltAzRequest {
    pub observatory: AltAzObservatoryRequest,
    pub start_mjd: f64,
    pub end_mjd: f64,
    pub targets: Vec<AltAzTargetRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltAzCurve {
    pub original_block_id: String,
    pub block_name: String,
    pub priority: f64,
    pub altitudes_deg: Vec<f64>,
    pub azimuths_deg: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltAzData {
    pub schedule_id: i64,
    pub sample_times_mjd: Vec<f64>,
    pub curves: Vec<AltAzCurve>,
}

pub const GET_ALT_AZ_DATA: &str = "get_alt_az_data";
