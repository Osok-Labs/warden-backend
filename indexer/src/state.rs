use std::sync::Arc;

use serde::Serialize;
use tokio::sync::Mutex;

/// A contract event, decoded into a form worth serving over HTTP. `topic`
/// and `value` are the XDR `ScVal`s' `Debug` representation, not a fully
/// polished pretty-printer -- honest for a first milestone: it turns opaque
/// base64 blobs into something a human (or the frontend, for now) can read,
/// without taking on writing a full ScVal-to-JSON converter that duplicates
/// what `stellar events --output json` already does for ad hoc inspection.
#[derive(Debug, Clone, Serialize)]
pub struct IndexedEvent {
    pub id: String,
    pub ledger: u32,
    pub ledger_closed_at: String,
    pub contract_id: String,
    pub topic: Vec<String>,
    pub value: String,
}

#[derive(Default)]
pub struct IndexerState {
    pub events: Vec<IndexedEvent>,
    /// Pagination cursor from the last successful `getEvents` call; `None`
    /// until the first poll runs.
    pub cursor: Option<String>,
    pub latest_ledger: u32,
}

pub type AppState = Arc<Mutex<IndexerState>>;
