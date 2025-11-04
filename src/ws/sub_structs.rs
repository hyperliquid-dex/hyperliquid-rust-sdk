use std::collections::HashMap;

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::Leverage;

/// Order side: Bid (Buy) or Ask (Sell)
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    /// Bid / Buy
    B,
    /// Ask / Sell
    A,
}

/// Time-in-force options for orders
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Tif {
    /// Good-til-canceled: Remains active until filled or canceled
    Gtc,
    /// Immediate-or-cancel: Fills immediately or cancels unfilled portion
    Ioc,
    /// Add-liquidity-only: Only adds liquidity
    Alo,
    /// Frontend market order (similar to Ioc)
    FrontendMarket,
    /// Liquidation market order (similar to Ioc)
    LiquidationMarket,
}

/// Order type for market execution
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum OrderType {
    /// Executes immediately at market price
    Market,
    /// Executes at specified limit price or better
    Limit,
    /// Stop order that becomes market order
    #[serde(rename = "Stop Market")]
    StopMarket,
    /// Stop order that becomes limit order
    #[serde(rename = "Stop Limit")]
    StopLimit,
    /// Take profit order that becomes market order
    #[serde(rename = "Take Profit Market")]
    TakeProfitMarket,
    /// Take profit order that becomes limit order
    #[serde(rename = "Take Profit Limit")]
    TakeProfitLimit,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Trade {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub time: u64,
    pub hash: String,
    pub tid: u64,
    pub users: (String, String),
}

#[derive(Deserialize, Clone, Debug)]
pub struct BookLevel {
    pub px: String,
    pub sz: String,
    pub n: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct L2BookData {
    pub coin: String,
    pub time: u64,
    pub levels: Vec<Vec<BookLevel>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AllMidsData {
    pub mids: HashMap<String, String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TradeInfo {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub time: u64,
    pub hash: String,
    pub start_position: String,
    pub dir: String,
    pub closed_pnl: String,
    pub oid: u64,
    pub cloid: Option<String>,
    pub crossed: bool,
    pub fee: String,
    pub fee_token: String,
    pub tid: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFillsData {
    pub is_snapshot: Option<bool>,
    pub user: Address,
    pub fills: Vec<TradeInfo>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum UserData {
    Fills(Vec<TradeInfo>),
    Funding(UserFunding),
    Liquidation(Liquidation),
    NonUserCancel(Vec<NonUserCancel>),
}

#[derive(Deserialize, Clone, Debug)]
pub struct Liquidation {
    pub lid: u64,
    pub liquidator: String,
    pub liquidated_user: String,
    pub liquidated_ntl_pos: String,
    pub liquidated_account_value: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NonUserCancel {
    pub coin: String,
    pub oid: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CandleData {
    #[serde(rename = "T")]
    pub time_close: u64,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "n")]
    pub num_trades: u64,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "s")]
    pub coin: String,
    #[serde(rename = "t")]
    pub time_open: u64,
    #[serde(rename = "v")]
    pub volume: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderUpdate {
    pub order: BasicOrder,
    pub status: String,
    pub status_timestamp: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BasicOrder {
    pub coin: String,
    pub side: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: u64,
    pub timestamp: u64,
    pub orig_sz: String,
    pub cloid: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFundingsData {
    pub is_snapshot: Option<bool>,
    pub user: Address,
    pub fundings: Vec<UserFunding>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFunding {
    pub time: u64,
    pub coin: String,
    pub usdc: String,
    pub szi: String,
    pub funding_rate: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserNonFundingLedgerUpdatesData {
    pub is_snapshot: Option<bool>,
    pub user: Address,
    pub non_funding_ledger_updates: Vec<LedgerUpdateData>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LedgerUpdateData {
    pub time: u64,
    pub hash: String,
    pub delta: LedgerUpdate,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum LedgerUpdate {
    Deposit(Deposit),
    Withdraw(Withdraw),
    InternalTransfer(InternalTransfer),
    SubAccountTransfer(SubAccountTransfer),
    LedgerLiquidation(LedgerLiquidation),
    VaultDeposit(VaultDelta),
    VaultCreate(VaultDelta),
    VaultDistribution(VaultDelta),
    VaultWithdraw(VaultWithdraw),
    VaultLeaderCommission(VaultLeaderCommission),
    AccountClassTransfer(AccountClassTransfer),
    SpotTransfer(SpotTransfer),
    SpotGenesis(SpotGenesis),
}

#[derive(Deserialize, Clone, Debug)]
pub struct Deposit {
    pub usdc: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Withdraw {
    pub usdc: String,
    pub nonce: u64,
    pub fee: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct InternalTransfer {
    pub usdc: String,
    pub user: Address,
    pub destination: Address,
    pub fee: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SubAccountTransfer {
    pub usdc: String,
    pub user: Address,
    pub destination: Address,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LedgerLiquidation {
    pub account_value: u64,
    pub leverage_type: String,
    pub liquidated_positions: Vec<LiquidatedPosition>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LiquidatedPosition {
    pub coin: String,
    pub szi: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct VaultDelta {
    pub vault: Address,
    pub usdc: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VaultWithdraw {
    pub vault: Address,
    pub user: Address,
    pub requested_usd: String,
    pub commission: String,
    pub closing_cost: String,
    pub basis: String,
    pub net_withdrawn_usd: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct VaultLeaderCommission {
    pub user: Address,
    pub usdc: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountClassTransfer {
    pub usdc: String,
    pub to_perp: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotTransfer {
    pub token: String,
    pub amount: String,
    pub usdc_value: String,
    pub user: Address,
    pub destination: Address,
    pub fee: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SpotGenesis {
    pub token: String,
    pub amount: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NotificationData {
    pub notification: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebData2Data {
    pub user: Address,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAssetCtxData {
    pub coin: String,
    pub ctx: AssetCtx,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum AssetCtx {
    Perps(PerpsAssetCtx),
    Spot(SpotAssetCtx),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SharedAssetCtx {
    pub day_ntl_vlm: String,
    pub prev_day_px: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PerpsAssetCtx {
    #[serde(flatten)]
    pub shared: SharedAssetCtx,
    pub funding: String,
    pub open_interest: String,
    pub oracle_px: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSpotAssetCtxData {
    pub coin: String,
    pub ctx: SpotAssetCtx,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetCtx {
    #[serde(flatten)]
    pub shared: SharedAssetCtx,
    pub circulating_supply: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAssetDataData {
    pub user: Address,
    pub coin: String,
    pub leverage: Leverage,
    pub max_trade_szs: Vec<String>,
    pub available_to_trade: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BboData {
    pub coin: String,
    pub time: u64,
    pub bbo: Vec<Option<BookLevel>>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebData3Data {
    pub user_state: UserState,
    pub perp_dex_states: Vec<PerpDexState>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserState {
    pub agent_address: Option<Address>,
    pub agent_valid_until: Option<u64>,
    pub cum_ledger: String,
    pub server_time: u64,
    pub is_vault: bool,
    pub user: Address,
    pub opt_out_of_spot_dusting: Option<bool>,
    pub dex_abstraction_enabled: Option<bool>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PerpDexState {
    pub clearinghouse_state: crate::info::UserStateResponse,
    pub total_vault_equity: String,
    pub open_orders: Option<Vec<DetailedOrder>>,
    pub asset_ctxs: Vec<crate::meta::AssetContext>,
    pub perps_at_open_interest_cap: Option<Vec<String>>,
    pub leading_vaults: Option<Vec<LeadingVault>>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LeadingVault {
    pub address: Address,
    pub name: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DetailedOrder {
    pub coin: String,
    pub side: Side,
    pub limit_px: String,
    pub sz: String,
    pub oid: u64,
    pub timestamp: u64,
    pub orig_sz: String,
    pub trigger_condition: String,
    pub is_trigger: bool,
    pub trigger_px: String,
    pub children: Vec<DetailedOrder>,
    pub is_position_tpsl: bool,
    pub reduce_only: bool,
    pub order_type: OrderType,
    pub tif: Option<Tif>,
    pub cloid: Option<String>,
}
