use crate::auth::models::{AuthorizationResponse, NestToken, OAuthConfig};
use reqwest::Client;
use serde::Serialize;
use std::error::Error;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Invalid state parameter, possible CSRF attack")]
    InvalidState,
    
    #[error("Missing authorization code")]
    MissingCode,
    
    #[error("OAuth error: {0}")]
    OAuthError(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// Generate a state parameter for OAuth flow and store it
pub fn generate_oauth_state() -> String {
    Uuid::new_v4().to_string()
}

/// Generate the authorization URL
pub fn get_authorization_url(config: &OAuthConfig, state: &str) -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
        config.auth_uri,
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(&config.scope),
        urlencoding::encode(state)
    )
}

/// Exchange authorization code for tokens
pub async fn exchange_code_for_token(
    config: &OAuthConfig,
    code: &str,
) -> Result<NestToken, AuthError> {
    let client = Client::new();
    
    #[derive(Serialize)]
    struct TokenRequest<'a> {
        client_id: &'a str,
        client_secret: &'a str,
        code: &'a str,
        grant_type: &'a str,
        redirect_uri: &'a str,
    }
    
    let token_request = TokenRequest {
        client_id: &config.client_id,
        client_secret: &config.client_secret,
        code,
        grant_type: "authorization_code",
        redirect_uri: &config.redirect_uri,
    };
    
    let res = client
        .post(&config.token_uri)
        .form(&token_request)
        .send()
        .await?;
    
    if !res.status().is_success() {
        let error_text = res.text().await?;
        return Err(AuthError::OAuthError(error_text));
    }
    
    let mut token = res.json::<NestToken>().await?;
    token.created_at = chrono::Utc::now();
    
    Ok(token)
}

/// Refresh an expired token
pub async fn refresh_token(
    config: &OAuthConfig,
    refresh_token: &str,
) -> Result<NestToken, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    
    #[derive(Serialize)]
    struct RefreshRequest<'a> {
        client_id: &'a str,
        client_secret: &'a str,
        refresh_token: &'a str,
        grant_type: &'a str,
    }
    
    let refresh_request = RefreshRequest {
        client_id: &config.client_id,
        client_secret: &config.client_secret,
        refresh_token,
        grant_type: "refresh_token",
    };
    
    let res = client
        .post(&config.token_uri)
        .form(&refresh_request)
        .send()
        .await?;
    
    let mut token = res.json::<NestToken>().await?;
    token.created_at = chrono::Utc::now();
    
    Ok(token)
}

/// Validate the authentication response
pub fn validate_oauth_response(
    response: &AuthorizationResponse,
    expected_state: &str,
) -> Result<String, AuthError> {
    // Verify state to prevent CSRF
    if response.state != expected_state {
        return Err(AuthError::InvalidState);
    }
    
    // Verify code is present
    if response.code.is_empty() {
        return Err(AuthError::MissingCode);
    }
    
    Ok(response.code.clone())
}
