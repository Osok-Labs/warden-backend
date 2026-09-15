use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use serde_json::json;

use crate::state::{AppState, IndexedEvent};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/events", get(list_events))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.lock().await;
    Json(json!({
        "status": "ok",
        "indexedEvents": guard.events.len(),
        "latestLedger": guard.latest_ledger,
    }))
}

#[derive(Serialize)]
struct EventsResponse {
    events: Vec<IndexedEvent>,
    #[serde(rename = "latestLedger")]
    latest_ledger: u32,
}

/// Returns every indexed event across all configured contracts. No
/// per-account filtering yet -- with a single demo account and no
/// account-factory, "which account does this belong to" isn't a meaningful
/// query yet; add it once there's more than one account to distinguish.
async fn list_events(State(state): State<AppState>) -> Json<EventsResponse> {
    let guard = state.lock().await;
    Json(EventsResponse {
        events: guard.events.clone(),
        latest_ledger: guard.latest_ledger,
    })
}
