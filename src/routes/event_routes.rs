use axum::{
    routing::{get, post, put},
    Router,
};
use crate::app_state::AppState;
use crate::handlers::event_handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Event CRUD
        .route("/", get(event_handler::get_events).post(event_handler::create_event))
        .route("/{id}", put(event_handler::update_event).delete(event_handler::delete_event))
        // Team CRUD under an event
        .route("/{event_id}/teams", post(event_handler::add_team))
        .route("/{event_id}/teams/{team_id}", put(event_handler::update_team).delete(event_handler::delete_team))
}
