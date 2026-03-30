use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::models::user::{LoginInput, RegisterInput, User};
use crate::utils::jwt::create_token;
use crate::validation::{validate_email, validate_string};

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // validate inputs upfront
    validate_string(&body.name, "Name", 2, 100)?;
    validate_string(&body.reg_number, "Registration number", 5, 50)?;
    validate_string(&body.password, "Password", 6, 255)?;
    validate_email(&body.email)?;

    // check if email already exists
    let existing = sqlx::query_scalar::<_, i32>("SELECT user_id FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.pool)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // hash password with argon2
    let salt = SaltString::generate(&mut OsRng);
    let hashed = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?
        .to_string();

    // auto-activate sust students, others pending
    let status = if body.email.ends_with("@student.sust.edu") {
        "active"
    } else {
        "pending"
    };

    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (reg_number, name, email, password, status) VALUES ($1, $2, $3, $4, $5) RETURNING user_id",
    )
    .bind(&body.reg_number)
    .bind(&body.name)
    .bind(&body.email)
    .bind(&hashed)
    .bind(status)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "user_id": user_id,
            "status": status,
            "message": if status == "active" {
                "registered successfully"
            } else {
                "registered — pending for admin approval"
            }
        })),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginInput>,
) -> Result<Json<Value>, AppError> {
    // find user by email — use vague error to prevent user enumeration
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.pool)
        .await?;

    let user = user.ok_or(AppError::Unauthorized(
        "Invalid email or password".to_string(),
    ))?;

    // check account status
    match user.status.as_deref() {
        Some("pending") => {
            return Err(AppError::Unauthorized(
                "Account pending approval".to_string(),
            ))
        }
        Some("rejected") => {
            return Err(AppError::Unauthorized(
                "Account has been rejected".to_string(),
            ))
        }
        _ => {}
    }

    // verify password
    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let is_valid = Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    // generate jwt token
    let token = create_token(user.user_id, &user.email, user.is_admin.unwrap_or(false))
        .map_err(|e| AppError::InternalError(e))?;

    Ok(Json(json!({
        "success": true,
        "token": token,
        "user": {
            "user_id": user.user_id,
            "name": user.name,
            "email": user.email,
            "is_admin": user.is_admin
        }
    })))
}
