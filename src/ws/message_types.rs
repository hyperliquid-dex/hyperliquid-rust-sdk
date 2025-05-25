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

#[derive(Deserialize, Clone, Debug)]
pub struct UserFundings {
    pub data: UserFundingsData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserNonFundingLedgerUpdates {
    pub data: UserNonFundingLedgerUpdatesData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Notification {
    pub data: NotificationData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WebData2 {
    pub data: WebData2Data,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ActiveAssetCtx {
    pub data: ActiveAssetCtxData,
}
