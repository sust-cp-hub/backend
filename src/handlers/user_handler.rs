use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::models::user::{UpdateProfile, User};
use crate::utils::jwt::Claims;

// get my profile
pub async fn get_me(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
        .bind(claims.user_id)
        .fetch_optional(&state.pool)
        .await?;

    let user = user.ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": user
    })))
}

// update my profile (name, vjudge_handle, codeforces_handle)
pub async fn update_me(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<UpdateProfile>,
) -> Result<Json<Value>, AppError> {
    let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
        .bind(claims.user_id)
        .fetch_optional(&state.pool)
        .await?;

    let existing = existing.ok_or(AppError::NotFound("User not found".to_string()))?;

    // merge: use new value if provided, keep existing if not
    let new_name = body.name.unwrap_or(existing.name);
    let new_vjudge = match body.vjudge_handle {
        Some(handle) => Some(handle),
        None => existing.vjudge_handle,
    };
    let new_codeforces = match body.codeforces_handle {
        Some(handle) => Some(handle),
        None => existing.codeforces_handle,
    };

    let user = sqlx::query_as::<_, User>(
        r#"UPDATE users
           SET name = $1, vjudge_handle = $2, codeforces_handle = $3
           WHERE user_id = $4
           RETURNING *"#,
    )
    .bind(&new_name)
    .bind(&new_vjudge)
    .bind(&new_codeforces)
    .bind(claims.user_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "message": "Profile updated successfully",
        "data": user
    })))
}
