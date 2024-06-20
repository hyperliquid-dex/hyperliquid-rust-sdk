use ethers::{
    contract::{Eip712, EthAbiType},
    types::H256,
};

pub(crate) mod l1 {
    use super::*;
    #[derive(Debug, Eip712, Clone, EthAbiType)]
    #[eip712(
        name = "Exchange",
        version = "1",
        chain_id = 1337,
        verifying_contract = "0x0000000000000000000000000000000000000000"
    )]
    pub(crate) struct Agent {
        pub(crate) source: String,
        pub(crate) connection_id: H256,
    }
}
