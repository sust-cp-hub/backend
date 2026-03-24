use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::app_state::AppState;
use crate::models::user::User;
use crate::utils::jwt::Claims;

// get my profile. axum will only let this run if the user is logged in because of the `claims` parameter.
pub async fn get_me(claims: Claims, State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    // looking up the current user based on the id from their token
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
        .bind(claims.user_id)
        .fetch_optional(&state.pool)
        .await;

    match user {
        Ok(Some(u)) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": u
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "error": "User not found"
            })),
        ),
        Err(e) => {
            tracing::error!("database error fetching user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Internal server error"
                })),
            )
        }
    }
}
