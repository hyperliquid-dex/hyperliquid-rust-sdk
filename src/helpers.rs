use chrono::prelude::Utc;

pub(crate) fn get_timestamp_ms() -> u64 {
    let now = Utc::now();
    now.timestamp_millis() as u64
}

#[derive(Copy, Clone)]
pub(crate) enum ChainType {
    L1,
    Mainnet,
    Testnet,
}
