use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status":"ok"})))
}
