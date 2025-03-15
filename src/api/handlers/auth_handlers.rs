#[cfg(feature = "web-api")]
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::auth::{
    models::{OAuthConfig, UserStore},
    oauth::{exchange_code_for_token, generate_oauth_state, get_authorization_url},
};

#[derive(Clone)]
pub struct AppState {
    pub users: UserStore,
    pub oauth_config: OAuthConfig,
    pub auth_states: Arc<tokio::sync::Mutex<HashMap<String, String>>>, // user_id -> state
}

// Simple HTML login page
pub async fn login_page() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Login to Dishwasher Monitor</title>
            <style>
                body { font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 2rem; }
                button { padding: 0.5rem 1rem; background: #4285f4; color: white; border: none; cursor: pointer; }
            </style>
        </head>
        <body>
            <h1>Dishwasher Monitor</h1>
            <p>Login with your Google account to monitor your smart devices.</p>
            <a href="/auth/authorize"><button>Sign in with Google</button></a>
        </body>
        </html>
        "#,
    )
}

// Start OAuth flow
pub async fn start_oauth(
    State(app_state): State<AppState>,
) -> Result<Redirect, (StatusCode, String)> {
    // For simplicity, we're using a random ID as user ID
    // In a real app, you'd have a proper user management system
    let user_id = uuid::Uuid::new_v4().to_string();
    
    // Generate and store state parameter for CSRF protection
    let state = generate_oauth_state();
    
    {
        let mut states = app_state.auth_states.lock().await;
        states.insert(user_id.clone(), state.clone());
    }
    
    // Generate authorization URL
    let auth_url = get_authorization_url(&app_state.oauth_config, &state);
    
    Ok(Redirect::to(&auth_url))
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

// Handle OAuth callback
pub async fn oauth_callback(
    State(app_state): State<AppState>,
    Query(params): Query<CallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Check for error in the callback
    if let Some(error) = params.error {
        return Err((StatusCode::BAD_REQUEST, format!("OAuth error: {}", error)));
    }
    
    // Get the code and state params
    let code = params.code.ok_or((StatusCode::BAD_REQUEST, "Missing code parameter".to_string()))?;
    let state = params.state.ok_or((StatusCode::BAD_REQUEST, "Missing state parameter".to_string()))?;
    
    // Find the user ID that matches this state
    let user_id = {
        let states = app_state.auth_states.lock().await;
        let mut matching_user_id = None;
        
        for (uid, saved_state) in states.iter() {
            if *saved_state == state {
                matching_user_id = Some(uid.clone());
                break;
            }
        }
        
        matching_user_id.ok_or((StatusCode::BAD_REQUEST, "Invalid state parameter".to_string()))?
    };
    
    // Exchange code for token
    let token = exchange_code_for_token(&app_state.oauth_config, &code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Token exchange error: {}", e)))?;
    
    // Clean up the state
    {
        let mut states = app_state.auth_states.lock().await;
        states.remove(&user_id);
    }
    
    // Store the user's token temporarily (in a real app, you'd use a database)
    let project_id = std::env::var("GOOGLE_PROJECT_ID")
        .unwrap_or_else(|_| "YOUR_PROJECT_ID".to_string());
    
    // Initialize user but without camera selection
    {
        let mut users_lock = app_state.users.lock().await;
        users_lock.insert(
            user_id.clone(),
            crate::auth::models::UserConfig {
                user_id: user_id.clone(),
                device_ids: Vec::new(), // No devices selected yet
                token,
                project_id: project_id.clone(),
            },
        );
    }
    
    // In a real app, you'd now:
    // 1. Create a proper session for the user
    // 2. Store the tokens more securely
    // 3. Redirect to a device selection page
    
    // Return success with user ID and link to camera selection
    Ok(Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Authorization Successful</title>
            <style>
                body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 2rem; }}
                button {{ padding: 0.5rem 1rem; background: #4285f4; color: white; border: none; cursor: pointer; margin-top: 1rem; }}
                pre {{ background: #f4f4f4; padding: 1rem; border-radius: 4px; overflow-x: auto; }}
            </style>
        </head>
        <body>
            <h1>Authorization Successful!</h1>
            <p>Your User ID: <code>{}</code></p>
            <p>You have been authorized to access your Google Nest devices.</p>
            
            <h2>Next Steps:</h2>
            <p>1. View your available cameras:</p>
            <pre>GET /devices/{}/cameras</pre>
            
            <p>2. Select cameras to monitor:</p>
            <p>Make a POST request to <code>/auth/register</code> with:</p>
            <pre>{{
  "user_id": "{}",
  "project_id": "{}",
  "device_ids": ["camera-id-1", "camera-id-2"]
}}</pre>
            
            <p>Use the API to manage your devices and monitoring settings.</p>
            
            <a href="/docs" target="_blank"><button>View API Documentation</button></a>
        </body>
        </html>
        "#,
        user_id, user_id, user_id, project_id
    )))
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub user_id: String,
    pub project_id: String,
    pub device_ids: Vec<String>,
}

// Register a user with their devices
pub async fn register_user(
    State(app_state): State<AppState>,
    Json(request): Json<RegisterUserRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    let user_id = request.user_id;
    let project_id = request.project_id;
    let device_ids = request.device_ids;
    let device_count = device_ids.len(); // Store count before we move device_ids
    
    // Get existing user config
    let user_config = {
        let users_lock = app_state.users.lock().await;
        users_lock.get(&user_id).cloned()
    };
    
    // If user doesn't exist or token is missing, return error
    if user_config.is_none() {
        return Err((StatusCode::NOT_FOUND, "User not found. Please authenticate first.".to_string()));
    }
    
    // Update user config with selected devices
    {
        let mut users_lock = app_state.users.lock().await;
        if let Some(config) = users_lock.get_mut(&user_id) {
            config.device_ids = device_ids.clone();
            config.project_id = project_id.clone();
        }
    }
    
    // Start a monitoring task for this user
    let token_clone = user_config.as_ref().unwrap().token.clone();
    let oauth_config_clone = app_state.oauth_config.clone();
    let users_clone = Arc::clone(&app_state.users);
    
    // Create user config to pass to monitoring task
    let monitoring_config = crate::auth::models::UserConfig {
        user_id: user_id.clone(),
        device_ids,
        token: token_clone,
        project_id,
    };
    
    // Spawn monitoring task
    tokio::spawn(async move {
        crate::monitor_user_cameras(
            monitoring_config,
            users_clone,
            oauth_config_clone,
        )
        .await;
    });
    
    Ok(Json(format!("User {} registered with {} devices. Monitoring started.", user_id, device_count)))
}
