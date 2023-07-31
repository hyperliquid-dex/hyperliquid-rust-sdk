use crate::{
    consts::{EPSILON, INF_BPS},
    prelude::*,
    Error,
};
use chrono::prelude::Utc;
use rand::{thread_rng, Rng};

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

pub(crate) fn generate_random_key() -> Result<[u8; 32]> {
    let mut arr = [0u8; 32];
    thread_rng()
        .try_fill(&mut arr[..])
        .map_err(|e| Error::RandGen(e.to_string()))?;
    Ok(arr)
}

pub fn truncate_float(float: f64, decimals: u32, round_up: bool) -> f64 {
    let pow10 = 10i64.pow(decimals) as f64;
    let mut float = (float * pow10) as u64;
    if round_up {
        float += 1;
    }
    float as f64 / pow10
}

pub fn bps_diff(x: f64, y: f64) -> u16 {
    if x.abs() < EPSILON {
        INF_BPS
    } else {
        (((y - x).abs() / (x)) * 10_000.0) as u16
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ChainType {
    Arbitrum,
    HyperliquidMainnet,
    HyperliquidTestnet,
}
