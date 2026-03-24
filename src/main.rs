use axum::{Router, routing::get, Json};
use serde_json::{json, Value};
use sqlx::postgres::PgPool;
use tower_http::cors::{CorsLayer, Any};
use http::Method;

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

    // cors setup for frontend
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

// health check — verifies server + db are alive
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
