# warden-backend

Off-chain services for [Warden](https://github.com/Osok-Labs/warden-contracts) smart accounts: a
relayer, an indexer, and a public API. None of these hold authorization privilege over user
accounts — see [Trust boundary](#trust-boundary) below.

> **Status: design phase, no code yet.** Depends on `warden-contracts` shipping deployed addresses
> and generated bindings before there's anything real to build against. See
> [Suggested next step](#suggested-next-step).

## Services

Deployable independently or together, depending on scale needs.

| Service | Role |
|---|---|
| **Relayer** | Holds the `executor` role on the deployed `fee-forwarder`. Accepts a signed `forward()` authorization tree from a client, fills in the real `fee_amount` (≤ the client's signed cap), submits the transaction, pays the network's XLM fee itself. |
| **Indexer** | Subscribes to contract events (`AccountCreated`, `SessionInstalled`/`Enforced`, the `PendingAction` events from `approval-delay-policy`, etc.), stores them queryable by account address. Backing store for anything the frontend needs that isn't a cheap direct RPC read: transaction history, "which policies are on this account," approval-queue status. |
| **API** | The public HTTP/RPC surface the frontend talks to. Wraps the indexer's data, exposes a batched "get everything about this account" view, and is the notification point for the approval-delay flow — when a `propose` event lands, this is what a co-approver's client polls or subscribes to. |

## Trust boundary

The backend is explicitly **untrusted for authorization purposes**. The relayer can withhold or
delay a transaction but cannot forge one — every meaningful authorization happens in the signed
payload the account's `__check_auth` validates on-chain, not in backend logic. The backend holds
real privilege in exactly one place: the relayer's `executor` credential, which is an operational
key-custody question, not a contract-level trust assumption. Custody lives in this repo's
deployment config (secrets manager, never committed).

## Repository structure (target)

```
warden-backend/
├── relayer/    # forward() submission service
├── indexer/    # event subscription + storage
├── api/        # public HTTP surface over indexer + relayer status
└── shared/     # generated contract bindings, common types
```

One repo with three internal services to start; splitting into separate repos later (if one needs
independent scaling or on-call ownership) doesn't require an interface change.

## Interfaces this repo consumes

| From `warden-contracts` | Format |
|---|---|
| Deployed addresses per network | JSON, versioned, published per release |
| Contract specs | Soroban XDR → generated TS/Rust bindings |
| Event schemas | Derived from `#[contractevent]` definitions |

Depend on the generated bindings package rather than hand-writing XDR encode/decode — that removes
a whole class of "backend and contract silently drift" bugs.

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

Minimal indexer only, no relayer yet (nothing to sponsor until an account exists on-chain),
pointed at whatever `warden-contracts` produces from its invariant-check prototype deployment.

## Related repos

Part of the Warden project — [`warden-contracts`](https://github.com/Osok-Labs/warden-contracts)
(the Soroban contracts this repo indexes and relays for) and
[`warden-frontend`](https://github.com/Osok-Labs/warden-frontend) (the client that consumes this
repo's API).

## License

TBD — see [`warden-contracts`](https://github.com/Osok-Labs/warden-contracts) for the current
licensing discussion; this repo will follow suit.
