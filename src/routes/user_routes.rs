use crate::app_state::AppState;
use crate::handlers::user_handler;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/me", get(user_handler::get_me))
}
