use crate::app_state::AppState;
use crate::handlers::auth_handler;
use axum::{routing::post, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth_handler::register))
        .route("/login", post(auth_handler::login))
}
