/// Runtime configuration, read from environment variables so the same binary
/// can point at testnet, a local network, or (eventually) mainnet without a
/// rebuild. Defaults match the current testnet deployment recorded in
/// `warden-contracts/deployments/testnet.json` -- see that repo for the
/// address history; this repo only consumes it, never hardcodes trust in it
/// beyond a dev-friendly default.
#[derive(Clone)]
pub struct Config {
    pub rpc_url: String,
    /// Contract IDs (strkey, "C...") to index events for.
    pub contract_ids: Vec<String>,
    pub poll_interval_secs: u64,
    /// How many ledgers back from the current tip to start indexing on first
    /// boot. Testnet RPC only retains events for a limited recent window, so
    /// a large lookback just wastes the first poll finding nothing further
    /// back than the network already dropped.
    pub initial_lookback_ledgers: u32,
    pub listen_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            rpc_url: std::env::var("WARDEN_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string()),
            contract_ids: std::env::var("WARDEN_CONTRACT_IDS")
                .unwrap_or_else(|_| {
                    // threshold-policy, warden-smart-account-demo -- see
                    // warden-contracts/deployments/testnet.json
                    "CDNXK75JQ6XPPBB525IVZ57XWQZ5IM6ZB53HUCSCNZYZHUVM64QHKN4S,\
                     CCBXMG3RJUQUW6RNONDSLQUKL6P7YIYR6T56BCXLECMPA3TVCASAABOG"
                        .to_string()
                })
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            poll_interval_secs: std::env::var("WARDEN_POLL_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            initial_lookback_ledgers: std::env::var("WARDEN_INITIAL_LOOKBACK_LEDGERS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2000),
            listen_addr: std::env::var("WARDEN_LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        }
    }
}
