use crate::app_state::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};

// health check which verifies server + db are alive
pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let db_status = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
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
