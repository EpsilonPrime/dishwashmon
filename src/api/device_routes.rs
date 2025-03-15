#[cfg(feature = "web-api")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};

use crate::api::handlers::auth_handlers::AppState;
use crate::devices::discovery;

#[derive(serde::Serialize)]
pub struct DeviceListResponse {
    devices: Vec<discovery::Device>,
}

#[cfg(feature = "web-api")]
pub fn device_routes() -> Router<AppState> {
    Router::new()
        .route("/devices/:user_id", get(list_devices))
        .route("/devices/:user_id/cameras", get(list_cameras))
}

// List all devices for a user
async fn list_devices(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<DeviceListResponse>, (StatusCode, String)> {
    // Get user config from the store
    let user_config = {
        let users_lock = app_state.users.lock().await;
        users_lock.get(&user_id).cloned()
    };

    let user_config = match user_config {
        Some(config) => config,
        None => return Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    };

    // Fetch devices
    match discovery::discover_devices(&user_config.project_id, &user_config.token).await {
        Ok(devices) => Ok(Json(DeviceListResponse { devices })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to discover devices: {}", e),
        )),
    }
}

// List only cameras for a user
async fn list_cameras(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<DeviceListResponse>, (StatusCode, String)> {
    // Get user config from the store
    let user_config = {
        let users_lock = app_state.users.lock().await;
        users_lock.get(&user_id).cloned()
    };

    let user_config = match user_config {
        Some(config) => config,
        None => return Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    };

    // Fetch devices and filter for cameras
    match discovery::discover_devices(&user_config.project_id, &user_config.token).await {
        Ok(devices) => {
            let cameras = discovery::filter_cameras(&devices);
            Ok(Json(DeviceListResponse { devices: cameras }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to discover devices: {}", e),
        )),
    }
}