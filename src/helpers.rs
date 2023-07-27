use chrono::prelude::Utc;

pub(crate) fn now_timestamp_ms() -> u64 {
    let now = Utc::now();
    now.timestamp_millis() as u64
}

pub(crate) fn float_to_int_for_hashing(num: f64) -> u64 {
    (num * 100_000_000.0).round() as u64
}

pub(crate) fn float_to_string_for_hashing(num: f64) -> String {
    let num = format!("{:0>9}", float_to_int_for_hashing(num).to_string());
    format!("{}.{}", &num[..num.len() - 8], &num[num.len() - 8..])
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ChainType {
    Arbitrum,
    HyperliquidMainnet,
    HyperliquidTestnet,
}
