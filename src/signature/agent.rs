pub(crate) mod l1 {
    use crate::Eip712;
    use alloy::{dyn_abi::Eip712Domain, primitives::Address, sol, sol_types::eip712_domain};
    use serde::{Deserialize, Serialize};

    sol! {
        #[derive(Serialize, Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct Agent {
            string source;
            bytes32 connection_id;
        }
    }

    impl Eip712 for Agent {
        fn domain(&self) -> Eip712Domain {
            eip712_domain!(
                name: "Exchange",
                version: "1",
                chain_id: 1337,
                verifying_contract: Address::ZERO,
            )
        }
    }
}
