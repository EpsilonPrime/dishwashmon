use crate::auth::models::UserConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StoredUserData {
    pub users: HashMap<String, UserConfig>,
}

pub type UserStore = Arc<Mutex<HashMap<String, UserConfig>>>;

// Saves user data to a JSON file
pub async fn save_user_data(users: &UserStore, file_path: &str) -> io::Result<()> {
    let users_lock = users.lock().await;
    let data = StoredUserData {
        users: users_lock.clone(),
    };
    
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    let serialized = serde_json::to_string_pretty(&data)?;
    let mut file = File::create(file_path)?;
    file.write_all(serialized.as_bytes())?;
    
    Ok(())
}

// Loads user data from a JSON file
pub async fn load_user_data(file_path: &str) -> io::Result<UserStore> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Ok(Arc::new(Mutex::new(HashMap::new())));
    }
    
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let data: StoredUserData = serde_json::from_str(&contents)?;
    Ok(Arc::new(Mutex::new(data.users)))
}

// Saves user data periodically
pub async fn start_periodic_save(users: UserStore, file_path: String, interval: std::time::Duration) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            if let Err(e) = save_user_data(&users, &file_path).await {
                log::error!("Failed to save user data: {}", e);
            } else {
                log::info!("User data saved successfully");
            }
        }
    });
}