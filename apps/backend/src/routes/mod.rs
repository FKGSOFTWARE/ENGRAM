mod cards;
mod ingest;
mod llm;
mod review;
mod ws;

use axum::{routing::get, Router};
use std::sync::Arc;

use crate::AppState;

pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/cards", cards::router())
        .nest("/review", review::router())
        .nest("/ingest", ingest::router())
        .nest("/llm", llm::router())
        .route("/ws", get(ws::ws_handler))
}
