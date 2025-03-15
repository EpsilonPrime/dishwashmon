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
    let _token = exchange_code_for_token(&app_state.oauth_config, &code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Token exchange error: {}", e)))?;
    
    // Clean up the state
    {
        let mut states = app_state.auth_states.lock().await;
        states.remove(&user_id);
    }
    
    // In a real app, you'd now:
    // 1. Create a session for the user
    // 2. Store the tokens securely
    // 3. Redirect to the main app
    
    // For now, we'll just return success with the user ID
    // In a real app, you would redirect to a welcome page
    Ok(Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Authorization Successful</title>
            <style>
                body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 2rem; }}
            </style>
        </head>
        <body>
            <h1>Authorization Successful!</h1>
            <p>Your User ID: {}</p>
            <p>You can now use this ID to configure your dishwasher monitoring.</p>
        </body>
        </html>
        "#,
        user_id
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
    State(_app_state): State<AppState>,
    Json(request): Json<RegisterUserRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    let _user_id = request.user_id;
    let _project_id = request.project_id;
    let _device_ids = request.device_ids;
    
    // In a real app, you'd verify the user exists and has valid tokens
    // For this example, we'll just return an error since we don't have actual token storage
    
    Err((
        StatusCode::NOT_IMPLEMENTED,
        "User registration not fully implemented - this would add the user to the monitoring system".to_string(),
    ))
}
