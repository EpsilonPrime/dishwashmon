#[cfg(feature = "web-api")]
use axum::{
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::api::handlers::*;
use crate::api::handlers::auth_handlers::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegisterRequest {
    pub email: String,
    pub password: String,
}

#[cfg(feature = "web-api")]
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", get(auth_handlers::login_page))
        .route("/auth/authorize", get(auth_handlers::start_oauth))
        .route("/auth/callback", get(auth_handlers::oauth_callback))
        .route("/auth/register", post(auth_handlers::register_user))
}
