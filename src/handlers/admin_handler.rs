use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::app_state::AppState;
use crate::models::user::User;
use crate::utils::jwt::Claims;

#[derive(Debug, Deserialize)]
pub struct UserFilter {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusUpdateInput {
    pub reason: Option<String>,
}

// helper guard to instantly block non-admins
fn require_admin(claims: &Claims) -> Result<(), (StatusCode, Json<Value>)> {
    if !claims.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "error": "Admin access required"
            })),
        ));
    }
    Ok(())
}

// get all users, or filter by status like ?status=pending
pub async fn admin_list_users(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<UserFilter>,
) -> (StatusCode, Json<Value>) {
    // block if not admin
    if let Err(response) = require_admin(&claims) {
        return response;
    }

    // find users based on the query parameter
    let result = match &filter.status {
        Some(status) => {
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE status = $1 ORDER BY user_id DESC")
                .bind(status)
                .fetch_all(&state.pool)
                .await
        }
        None => {
            sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY user_id DESC")
                .fetch_all(&state.pool)
                .await
        }
    };

    match result {
        Ok(users) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "count": users.len(),
                "data": users
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"success": false, "error": format!("Database error: {}", e)})),
        ),
    }
}

// get a single user's detailed profile
pub async fn admin_get_user(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    if let Err(response) = require_admin(&claims) {
        return response;
    }

    let result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    match result {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": user
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"success": false, "error": "User not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"success": false, "error": format!("Database error: {}", e)})),
        ),
    }
}

// approve a user so they can log in
pub async fn admin_approve_user(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    if let Err(response) = require_admin(&claims) {
        return response;
    }

    // make sure user exists
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"success": false, "error": "User not found"})),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": format!("Database error: {}", e)})),
            );
        }
    };

    // they must be pending to be approved
    if user.status.as_deref() != Some("pending") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": format!("Cannot approve user with status '{:?}'", user.status)
            })),
        );
    }

    // update to active
    let result = sqlx::query_as::<_, User>(
        "UPDATE users SET status = 'active' WHERE user_id = $1 RETURNING *"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(updated) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": format!("User '{}' has been approved", updated.name),
                "data": updated
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"success": false, "error": format!("Failed to approve: {}", e)})),
        ),
    }
}

// reject a pending user
pub async fn admin_reject_user(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<StatusUpdateInput>,
) -> (StatusCode, Json<Value>) {
    if let Err(response) = require_admin(&claims) {
        return response;
    }

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"success": false, "error": "User not found"}))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"success": false, "error": format!("Database error: {}", e)}))),
    };

    if user.status.as_deref() != Some("pending") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": format!("Cannot reject user with status '{:?}'", user.status)
            })),
        );
    }

    // move status to rejected
    let result = sqlx::query_as::<_, User>(
        "UPDATE users SET status = 'rejected' WHERE user_id = $1 RETURNING *"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(updated) => {
            let message = match &body.reason {
                Some(reason) => format!("User '{}' rejected. Reason: {}", updated.name, reason),
                None => format!("User '{}' has been rejected", updated.name),
            };
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": message,
                    "data": updated
                })),
            )
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"success": false, "error": format!("Failed to reject: {}", e)}))),
    }
}

// ban an already active user
pub async fn admin_ban_user(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<StatusUpdateInput>,
) -> (StatusCode, Json<Value>) {
    if let Err(response) = require_admin(&claims) {
        return response;
    }

    // safety measure to stop admins locking themselves out
    if claims.user_id == id {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "error": "You cannot ban yourself"})),
        );
    }

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"success": false, "error": "User not found"}))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"success": false, "error": format!("Database error: {}", e)}))),
    };

    if user.status.as_deref() == Some("rejected") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "error": "User is already rejected/banned"})),
        );
    }

    let result = sqlx::query_as::<_, User>(
        "UPDATE users SET status = 'rejected' WHERE user_id = $1 RETURNING *"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(updated) => {
            let message = match &body.reason {
                Some(reason) => format!("User '{}' banned. Reason: {}", updated.name, reason),
                None => format!("User '{}' has been banned", updated.name),
            };
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": message,
                    "data": updated
                })),
            )
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"success": false, "error": format!("Failed to ban: {}", e)}))),
    }
}
