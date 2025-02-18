use alloy_primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Types {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Field {
    pub name: String,
    pub ty: String,
}

impl Types {
    pub(crate) fn new() -> Self {
        Self {
            name: String::new(),
            fields: Vec::new(),
        }
    }

    pub(crate) fn add_field(&mut self, name: &str, ty: &str) {
        self.fields.push(Field {
            name: name.to_string(),
            ty: ty.to_string(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Domain {
    pub name: String,
    pub version: String,
    pub chain_id: U256,
    pub verifying_contract: Option<Address>,
    pub salt: Option<B256>,
}

impl Domain {
    pub(crate) fn new(
        name: String,
        version: String,
        chain_id: U256,
        verifying_contract: Option<Address>,
        salt: Option<B256>,
    ) -> Self {
        Self {
            name,
            version,
            chain_id,
            verifying_contract,
            salt,
        }
    }
} 