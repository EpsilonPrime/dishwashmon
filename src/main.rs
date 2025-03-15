use dotenv::dotenv;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NestToken {
    access_token: String,
    expires_in: u64,
    token_type: String,
    refresh_token: String, // Important for long-running operations
}

#[derive(Debug, Deserialize)]
struct CameraEvent {
    event_id: String,
    event_type: String,
    timestamp: String,
    device_id: String,
    // Additional fields based on Google Nest API response
}

#[derive(Debug, Clone)]
struct UserConfig {
    user_id: String,
    device_ids: Vec<String>,
    token: NestToken,
    project_id: String,
}

// Store user configurations and their tokens
type UserStore = Arc<Mutex<HashMap<String, UserConfig>>>;

async fn refresh_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<NestToken, Box<dyn Error + Send + Sync>> {
    let client = Client::new();

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let res = client
        .post("https://www.googleapis.com/oauth2/v4/token")
        .form(&params)
        .send()
        .await?
        .json::<NestToken>()
        .await?;

    Ok(res)
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
                eprintln!("Error polling device {}: {}", device_id, e);
            }
        }
    }

    Ok(all_events)
}

async fn process_event(event: &CameraEvent, user_id: &str) {
    match event.event_type.as_str() {
        "motion" => {
            println!(
                "Motion detected on camera {} for user {} at {}",
                event.device_id, user_id, event.timestamp
            );
            // Your custom logic here - could be different per user
        }
        "person" => {
            println!(
                "Person detected on camera {} for user {} at {}",
                event.device_id, user_id, event.timestamp
            );
            // Your custom logic here
        }
        // Add other event types
        _ => println!("Unhandled event type: {}", event.event_type),
    }
}

async fn monitor_user_cameras(
    user_config: UserConfig,
    users: UserStore,
    client_id: String,
    client_secret: String,
) {
    let user_id = user_config.user_id.clone();

    loop {
        // Check if token needs refresh
        // In a real app, you'd check expiration time

        // Poll for events
        let current_config = {
            let users_lock = users.lock().await;
            users_lock.get(&user_id).cloned()
        };

        if let Some(config) = current_config {
            match poll_camera_events(&config).await {
                Ok(events) => {
                    for event in events {
                        process_event(&event, &user_id).await;
                    }
                }
                Err(e) => {
                    eprintln!("Error polling events for user {}: {}", user_id, e);

                    // If unauthorized, try refreshing token
                    if e.to_string().contains("401") {
                        match refresh_token(&client_id, &client_secret, &config.token.refresh_token)
                            .await
                        {
                            Ok(new_token) => {
                                let mut users_lock = users.lock().await;
                                if let Some(user_config) = users_lock.get_mut(&user_id) {
                                    user_config.token = new_token;
                                    println!("Refreshed token for user {}", user_id);
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to refresh token for user {}: {}", user_id, e);
                            }
                        }
                    }
                }
            }
        } else {
            // User was removed while we were running
            break;
        }

        sleep(Duration::from_secs(15)).await;
    }
}

async fn add_user(
    users: &UserStore,
    user_id: String,
    token: NestToken,
    device_ids: Vec<String>,
    project_id: String,
) {
    let mut users_lock = users.lock().await;
    users_lock.insert(
        user_id.clone(),
        UserConfig {
            user_id,
            device_ids,
            token,
            project_id,
        },
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let client_id = env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID not set");
    let client_secret = env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET not set");

    let users: UserStore = Arc::new(Mutex::new(HashMap::new()));

    // In a real app, you'd load these from a database
    // This is just a simplified example

    // Spawn a task for each user
    let mut handles = Vec::new();

    {
        let users_lock = users.lock().await;
        for (_, config) in users_lock.iter() {
            let user_config = config.clone();
            let users_clone = Arc::clone(&users);
            let client_id_clone = client_id.clone();
            let client_secret_clone = client_secret.clone();

            let handle = tokio::spawn(async move {
                monitor_user_cameras(
                    user_config,
                    users_clone,
                    client_id_clone,
                    client_secret_clone,
                )
                .await;
            });

            handles.push(handle);
        }
    }

    // Here you'd implement your API or web interface to register new users
    // When a new user connects:
    // 1. Get their OAuth consent
    // 2. Create a token
    // 3. Call add_user()
    // 4. Spawn a new monitoring task

    // Wait for all tasks to complete (they won't in a real app)
    for handle in handles {
        handle.await?;
    }

    Ok(())
}
