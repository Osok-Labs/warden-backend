use std::time::Duration;

use stellar_rpc_client::{Client, EventStart, EventType};
use stellar_xdr::{Limits, ReadXdr, ScVal};

use crate::{
    config::Config,
    state::{AppState, IndexedEvent},
};

/// Polls `getEvents` for the configured contract IDs on a fixed interval and
/// appends newly seen events to shared state. Runs forever; errors from a
/// single poll are logged and retried next tick rather than crashing the
/// process -- a transient RPC hiccup shouldn't take the whole indexer down.
pub async fn run(config: Config, state: AppState) {
    let client = match Client::new(&config.rpc_url) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("indexer: failed to build RPC client: {err}");
            return;
        }
    };

    loop {
        if let Err(err) = poll_once(&client, &config, &state).await {
            eprintln!("indexer: poll failed: {err}");
        }
        tokio::time::sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}

async fn poll_once(
    client: &Client,
    config: &Config,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = {
        let guard = state.lock().await;
        match &guard.cursor {
            Some(cursor) => EventStart::Cursor(cursor.clone()),
            None => {
                let latest = client.get_latest_ledger().await?;
                let start_ledger = latest
                    .sequence
                    .saturating_sub(config.initial_lookback_ledgers)
                    .max(1);
                EventStart::Ledger(start_ledger)
            }
        }
    };

    let response = client
        .get_events(
            start,
            Some(EventType::Contract),
            &config.contract_ids,
            &[],
            Some(100),
        )
        .await?;

    let mut guard = state.lock().await;
    guard.latest_ledger = response.latest_ledger;
    guard.cursor = Some(response.cursor);

    for event in response.events {
        let topic = event
            .topic
            .iter()
            .map(|t| decode_scval(t))
            .collect::<Vec<_>>();
        let value = decode_scval(&event.value);

        guard.events.push(IndexedEvent {
            id: event.id,
            ledger: event.ledger,
            ledger_closed_at: event.ledger_closed_at,
            contract_id: event.contract_id,
            topic,
            value,
        });
    }

    Ok(())
}

/// Decodes a base64-XDR `ScVal` into its `Debug` string. Not a polished
/// pretty-printer (see `state::IndexedEvent`'s doc comment) -- if decoding
/// fails for any reason, falls back to the raw base64 rather than dropping
/// the event, since a partially-readable event still beats a silently
/// missing one.
fn decode_scval(raw: &str) -> String {
    match ScVal::from_xdr_base64(raw, Limits::none()) {
        Ok(scval) => format!("{scval:?}"),
        Err(_) => raw.to_string(),
    }
}
