use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NestToken {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub refresh_token: String,
    #[serde(skip, default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

impl NestToken {
    pub fn is_expired(&self) -> bool {
        let expiry = self.created_at + chrono::Duration::seconds(self.expires_in as i64);
        // Consider token expired if it has less than 5 minutes left
        expiry <= Utc::now() + chrono::Duration::minutes(5)
    }
}

#[derive(Debug, Clone)]
pub struct UserConfig {
    pub user_id: String,
    pub device_ids: Vec<String>,
    pub token: NestToken,
    pub project_id: String,
}

// Store user configurations and their tokens
pub type UserStore = Arc<Mutex<HashMap<String, UserConfig>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: String,
    pub auth_uri: String,
    pub token_uri: String,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: String::new(),
            scope: "https://www.googleapis.com/auth/sdm.service".to_string(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
        }
    }
}
