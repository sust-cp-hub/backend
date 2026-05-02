use crate::app_state::AppState;
use crate::handlers::auth_handler;
use axum::{routing::post, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth_handler::register))
        .route("/login", post(auth_handler::login))
        .route("/verify-otp", post(auth_handler::verify_otp_handler))
        .route("/resend-otp", post(auth_handler::resend_otp_handler))
}
