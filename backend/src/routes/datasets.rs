/// Dataset management endpoints
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::path::PathBuf;

use crate::{
    loaders,
    models::api::{DatasetListResponse, DatasetResponse, ErrorResponse},
    state::AppState,
};

/// Upload and load a CSV file
pub async fn upload_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Extract the file from multipart form data
    let mut file_data = Vec::new();
    let mut filename = String::from("uploaded.csv");

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(name) = field.file_name() {
            filename = name.to_string();
        }
        
        if let Ok(data) = field.bytes().await {
            file_data = data.to_vec();
        }
    }

    if file_data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No file data provided".to_string(),
                details: None,
            }),
        )
            .into_response();
    }

    // Write to temp file (Polars reads from file path)
    let temp_path = std::env::temp_dir().join(&filename);
    if let Err(e) = std::fs::write(&temp_path, &file_data) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to write temp file".to_string(),
                details: Some(e.to_string()),
            }),
        )
            .into_response();
    }

    // Load CSV
    let blocks = match loaders::load_csv(&temp_path) {
        Ok(b) => b,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Failed to parse CSV".to_string(),
                    details: Some(e.to_string()),
                }),
            )
                .into_response();
        }
    };

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    // Store in state
    let metadata = match state.load_dataset(blocks, filename) {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to store dataset".to_string(),
                    details: Some(e),
                }),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(DatasetResponse {
            metadata,
            message: "Dataset loaded successfully".to_string(),
        }),
    )
        .into_response()
}

/// Load the sample dataset from data/schedule.csv
pub async fn load_sample(State(state): State<AppState>) -> impl IntoResponse {
    // Path to sample data (relative to project root)
    let sample_path = PathBuf::from("../data/schedule.csv");
    
    let blocks = match loaders::load_csv(&sample_path) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Failed to load sample dataset".to_string(),
                    details: Some(e.to_string()),
                }),
            )
                .into_response();
        }
    };

    let metadata = match state.load_dataset(blocks, "schedule.csv (sample)".to_string()) {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to store dataset".to_string(),
                    details: Some(e),
                }),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(DatasetResponse {
            metadata,
            message: "Sample dataset loaded successfully".to_string(),
        }),
    )
        .into_response()
}

/// Get current dataset metadata
pub async fn get_current_metadata(State(state): State<AppState>) -> impl IntoResponse {
    match state.get_metadata() {
        Ok(Some(metadata)) => (StatusCode::OK, Json(metadata)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "No dataset loaded".to_string(),
                details: None,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to get metadata".to_string(),
                details: Some(e),
            }),
        )
            .into_response(),
    }
}

/// Get current dataset with all blocks
pub async fn get_current_dataset(State(state): State<AppState>) -> impl IntoResponse {
    match state.get_dataset() {
        Ok(Some((blocks, metadata))) => (
            StatusCode::OK,
            Json(DatasetListResponse { blocks, metadata }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "No dataset loaded".to_string(),
                details: None,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to get dataset".to_string(),
                details: Some(e),
            }),
        )
            .into_response(),
    }
}

/// Clear the current dataset
pub async fn clear_dataset(State(state): State<AppState>) -> impl IntoResponse {
    match state.clear_dataset() {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"message": "Dataset cleared"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to clear dataset".to_string(),
                details: Some(e),
            }),
        )
            .into_response(),
    }
}
