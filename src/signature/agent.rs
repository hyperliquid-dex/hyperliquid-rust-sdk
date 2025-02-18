use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolType, SolValue};
use crate::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) mod l1 {
    use super::*;

    sol! {
        #[derive(Debug)]
        struct Agent {
            address agent;
            string name;
            uint64 time;
        }
    }

    impl Agent {
        pub fn new(agent: Address, name: String, time: u64) -> Self {
            Self {
                agent,
                name,
                time,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Agent {
    pub address: Address,
    pub name: Option<String>,
}

impl Agent {
    pub(crate) fn new(address: Address, name: Option<String>) -> Self {
        Self { address, name }
    }
}
