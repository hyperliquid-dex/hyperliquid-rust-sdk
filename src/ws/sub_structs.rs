use ethers::types::H160;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug)]
pub struct Trade {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub time: u64,
    pub hash: String,
    pub tid: u64,
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
    pub tid: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFillsData {
    pub is_snapshot: Option<bool>,
    pub user: H160,
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
    pub user: H160,
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
    pub user: H160,
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
    pub user: H160,
    pub destination: H160,
    pub fee: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SubAccountTransfer {
    pub usdc: String,
    pub user: H160,
    pub destination: H160,
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
    pub vault: H160,
    pub usdc: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VaultWithdraw {
    pub vault: H160,
    pub user: H160,
    pub requested_usd: String,
    pub commission: String,
    pub closing_cost: String,
    pub basis: String,
    pub net_withdrawn_usd: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct VaultLeaderCommission {
    pub user: H160,
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
    pub user: H160,
    pub destination: H160,
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
    pub user: H160,
    pub clearinghouse_state: ClearinghouseState,
    pub leading_vaults: Vec<Vault>,
    pub total_vault_equity: String,
    pub open_orders: Vec<OrderUpdate>,
    pub agent_address: String,
    pub agent_valid_until: u64,
    pub cum_ledger: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClearinghouseState {
    pub margin_summary: MarginSummary,
    pub cross_margin_summary: MarginSummary,
    pub cross_maintenance_margin_used: String,
    pub withdrawable: String,
    pub asset_positions: Vec<AssetPosition>,
    pub time: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {
    pub account_value: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
    pub total_margin_used: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetPosition {
    #[serde(rename = "type")]
    pub position_type: String,
    pub position: Position,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub coin: String,
    pub szi: String,
    pub leverage: Leverage,
    pub entry_px: String,
    pub position_value: String,
    pub unrealized_pnl: String,
    pub return_on_equity: String,
    pub liquidation_px: Option<String>,
    pub margin_used: String,
    pub max_leverage: u64,
    pub cum_funding: CumFunding,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
    #[serde(rename = "type")]
    pub leverage_type: String,
    pub value: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CumFunding {
    pub all_time: String,
    pub since_open: String,
    pub since_change: String,
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

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetCtx {
    #[serde(flatten)]
    pub shared: SharedAssetCtx,
    pub circulating_supply: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vault {
    pub vault: H160,
    pub equity: String,
}

#[cfg(test)]
mod tests {
    use crate::WebData2;

    use ethers::types::H160;
    use std::str::FromStr;

    #[test]
    fn test_parse_web_data2() {
        let json_str = r#"
        {
            "channel": "webData2",
            "data": {
                "clearinghouseState": {
                "marginSummary": {
                    "accountValue": "1952.175398",
                    "totalNtlPos": "18.412",
                    "totalRawUsd": "1933.763398",
                    "totalMarginUsed": "3.6824"
                },
                "crossMarginSummary": {
                    "accountValue": "1952.175398",
                    "totalNtlPos": "18.412",
                    "totalRawUsd": "1933.763398",
                    "totalMarginUsed": "3.6824"
                },
                "crossMaintenanceMarginUsed": "1.8412",
                "withdrawable": "1948.492998",
                "assetPositions": [
                    {
                    "type": "oneWay",
                    "position": {
                        "coin": "HYPE",
                        "szi": "1.0",
                        "leverage": {
                        "type": "cross",
                        "value": 5
                        },
                        "entryPx": "18.398",
                        "positionValue": "18.412",
                        "unrealizedPnl": "0.014",
                        "returnOnEquity": "0.0038047614",
                        "liquidationPx": null,
                        "marginUsed": "3.6824",
                        "maxLeverage": 5,
                        "cumFunding": {
                        "allTime": "0.540959",
                        "sinceOpen": "0.0",
                        "sinceChange": "0.0"
                        }
                    }
                    }
                ],
                "time": 1745599106954
                },
                "leadingVaults": [],
                "totalVaultEquity": "0.0",
                "openOrders": [],
                "agentAddress": "0xe892bbcc0ad7b7ff8211fe0c23689df75491c641",
                "agentValidUntil": 1746138919412,
                "cumLedger": "2200.0",
                "user": "0xe892bbcc0ad7b7ff8211fe0c23689df75491c641"
            }
            }
        "#;

        let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        assert!(
            parsed.is_ok(),
            "Failed to parse JSON: {}",
            parsed.err().unwrap()
        );

        let web_data2_data: Result<WebData2, _> = serde_json::from_str(json_str);
        assert!(
            web_data2_data.is_ok(),
            "Failed to deserialize WebData2: {}",
            web_data2_data.err().unwrap()
        );

        let data = web_data2_data.unwrap();
        assert_eq!(
            data.data.user,
            H160::from_str("0xe892bbcc0ad7b7ff8211fe0c23689df75491c641").unwrap()
        );
        assert_eq!(data.data.clearinghouse_state.time, 1745599106954);
        assert_eq!(data.data.total_vault_equity, "0.0");
        assert_eq!(data.data.cum_ledger, "2200.0");
    }
}
