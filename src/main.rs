mod auth;
mod api;

use auth::models::{NestToken, OAuthConfig, UserConfig, UserStore};
use dotenv::dotenv;
use log::info;
use reqwest::{header, Client};
use serde::Deserialize;
use std::{collections::HashMap, env, error::Error, sync::Arc};
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

#[derive(Debug, Deserialize)]
struct CameraEvent {
    event_id: String,
    event_type: String,
    timestamp: String,
    device_id: String,
    // Additional fields based on Google Nest API response
}

async fn poll_camera_events(
    user_config: &UserConfig,
) -> Result<Vec<CameraEvent>, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", user_config.token.access_token))?,
    );

    let mut all_events = Vec::new();

    // Poll each device for the user
    for device_id in &user_config.device_ids {
        let url = format!(
            "https://smartdevicemanagement.googleapis.com/v1/enterprises/{}/devices/{}/events",
            user_config.project_id, device_id
        );

        match client.get(&url).headers(headers.clone()).send().await {
            Ok(response) => {
                if let Ok(events) = response.json::<Vec<CameraEvent>>().await {
                    all_events.extend(events);
                }
            }
            Err(e) => {
                log::error!("Error polling device {}: {}", device_id, e);
            }
        }
    }

    Ok(all_events)
}

async fn process_event(event: &CameraEvent, user_id: &str) {
    match event.event_type.as_str() {
        "motion" => {
            info!(
                "Motion detected on camera {} for user {} at {}",
                event.device_id, user_id, event.timestamp
            );
            // Your custom logic here - could be different per user
        }
        "person" => {
            info!(
                "Person detected on camera {} for user {} at {}",
                event.device_id, user_id, event.timestamp
            );
            // Your custom logic here
        }
        // Add other event types
        _ => info!("Unhandled event type: {}", event.event_type),
    }
}

async fn monitor_user_cameras(
    user_config: UserConfig,
    users: UserStore,
    oauth_config: OAuthConfig, 
) {
    let user_id = user_config.user_id.clone();

    loop {
        // Get current user config
        let current_config = {
            let users_lock = users.lock().await;
            users_lock.get(&user_id).cloned()
        };

        if let Some(mut config) = current_config {
            // Check if token needs refresh
            if config.token.is_expired() {
                log::info!("Token expired for user {}, refreshing", user_id);
                match auth::oauth::refresh_token(&oauth_config, &config.token.refresh_token).await {
                    Ok(new_token) => {
                        let mut users_lock = users.lock().await;
                        if let Some(user_config) = users_lock.get_mut(&user_id) {
                            user_config.token = new_token;
                            log::info!("Refreshed token for user {}", user_id);
                            config = user_config.clone();
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to refresh token for user {}: {}", user_id, e);
                    }
                }
            }

            // Poll for events
            match poll_camera_events(&config).await {
                Ok(events) => {
                    for event in events {
                        process_event(&event, &user_id).await;
                    }
                }
                Err(e) => {
                    log::error!("Error polling events for user {}: {}", user_id, e);

                    // If unauthorized, try refreshing token
                    if e.to_string().contains("401") {
                        match auth::oauth::refresh_token(&oauth_config, &config.token.refresh_token).await {
                            Ok(new_token) => {
                                let mut users_lock = users.lock().await;
                                if let Some(user_config) = users_lock.get_mut(&user_id) {
                                    user_config.token = new_token;
                                    log::info!("Refreshed token for user {}", user_id);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to refresh token for user {}: {}", user_id, e);
                            }
                        }
                    }
                }
            }
        } else {
            // User was removed while we were running
            log::info!("User {} was removed, stopping monitoring", user_id);
            break;
        }

        sleep(Duration::from_secs(15)).await;
    }
}

pub async fn add_user(
    users: &UserStore,
    user_id: String,
    token: NestToken,
    device_ids: Vec<String>,
    project_id: String,
) {
    let mut users_lock = users.lock().await;
    
    let user_config = UserConfig {
        user_id: user_id.clone(),
        device_ids,
        token,
        project_id,
    };
    
    users_lock.insert(user_id, user_config);
}

fn setup_logging() {
    env_logger::init_from_env(
        env_logger::Env::default().default_filter_or("info")
    );
}

#[cfg(feature = "web-api")]
async fn start_web_server(
    users: UserStore,
    oauth_config: OAuthConfig,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use axum::Router;
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::trace::TraceLayer;
    
    // Create app state for the web server
    let app_state = api::handlers::auth_handlers::AppState {
        users: Arc::clone(&users),
        oauth_config,
        auth_states: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    };
    
    // Start the web server
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);
    
    // Get routes and add state and middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = api::auth_routes::auth_routes()
        .with_state(app_state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    
    let addr = SocketAddr::from(([0, 0, 0, 0], server_port));
    log::info!("Server listening on {}", addr);
    
    // Run the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize .env file and logging
    dotenv().ok();
    setup_logging();
    
    // Create OAuth configuration
    let oauth_config = OAuthConfig {
        client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID not set"),
        client_secret: env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET not set"),
        redirect_uri: env::var("REDIRECT_URI").unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string()),
        ..Default::default()
    };
    
    log::info!("Starting dishwasher monitor service");
    
    // Create user store
    let users: UserStore = Arc::new(Mutex::new(HashMap::new()));
    
    // Handle web API if the feature is enabled
    #[cfg(feature = "web-api")]
    {
        log::info!("Starting web server for authentication");
        let users_clone = Arc::clone(&users);
        let oauth_config_clone = oauth_config.clone();
        
        tokio::spawn(async move {
            if let Err(e) = start_web_server(users_clone, oauth_config_clone).await {
                log::error!("Web server error: {}", e);
            }
        });
    }
    
    // In a real app, you'd load existing users from a database
    // For now we'll just monitor the empty user list
    
    // Start monitoring tasks for any existing users
    let mut handles = Vec::new();
    
    {
        let users_lock = users.lock().await;
        for (_, config) in users_lock.iter() {
            let user_config = config.clone();
            let users_clone = Arc::clone(&users);
            let oauth_config_clone = oauth_config.clone();
            
            let handle = tokio::spawn(async move {
                monitor_user_cameras(
                    user_config,
                    users_clone,
                    oauth_config_clone,
                )
                .await;
            });
            
            handles.push(handle);
        }
    }
    
    // Keep the main task running indefinitely
    log::info!("Monitoring service running. Press Ctrl+C to exit.");
    
    // Wait for all tasks to complete (they won't in a real app)
    // In a real app, you'd want to implement a proper shutdown mechanism
    for handle in handles {
        handle.await?;
    }
    
    // This point will never be reached in a real app
    // as the program should run continuously
    Ok(())
}
