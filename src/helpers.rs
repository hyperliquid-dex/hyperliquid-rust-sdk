use chrono::prelude::Utc;

pub(crate) fn now_timestamp_ms() -> u64 {
    let now = Utc::now();
    now.timestamp_millis() as u64
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ChainType {
    Arbitrum,
    HyperliquidMainnet,
    HyperliquidTestnet,
}
