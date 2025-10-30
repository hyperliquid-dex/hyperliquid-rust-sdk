use std::collections::HashMap;

use ethers::abi::ethereum_types::H128;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Meta {
    pub universe: Vec<AssetMeta>,
}

#[derive(Deserialize, Debug, Clone)]
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

            coin_to_asset.insert(format!("{}/{}", token_1_name, token_2_name), spot_ind);
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

#[derive(Deserialize, Debug, Clone)]
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
pub struct AssetMeta {
    pub name: String,
    pub sz_decimals: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetMeta {
    pub tokens: [usize; 2],
    pub name: String,
    pub index: usize,
    pub is_canonical: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub name: String,
    pub sz_decimals: u8,
    pub wei_decimals: u8,
    pub index: usize,
    pub token_id: H128,
    pub is_canonical: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PerpDexMeta {
    pub universe: Vec<AssetMeta>,
}

impl PerpDexMeta {
    /// Creates a mapping of perp names to their asset IDs for builder-deployed perps.
    ///
    /// Builder-deployed perps use the formula: 100000 + perp_dex_index * 10000 + index_in_meta
    ///
    /// # Arguments
    /// * `perp_dex_index` - The index of the perp dex (obtained from perpDexs endpoint)
    /// * `coin_to_asset` - Optional existing HashMap to extend with perp mappings
    ///
    /// # Example
    /// For test:ABC on testnet with perp_dex_index=1 and index_in_meta=0,
    /// the asset ID will be: 100000 + 1*10000 + 0 = 110000
    pub fn add_perp_to_asset_map(
        &self,
        perp_dex_index: usize,
        mut coin_to_asset: HashMap<String, u32>,
    ) -> HashMap<String, u32> {
        for (index_in_meta, asset) in self.universe.iter().enumerate() {
            let asset_id: u32 = 100000 + (perp_dex_index as u32 * 10000) + index_in_meta as u32;
            coin_to_asset.insert(asset.name.clone(), asset_id);
        }

        coin_to_asset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perp_dex_meta_parsing() {
        // Test parsing meta response for a perp dex (flxn example from testnet)
        let json = r#"{
            "universe": [
                {
                    "szDecimals": 2,
                    "name": "flxn:TSLA"
                }
            ]
        }"#;

        let meta: Meta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.universe.len(), 1);
        assert_eq!(meta.universe[0].name, "flxn:TSLA");
        assert_eq!(meta.universe[0].sz_decimals, 2);
    }

    #[test]
    fn test_perp_dex_meta_asset_id_calculation() {
        // Test the asset ID calculation formula
        // For test:ABC with perp_dex_index=1 and index_in_meta=0, asset should be 110000
        let perp_dex_meta = PerpDexMeta {
            universe: vec![
                AssetMeta {
                    name: "test:ABC".to_string(),
                    sz_decimals: 2,
                },
                AssetMeta {
                    name: "test:XYZ".to_string(),
                    sz_decimals: 3,
                },
            ],
        };

        let perp_dex_index = 1;
        let asset_map = perp_dex_meta.add_perp_to_asset_map(perp_dex_index, HashMap::new());

        // test:ABC is at index 0: 100000 + 1*10000 + 0 = 110000
        assert_eq!(asset_map.get("test:ABC"), Some(&110000));

        // test:XYZ is at index 1: 100000 + 1*10000 + 1 = 110001
        assert_eq!(asset_map.get("test:XYZ"), Some(&110001));
    }

    #[test]
    fn test_perp_dex_meta_multiple_indices() {
        // Test with different perp_dex_index values
        let perp_dex_meta = PerpDexMeta {
            universe: vec![AssetMeta {
                name: "xyz:XYZ100".to_string(),
                sz_decimals: 2,
            }],
        };

        // Test with perp_dex_index = 0
        let asset_map = perp_dex_meta.add_perp_to_asset_map(0, HashMap::new());
        assert_eq!(asset_map.get("xyz:XYZ100"), Some(&100000));

        // Test with perp_dex_index = 2
        let asset_map = perp_dex_meta.add_perp_to_asset_map(2, HashMap::new());
        assert_eq!(asset_map.get("xyz:XYZ100"), Some(&120000));
    }

    #[test]
    fn test_perp_dex_meta_extend_existing_map() {
        // Test that the method can extend an existing HashMap
        let perp_dex_meta = PerpDexMeta {
            universe: vec![AssetMeta {
                name: "test:ABC".to_string(),
                sz_decimals: 2,
            }],
        };

        let mut existing_map = HashMap::new();
        existing_map.insert("existing:COIN".to_string(), 99999);

        let asset_map = perp_dex_meta.add_perp_to_asset_map(1, existing_map);

        // Check that both the existing and new entries are present
        assert_eq!(asset_map.get("existing:COIN"), Some(&99999));
        assert_eq!(asset_map.get("test:ABC"), Some(&110000));
    }
}
