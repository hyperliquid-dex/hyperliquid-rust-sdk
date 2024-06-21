use crate::{
    info::{AssetPosition, Level, MarginSummary},
    PerpetualAssetContext, Universe, UniverseItem,
};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpetualsAssetContextsResponse([PerpetualsAssetContexts; 2]);

impl PerpetualsAssetContextsResponse {
    /// Consumes the response and returns universe.
    pub fn into_universe(self) -> Universe {
        match self.0[0].clone() {
            PerpetualsAssetContexts::Universe(u) => u,
            _ => panic!("Expected universe."),
        }
    }

    /// Consumes the response and returns asset contexts.
    pub fn into_asset_contexts(self) -> Vec<PerpetualAssetContext> {
        match self.0[1].clone() {
            PerpetualsAssetContexts::AssetContexts(ac) => ac,
            _ => panic!("Expected asset contexts."),
        }
    }

    /// Consumes the response and returns an array of tuples containing an universe item and asset contexts.
    /// Each element of universe should match each element of asset contexts.
    pub fn into_universe_and_asset_contexts(self) -> Vec<(UniverseItem, PerpetualAssetContext)> {
        let universe = match self.0[0].clone() {
            PerpetualsAssetContexts::Universe(u) => u,
            _ => panic!("Expected universe."),
        };
        let asset_contexts = match self.0[1].clone() {
            PerpetualsAssetContexts::AssetContexts(ac) => ac,
            _ => panic!("Expected asset contexts."),
        };
        universe
            .universe
            .into_iter()
            .zip(asset_contexts.into_iter())
            .collect()
    }

    /// Returns a reference to the universe.
    pub fn universe(&self) -> &Universe {
        match &self.0[0] {
            PerpetualsAssetContexts::Universe(u) => u,
            _ => panic!("Expected universe."),
        }
    }

    /// Returns a reference to the asset contexts.
    pub fn asset_contexts(&self) -> &Vec<PerpetualAssetContext> {
        match &self.0[1] {
            PerpetualsAssetContexts::AssetContexts(ac) => ac,
            _ => panic!("Expected asset contexts."),
        }
    }

    /// Returns a reference to an array of tuples containing an universe item and asset contexts.
    /// Each element of universe should match each element of asset contexts.
    pub fn universe_and_asset_contexts(&self) -> Vec<(&UniverseItem, &PerpetualAssetContext)> {
        let universe = match &self.0[0] {
            PerpetualsAssetContexts::Universe(u) => u,
            _ => panic!("Expected universe."),
        };
        let asset_contexts = match &self.0[1] {
            PerpetualsAssetContexts::AssetContexts(ac) => ac,
            _ => panic!("Expected asset contexts."),
        };
        universe
            .universe
            .iter()
            .zip(asset_contexts.iter())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum PerpetualsAssetContexts {
    Universe(Universe),
    AssetContexts(Vec<PerpetualAssetContext>),
}

impl<'de> Deserialize<'de> for PerpetualsAssetContexts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;
        if value.is_object() {
            Ok(PerpetualsAssetContexts::Universe(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            ))
        } else if value.is_array() {
            Ok(PerpetualsAssetContexts::AssetContexts(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            ))
        } else {
            Err(serde::de::Error::custom("Expected object or array"))
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserStateResponse {
    pub asset_positions: Vec<AssetPosition>,
    pub cross_margin_summary: MarginSummary,
    pub margin_summary: MarginSummary,
    pub withdrawable: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrdersResponse {
    pub coin: String,
    pub limit_px: String,
    pub oid: u64,
    pub side: String,
    pub sz: String,
    pub timestamp: u64,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFillsResponse {
    pub closed_pnl: String,
    pub coin: String,
    pub crossed: bool,
    pub dir: String,
    pub hash: String,
    pub oid: u64,
    pub px: String,
    pub side: String,
    pub start_position: String,
    pub sz: String,
    pub time: u64,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FundingHistoryResponse {
    pub coin: String,
    pub funding_rate: String,
    pub premium: String,
    pub time: u64,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L2SnapshotResponse {
    pub coin: String,
    pub levels: Vec<Vec<Level>>,
    pub time: u64,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecentTradesResponse {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub time: u64,
    pub hash: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct CandlesSnapshotResponse {
    #[serde(rename = "t")]
    pub time_open: u64,
    #[serde(rename = "T")]
    pub time_close: u64,
    #[serde(rename = "s")]
    pub coin: String,
    #[serde(rename = "i")]
    pub candle_interval: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "v")]
    pub vlm: String,
    #[serde(rename = "n")]
    pub num_trades: u64,
}
