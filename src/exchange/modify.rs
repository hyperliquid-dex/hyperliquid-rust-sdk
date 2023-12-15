use crate::{
    prelude::*, ClientOrderRequest, Error, helpers::float_to_int_for_hashing,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::order::OrderRequest;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModifyRequest {
    pub oid: u64,
    pub order: OrderRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientModifyRequest {
    pub oid: u64,
    pub order: ClientOrderRequest,
}

impl ClientModifyRequest {
    pub(crate) fn convert(self, coin_to_asset: &HashMap<String, u32>) -> Result<ModifyRequest> {
        let order = self.order.convert(coin_to_asset)?;
        Ok(ModifyRequest {
            oid: self.oid,
            order,
        })
    }

    pub(crate) fn create_hashable_tuple(
        &self,
        coin_to_asset: &HashMap<String, u32>,
    ) -> Result<(u64, u32, bool, u64, u64, bool, u8, u64)> {
        let hashed_order_type = self.order.order_type.get_type()?;
        let &asset = coin_to_asset.get(&self.order.asset).ok_or(Error::AssetNotFound)?;
        
        Ok(( 
            self.oid,
            asset,
            self.order.is_buy,
            float_to_int_for_hashing(self.order.limit_px),
            float_to_int_for_hashing(self.order.sz),
            self.order.reduce_only,
            hashed_order_type.0,
            hashed_order_type.1,
        ))
    }

    pub(crate) fn create_hashable_tuple_with_cloid(
        &self,
        coin_to_asset: &HashMap<String, u32>,
    ) -> Result<(u64, u32, bool, u64, u64, bool, u8, u64, [u8; 16])> {
        let hashed_order_type = self.order.order_type.get_type()?;
        let &asset = coin_to_asset.get(&self.order.asset).ok_or(Error::AssetNotFound)?;
        
        // cloid is Some("0x1234567890abcdef1234567890abcdef".to_string()) is a 128 hex string
        if let Some(cloid) = &self.order.cloid {
            let hashed_cloid: [u8;16] = u128::from_str_radix(&cloid[2..], 16).unwrap().to_be_bytes();
            Ok((
                self.oid,
                asset,
                self.order.is_buy,
                float_to_int_for_hashing(self.order.limit_px),
                float_to_int_for_hashing(self.order.sz),
                self.order.reduce_only,
                hashed_order_type.0,
                hashed_order_type.1,
                hashed_cloid,
            ))
        } else {
            Err(Error::NoCloid)
        }
    }
}