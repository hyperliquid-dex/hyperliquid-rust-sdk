use std::collections::HashMap;

use alloy::primitives::B128;
use ethers::abi::ethereum_types::H128;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct Meta {
    pub universe: Vec<AssetMeta>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct SpotMeta {
    pub universe: Vec<SpotAssetMeta>,
    pub tokens: Vec<TokenInfo>,
}

impl SpotMeta {
    pub fn add_pair_and_name_to_index_map(
        &self,
        mut coin_to_asset: HashMap<String, u32>,
    ) -> HashMap<String, u32> {
        let index_to_name: HashMap<usize, &str> = self
            .tokens
            .iter()
            .map(|info| (info.index, info.name.as_str()))
            .collect();

        for asset in self.universe.iter() {
            let spot_ind: u32 = 10000 + asset.index as u32;
            let name_to_ind = (asset.name.clone(), spot_ind);

            let Some(token_1_name) = index_to_name.get(&asset.tokens[0]) else {
                continue;
            };

            let Some(token_2_name) = index_to_name.get(&asset.tokens[1]) else {
                continue;
            };

            coin_to_asset.insert(format!("{token_1_name}/{token_2_name}"), spot_ind);
            coin_to_asset.insert(name_to_ind.0, name_to_ind.1);
        }

        coin_to_asset
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SpotMetaAndAssetCtxs {
    SpotMeta(SpotMeta),
    Context(Vec<SpotAssetContext>),
}

impl SpotMetaAndAssetCtxs {
    pub fn get_spot_meta(&self) -> &SpotMeta {
        match self {
            SpotMetaAndAssetCtxs::SpotMeta(meta) => meta,
            _ => panic!("Not a spot meta"),
        }
    }
    pub fn get_context(&self) -> &Vec<SpotAssetContext> {
        match self {
            SpotMetaAndAssetCtxs::Context(ctxs) => ctxs,
            _ => panic!("Not a context"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MetaAndAssetCtxs {
    Meta(Meta),
    Context(Vec<AssetContext>),
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetContext {
    pub day_ntl_vlm: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub prev_day_px: String,
    pub circulating_supply: String,
    pub coin: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetContext {
    pub day_ntl_vlm: String,
    pub funding: String,
    pub impact_pxs: Option<Vec<String>>,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub open_interest: String,
    pub oracle_px: String,
    pub premium: Option<String>,
    pub prev_day_px: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetMeta {
    pub name: String,
    pub sz_decimals: u32,
    pub max_leverage: usize,
    #[serde(default)]
    pub only_isolated: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetMeta {
    pub tokens: [usize; 2],
    pub name: String,
    pub index: usize,
    pub is_canonical: bool,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub name: String,
    pub sz_decimals: u8,
    pub wei_decimals: u8,
    pub index: usize,
    pub token_id: B128,
    pub is_canonical: bool,
    pub full_name: Option<String>,
}
