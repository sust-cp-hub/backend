use axum::{Router, routing::get, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use tower_http::cors::{CorsLayer, Any};
use http::Method;

// ---------- models ----------

// maps to the users table in neon
#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub user_id: i32,
    pub reg_number: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)] // used not to expose password in responses
    pub password: String,
    pub vjudge_handle: Option<String>,
    pub codeforces_handle: Option<String>,
    pub is_admin: Option<bool>,
    pub status: Option<String>,
}

// maps to the contests table
#[derive(Debug, FromRow, Serialize)]
pub struct Contest {
    pub contest_no: i32,
    pub title: String,
    pub contest_link: String,
    pub contest_date: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

// maps to the announcements table
#[derive(Debug, FromRow, Serialize)]
pub struct Announcement {
    pub post_id: i32,
    pub author_id: Option<i32>,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub event_date: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

// ---------- input structs (for POST/PUT requests) ----------

#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub reg_number: String,
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateContest {
    pub title: String,
    pub contest_link: String,
    pub contest_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContest {
    pub title: Option<String>,
    pub contest_link: Option<String>,
    pub contest_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnnouncement {
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub event_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnnouncement {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub event_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfile {
    pub name: Option<String>,
    pub vjudge_handle: Option<String>,
    pub codeforces_handle: Option<String>,
}

#[tokio::main]
async fn main() {
    // load env file
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // db pool for neon postgres
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("connected to database");

    // setup for frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .with_state(pool)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    tracing::info!("server running at http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}

// health check which verifies server + db are alive
async fn health_check(
    axum::extract::State(pool): axum::extract::State<PgPool>,
) -> Json<Value> {
    let db_status = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await;

    match db_status {
        Ok(_) => Json(json!({
            "status": "ok",
            "database": "connected"
        })),
        Err(e) => {
            tracing::error!("db health check failed: {}", e);
            Json(json!({
                "status": "error",
                "database": "disconnected"
            }))
        }
    }
}
