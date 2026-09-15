//! Minimal warden-backend indexer: polls Soroban RPC `getEvents` for the
//! configured contract IDs and serves what it's seen over a small HTTP API.
//!
//! Deliberately combines what the target architecture (see repo README)
//! splits into separate `indexer`/`api` services -- there's no reason yet
//! (independent scaling, on-call ownership) to pay for two processes, and
//! splitting later is mechanical, not a redesign. `relayer` isn't started at
//! all: nothing to sponsor until accounts exist beyond this one demo.
//!
//! State is in-memory only and reset on restart. That's an acknowledged gap
//! for a first milestone, not an oversight -- see the repo README's
//! "Status" section for what's next.

mod api;
mod config;
mod poller;
mod state;

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{config::Config, state::IndexerState};

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let state: state::AppState = Arc::new(Mutex::new(IndexerState::default()));

    println!(
        "warden-indexer: watching {} contract(s) on {}",
        config.contract_ids.len(),
        config.rpc_url
    );

    tokio::spawn(poller::run(config.clone(), state.clone()));

    let listen_addr = config.listen_addr.clone();
    let app = api::router(state);
    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {listen_addr}: {err}"));

    println!("warden-indexer: serving on http://{listen_addr}");
    axum::serve(listener, app)
        .await
        .expect("indexer HTTP server crashed");
}
