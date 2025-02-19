use alloy::primitives::{Address, U256};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    exchange::actions::types::{ApproveAgent, ClassTransfer, SpotSend, UsdSend, Withdraw3},
    BuilderInfo, ClientCancelRequest, ClientCancelRequestCloid, ClientOrderRequest, Error,
    ExchangeResponseStatus, VaultTransfer,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Actions {
    UsdSend(UsdSend),
    ApproveAgent(ApproveAgent),
    Withdraw3(Withdraw3),
    SpotSend(SpotSend),
    ClassTransfer(ClassTransfer),
}

#[derive(Debug)]
pub struct ExchangeClient {
    http_client: Client,
    base_url: String,
}

impl ExchangeClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
        }
    }

    pub fn get_timestamp(&self) -> U256 {
        U256::from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }

    pub async fn usd_send(
        &self,
        destination: Address,
        amount: U256,
        hyperliquid_chain: String,
    ) -> Result<(), Error> {
        let timestamp = self.get_timestamp();
        let req = UsdSend {
            signatureChainId: U256::from(421614u64),
            hyperliquidChain: hyperliquid_chain,
            destination,
            amount,
            time: timestamp,
        };
        let req_json = serde_json::to_string(&req)?;
        self.http_client
            .post(format!("{}/exchange/usdTransfer", self.base_url))
            .body(req_json)
            .send()
            .await?;
        Ok(())
    }

    pub async fn approve_agent(
        &self,
        address: Address,
        hyperliquid_chain: String,
    ) -> Result<(), Error> {
        let timestamp = self.get_timestamp();
        let req = ApproveAgent {
            signatureChainId: U256::from(421614u64),
            hyperliquidChain: hyperliquid_chain,
            agent: address,
            time: timestamp,
        };
        let req_json = serde_json::to_string(&req)?;
        self.http_client
            .post(format!("{}/exchange/approveAgent", self.base_url))
            .body(req_json)
            .send()
            .await?;
        Ok(())
    }

    pub async fn withdraw(
        &self,
        destination: Address,
        amount: U256,
        hyperliquid_chain: String,
    ) -> Result<(), Error> {
        let timestamp = self.get_timestamp();
        let req = Withdraw3 {
            signatureChainId: U256::from(421614u64),
            hyperliquidChain: hyperliquid_chain,
            destination,
            amount,
            time: timestamp,
        };
        let req_json = serde_json::to_string(&req)?;
        self.http_client
            .post(format!("{}/exchange/withdraw", self.base_url))
            .body(req_json)
            .send()
            .await?;
        Ok(())
    }

    pub async fn spot_send(
        &self,
        destination: Address,
        token: String,
        amount: U256,
        hyperliquid_chain: String,
    ) -> Result<(), Error> {
        let timestamp = self.get_timestamp();
        let req = SpotSend {
            signatureChainId: U256::from(421614u64),
            hyperliquidChain: hyperliquid_chain,
            destination,
            amount,
            token,
            time: timestamp,
        };
        let req_json = serde_json::to_string(&req)?;
        self.http_client
            .post(format!("{}/exchange/spotTransfer", self.base_url))
            .body(req_json)
            .send()
            .await?;
        Ok(())
    }

    pub async fn class_transfer(
        &self,
        amount: U256,
        to_perp: bool,
        hyperliquid_chain: String,
    ) -> Result<(), Error> {
        let timestamp = self.get_timestamp();
        let req = ClassTransfer {
            signatureChainId: U256::from(421614u64),
            hyperliquidChain: hyperliquid_chain,
            amount,
            toPerp: to_perp,
            time: timestamp,
        };
        let req_json = serde_json::to_string(&req)?;
        self.http_client
            .post(format!("{}/exchange/classTransfer", self.base_url))
            .body(req_json)
            .send()
            .await?;
        Ok(())
    }

    pub async fn cancel(
        &self,
        req: ClientCancelRequest,
        _builder: Option<BuilderInfo>,
    ) -> Result<ExchangeResponseStatus, Error> {
        let req_json = serde_json::to_string(&req)?;
        let response = self
            .http_client
            .post(format!("{}/exchange/cancel", self.base_url))
            .body(req_json)
            .send()
            .await?;
        let text = response.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn order(
        &self,
        req: ClientOrderRequest,
        _builder: Option<BuilderInfo>,
    ) -> Result<ExchangeResponseStatus, Error> {
        let req_json = serde_json::to_string(&req)?;
        let response = self
            .http_client
            .post(format!("{}/exchange/order", self.base_url))
            .body(req_json)
            .send()
            .await?;
        let text = response.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn set_referrer(&self, code: String) -> Result<(), Error> {
        let req = serde_json::json!({
            "code": code
        });
        let req_json = serde_json::to_string(&req)?;
        self.http_client
            .post(format!("{}/exchange/setReferrer", self.base_url))
            .body(req_json)
            .send()
            .await?;
        Ok(())
    }

    pub async fn cancel_by_cloid(
        &self,
        req: ClientCancelRequestCloid,
        _builder: Option<BuilderInfo>,
    ) -> Result<ExchangeResponseStatus, Error> {
        let req_json = serde_json::to_string(&req)?;
        let response = self
            .http_client
            .post(format!("{}/exchange/cancelByCloid", self.base_url))
            .body(req_json)
            .send()
            .await?;
        let text = response.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn vault_transfer(
        &self,
        vault_address: Address,
        is_deposit: bool,
        usd: String,
        hyperliquid_chain: String,
    ) -> Result<(), Error> {
        let req = VaultTransfer {
            vault_address,
            is_deposit,
            usd,
        };
        let req_json = serde_json::to_string(&req)?;
        self.http_client
            .post(format!("{}/exchange/vaultTransfer", self.base_url))
            .body(req_json)
            .send()
            .await?;
        Ok(())
    }
}
