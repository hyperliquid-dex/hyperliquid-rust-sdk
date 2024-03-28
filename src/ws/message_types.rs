use crate::ws::sub_structs::*;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Trades {
    pub data: Vec<Trade>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct L2Book {
    pub data: L2BookData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AllMids {
    pub data: AllMidsData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct User {
    pub data: UserData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserFills {
    pub data: UserFillsData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Candle {
    pub data: CandleData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OrderUpdates {
    pub data: Vec<OrderUpdate>,
}
