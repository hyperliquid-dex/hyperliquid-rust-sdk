use ethers::contract::{Eip712, EthAbiType};
use serde::Serialize;

pub(crate) mod mainnet {
    use super::*;
    #[derive(Debug, Eip712, Clone, EthAbiType, Serialize)]
    #[eip712(
        name = "Exchange",
        version = "1",
        chain_id = 42161,
        verifying_contract = "0x0000000000000000000000000000000000000000"
    )]
    pub(crate) struct UsdTransferSignPayload {
        pub(crate) destination: String,
        pub(crate) amount: String,
        pub(crate) time: u64,
    }
}

pub(crate) mod testnet {
    use super::*;
    #[derive(Debug, Eip712, Clone, EthAbiType)]
    #[eip712(
        name = "Exchange",
        version = "1",
        chain_id = 421613,
        verifying_contract = "0x0000000000000000000000000000000000000000"
    )]
    pub(crate) struct UsdTransferSignPayload {
        pub(crate) destination: String,
        pub(crate) amount: String,
        pub(crate) time: u64,
    }
}
