use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::ranker::RankerResponse;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub results_cache: Arc<Mutex<HashMap<String, RankerResponse>>>,
}
