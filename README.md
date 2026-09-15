# warden-backend

Off-chain services for [Warden](https://github.com/Osok-Labs/warden-contracts) smart accounts: a
relayer, an indexer, and a public API. None of these hold authorization privilege over user
accounts — see [Trust boundary](#trust-boundary) below.

> **Status: minimal indexer running against testnet.** `indexer/` polls Soroban RPC `getEvents` for
> the contract IDs in `warden-contracts/deployments/testnet.json` and serves what it's seen over a
> small HTTP API — see [Running the indexer](#running-the-indexer). No relayer or persistent
> storage yet; see [Status](#status) for what's deliberately deferred.

## Services

Deployable independently or together, depending on scale needs.

| Service | Role |
|---|---|
| **Relayer** | Holds the `executor` role on the deployed `fee-forwarder`. Accepts a signed `forward()` authorization tree from a client, fills in the real `fee_amount` (≤ the client's signed cap), submits the transaction, pays the network's XLM fee itself. |
| **Indexer** | Subscribes to contract events (`AccountCreated`, `SessionInstalled`/`Enforced`, the `PendingAction` events from `approval-delay-policy`, etc.), stores them queryable by account address. Backing store for anything the frontend needs that isn't a cheap direct RPC read: transaction history, "which policies are on this account," approval-queue status. |
| **API** | The public HTTP/RPC surface the frontend talks to. Wraps the indexer's data, exposes a batched "get everything about this account" view, and is the notification point for the approval-delay flow — when a `propose` event lands, this is what a co-approver's client polls or subscribes to. |

## Status

| Piece | State |
|---|---|
| `indexer/` (event polling) | ✅ Polls `getEvents` for the configured contract IDs, decodes XDR topics/values, holds them in memory |
| `indexer/` (HTTP surface) | ✅ `GET /health`, `GET /events` — combined into the same process as the poller for now, not a separate `api/` crate |
| Persistent storage | ⬜ In-memory only; restart loses history |
| Per-account filtering | ⬜ Returns all indexed events; meaningful account-scoped queries wait on `account-factory` producing more than one account to distinguish |
| Relayer | ⬜ Not started — nothing to sponsor until accounts beyond the one demo exist |

`indexer` and `api` are deliberately combined into one binary for this first milestone — splitting
them into the target layout's separate crates is mechanical once there's an actual reason
(independent scaling, on-call ownership), not a redesign. See `indexer/src/main.rs`'s module doc.

## Running the indexer

```sh
cd indexer
cargo run
# GET http://localhost:8080/health
# GET http://localhost:8080/events
```

Defaults to Stellar testnet and the contract IDs recorded in
`warden-contracts/deployments/testnet.json` (`threshold-policy` and the demo
`warden-smart-account`). Override via env vars: `WARDEN_RPC_URL`, `WARDEN_CONTRACT_IDS`
(comma-separated), `WARDEN_POLL_INTERVAL_SECS`, `WARDEN_INITIAL_LOOKBACK_LEDGERS`,
`WARDEN_LISTEN_ADDR`.

## Trust boundary

The backend is explicitly **untrusted for authorization purposes**. The relayer can withhold or
delay a transaction but cannot forge one — every meaningful authorization happens in the signed
payload the account's `__check_auth` validates on-chain, not in backend logic. The backend holds
real privilege in exactly one place: the relayer's `executor` credential, which is an operational
key-custody question, not a contract-level trust assumption. Custody lives in this repo's
deployment config (secrets manager, never committed).

## Repository structure

```
warden-backend/
├── indexer/    # ✅ event polling + HTTP API, combined for now (see Status above)
├── relayer/    # forward() submission service -- not started
└── shared/     # generated contract bindings, common types -- not started
```

One repo with internal services to start; splitting into separate repos later (if one needs
independent scaling or on-call ownership) doesn't require an interface change.

## Interfaces this repo consumes

| From `warden-contracts` | Format |
|---|---|
| Deployed addresses per network | JSON, versioned, published per release |
| Contract specs | Soroban XDR → generated TS/Rust bindings |
| Event schemas | Derived from `#[contractevent]` definitions |

Depend on the generated bindings package rather than hand-writing XDR encode/decode — that removes
a whole class of "backend and contract silently drift" bugs. Not wired up yet: the indexer decodes
event XDR generically via `stellar-xdr`'s `ScVal` (`Debug`-formatted, not schema-aware) rather than
through per-contract generated bindings — fine for "what happened," not yet for typed method calls.

## Interfaces this repo publishes

Account state, transaction history, and approval-queue status to
[`warden-frontend`](https://github.com/Osok-Labs/warden-frontend), over a versioned REST/RPC API —
the one interface here that isn't directly generated from the contracts, so it needs its own
explicit versioning discipline.

## Open question

Indexer backfill strategy: support historical backfill from genesis, or only track forward from
the indexer's own start time? Affects what "transaction history" completeness claims the API can
make to the frontend. No urgency — decide before the first real deployment, not before this repo
exists.

## Suggested next step

Persistent storage (even just sqlite) so indexed history survives a restart, plus per-account
event filtering once `warden-contracts` ships `account-factory` and there's more than one account
to distinguish.

## Related repos

Part of the Warden project — [`warden-contracts`](https://github.com/Osok-Labs/warden-contracts)
(the Soroban contracts this repo indexes and relays for) and
[`warden-frontend`](https://github.com/Osok-Labs/warden-frontend) (the client that consumes this
repo's API).

## License

[MIT](./LICENSE)
