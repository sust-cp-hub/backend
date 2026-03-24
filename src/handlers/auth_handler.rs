use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::app_state::AppState;
use crate::models::user::{LoginInput, RegisterInput, User};
use crate::utils::jwt::create_token;

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterInput>,
) -> Json<Value> {
    // checking if the email is already in the db
    let existing = sqlx::query_scalar::<_, i32>("SELECT user_id FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.pool)
        .await;

    if let Ok(Some(_)) = existing {
        return Json(json!({
            "success": false,
            "error": "email already registered"
        }));
    }

    // hashing password with argon2
    let salt = SaltString::generate(&mut OsRng);
    let hashed = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .expect("failed to hash password")
        .to_string();

    // automatically activate only sust students, others will be approved later by admin
    let status = if body.email.ends_with("@student.sust.edu") {
        "active"
    } else {
        "pending"
    };

    // saving the new user to neon postgres
    let result = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (reg_number, name, email, password, status) VALUES ($1, $2, $3, $4, $5) RETURNING user_id"
    )
    .bind(&body.reg_number)
    .bind(&body.name)
    .bind(&body.email)
    .bind(&hashed)
    .bind(status)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(user_id) => Json(json!({
            "success": true,
            "user_id": user_id,
            "status": status,
            "message": if status == "active" {
                "registered successfully"
            } else {
                "registered — pending for admin approval"
            }
        })),
        Err(e) => {
            tracing::error!("registration failed: {}", e);
            Json(json!({
                "success": false,
                "error": "registration failed — email or reg number may already exist"
            }))
        }
    }
}

pub async fn login(State(state): State<AppState>, Json(body): Json<LoginInput>) -> Json<Value> {
    // find user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.pool)
        .await;

    let user = match user {
        Ok(Some(u)) => u,
        _ => {
            return Json(json!({
                "success": false,
                "error": "invalid email or password"
            }))
        }
    };

    match user.status.as_deref() {
        Some("pending") => {
            return Json(json!({
                "success": false,
                "error": "account pending approval"
            }))
        }
        Some("rejected") => {
            return Json(json!({
                "success": false,
                "error": "account has been rejected"
            }))
        }
        _ => {}
    }

    // verify the provided password against hashed password
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(h) => h,
        Err(_) => {
            return Json(json!({
                "success": false,
                "error": "invalid email or password"
            }))
        }
    };

    let is_valid = Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Json(json!({
            "success": false,
            "error": "invalid email or password"
        }));
    }

    // if valid, generate jwt token
    let token = match create_token(user.user_id, &user.email, user.is_admin.unwrap_or(false)) {
        Ok(t) => t,
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": e
            }))
        }
    };

    return Json(json!({
        "success": true,
        "token": token,
        "user": {
            "user_id": user.user_id,
            "name": user.name,
            "email": user.email,
            "is_admin": user.is_admin
        }
    }));
}
