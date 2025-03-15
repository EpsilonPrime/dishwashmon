#[cfg(feature = "web-api")]
use axum::{
    extract::{Query, State, Form},
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::collections::HashSet;

use crate::api::handlers::auth_handlers::AppState;
use crate::devices::discovery;
use crate::views;

#[cfg(feature = "web-api")]
pub fn web_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index_page))
        .route("/dashboard", get(dashboard_page))
        .route("/cameras/select", get(camera_selection))
        .route("/cameras/register", post(register_camera))
        .route("/cameras/unregister", post(unregister_camera))
}

// Index page handler
async fn index_page() -> Html<String> {
    Html(views::home_page())
}

// Query params for user ID
#[derive(Debug, Deserialize)]
struct UserIdQuery {
    user_id: Option<String>,
}

// Camera selection page handler
async fn camera_selection(
    State(app_state): State<AppState>,
    Query(params): Query<UserIdQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    // Get user ID from query params
    let user_id = match params.user_id {
        Some(id) => id,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Missing user_id parameter".to_string(),
            ))
        }
    };

    // Get user config
    let user_config = {
        let users_lock = app_state.users.lock().await;
        users_lock.get(&user_id).cloned()
    };

    let user_config = match user_config {
        Some(config) => config,
        None => {
            return Ok(Html(views::error_page(
                "User Not Found",
                "The user ID provided is not valid. Please authenticate again.",
            )))
        }
    };

    // Fetch camera list
    match discovery::discover_devices(&user_config.project_id, &user_config.token).await {
        Ok(all_devices) => {
            let cameras = discovery::filter_cameras(&all_devices);
            Ok(Html(views::camera_selection_page(&user_id, &cameras)))
        }
        Err(e) => {
            let error_message = format!("Failed to fetch cameras: {}", e);
            Ok(Html(views::error_page("Camera Error", &error_message)))
        }
    }
}

// Dashboard page handler
async fn dashboard_page(
    State(app_state): State<AppState>,
    Query(params): Query<UserIdQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    // Get user ID from query params
    let user_id = match params.user_id {
        Some(id) => id,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Missing user_id parameter".to_string(),
            ))
        }
    };

    // Get user config
    let user_config = {
        let users_lock = app_state.users.lock().await;
        users_lock.get(&user_id).cloned()
    };

    let user_config = match user_config {
        Some(config) => config,
        None => {
            return Ok(Html(views::error_page(
                "User Not Found",
                "The user ID provided is not valid. Please authenticate again.",
            )))
        }
    };

    // Fetch all devices to get details for the registered ones
    match discovery::discover_devices(&user_config.project_id, &user_config.token).await {
        Ok(all_devices) => {
            // Create a HashSet of registered device IDs for efficient lookup
            let registered_ids: HashSet<String> = user_config.device_ids.into_iter().collect();
            
            // Filter devices to only include registered ones
            let registered_cameras: Vec<_> = all_devices
                .into_iter()
                .filter(|device| registered_ids.contains(&device.device_id))
                .collect();
            
            Ok(Html(views::dashboard_page(&user_id, &registered_cameras)))
        }
        Err(e) => {
            let error_message = format!("Failed to fetch cameras: {}", e);
            Ok(Html(views::error_page("Camera Error", &error_message)))
        }
    }
}

// Form data for camera registration
#[derive(Debug, Deserialize)]
struct CameraForm {
    user_id: String,
    device_id: String,
}

// Register a camera
async fn register_camera(
    State(app_state): State<AppState>,
    Form(form): Form<CameraForm>,
) -> Result<Redirect, (StatusCode, String)> {
    let user_id = form.user_id;
    let device_id = form.device_id;
    
    // Get user config
    let mut update_successful = false;
    {
        let mut users_lock = app_state.users.lock().await;
        if let Some(config) = users_lock.get_mut(&user_id) {
            // Add the device ID if not already present
            if !config.device_ids.contains(&device_id) {
                config.device_ids.push(device_id.clone());
                update_successful = true;
            }
        }
    }
    
    // Redirect to dashboard
    if update_successful {
        Ok(Redirect::to(&format!("/dashboard?user_id={}", user_id)))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            "Failed to update user configuration".to_string(),
        ))
    }
}

// Unregister a camera
async fn unregister_camera(
    State(app_state): State<AppState>,
    Form(form): Form<CameraForm>,
) -> Result<Redirect, (StatusCode, String)> {
    let user_id = form.user_id;
    let device_id = form.device_id;
    
    // Get user config
    let mut update_successful = false;
    {
        let mut users_lock = app_state.users.lock().await;
        if let Some(config) = users_lock.get_mut(&user_id) {
            // Remove the device ID
            config.device_ids.retain(|id| id != &device_id);
            update_successful = true;
        }
    }
    
    // Redirect to dashboard
    if update_successful {
        Ok(Redirect::to(&format!("/dashboard?user_id={}", user_id)))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            "Failed to update user configuration".to_string(),
        ))
    }
}