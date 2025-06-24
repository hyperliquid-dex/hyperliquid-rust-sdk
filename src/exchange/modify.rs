use crate::helpers::{hex_string_to_uuid, uuid_to_hex_string};

use super::{order::OrderRequest, ClientOrderRequest};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum OidOrCloid {
    Oid(u64),
    Cloid(Uuid),
}

impl Serialize for OidOrCloid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OidOrCloid::Oid(oid) => serializer.serialize_u64(*oid),
            // UUID rendered as compact hex (no dashes)
            OidOrCloid::Cloid(cloid) => serializer.serialize_str(&uuid_to_hex_string(*cloid)),
        }
    }
}

impl<'de> Deserialize<'de> for OidOrCloid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl<'de> de::Visitor<'de> for V {
            type Value = OidOrCloid;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a u64 or a 32-character hex UUID string")
            }

            fn visit_u64<E>(self, n: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(OidOrCloid::Oid(n))
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                hex_string_to_uuid(s)
                    .map(OidOrCloid::Cloid)
                    .map_err(E::custom)
            }
        }

        deserializer.deserialize_any(V)
    }
}

#[derive(Debug)]
pub struct ClientModifyRequest {
    pub oid: OidOrCloid,
    pub order: ClientOrderRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifyRequest {
    pub oid: OidOrCloid,
    pub order: OrderRequest,
}
