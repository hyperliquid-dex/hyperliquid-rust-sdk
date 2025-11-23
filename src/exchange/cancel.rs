use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Shared Types and Aliases ---

/// Type alias for the internal/wire format of an Asset ID.
pub type AssetId = u32;

/// Type alias for a standard Order ID (OID).
pub type OrderId = u64;

/// Type alias for a Client Order ID (CLOID) in the wire format (String representation).
pub type ClientOrderIdWire = String;

// --- API / Internal Application Structures (Public Facing) ---

/// Represents an incoming API request to cancel an order by its standard ID (OID).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ApiCancelRequest {
    /// Human-readable asset identifier (e.g., "BTC-USD").
    pub asset: String,
    /// The standard Order ID assigned by the exchange.
    pub oid: OrderId,
}

/// Represents an incoming API request to cancel an order by its Client Order ID (CLOID).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ApiCancelRequestCloid {
    /// Human-readable asset identifier (e.g., "BTC-USD").
    pub asset: String,
    /// The UUID assigned by the client.
    pub cloid: Uuid,
}

// --- Wire / DTO Structures (Serialized Format) ---

/// Wire format for canceling an order by its standard ID.
/// Uses minimized keys ("a", "o") for efficiency over the wire.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WireCancelRequest {
    #[serde(rename = "a", alias = "asset")]
    pub asset: AssetId,
    #[serde(rename = "o", alias = "oid")]
    pub oid: OrderId,
}

/// Wire format for canceling an order by its Client Order ID.
/// Uses minimized keys ("a", "c") for efficiency over the wire (consistent optimization).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WireCancelRequestCloid {
    #[serde(rename = "a", alias = "asset")]
    pub asset: AssetId,
    // CLOID is typically sent as a canonical hex string over the wire.
    #[serde(rename = "c", alias = "cloid")]
    pub cloid: ClientOrderIdWire,
}

// --- Conversion Implementations (Example mapping needed) ---

// NOTE: Conversion from the public 'Api*' structures to the wire 'Wire*' 
// structures requires business logic to map the human-readable 'asset: String' 
// to the internal 'asset: AssetId (u32)'. The following implementations assume 
// this mapping is external or mocked for demonstration.

/// Example implementation for converting an internal OID request to the wire format.
impl From<ApiCancelRequest> for WireCancelRequest {
    fn from(api_req: ApiCancelRequest) -> Self {
        // --- TODO: Implement asset name to u32 ID resolution here ---
        // Placeholder implementation:
        let internal_asset_id = if api_req.asset == "BTC-USD" { 101 } else { 0 }; 
        // -----------------------------------------------------------------

        WireCancelRequest {
            asset: internal_asset_id,
            oid: api_req.oid,
        }
    }
}

/// Example implementation for converting an internal CLOID request to the wire format.
impl From<ApiCancelRequestCloid> for WireCancelRequestCloid {
    fn from(api_req: ApiCancelRequestCloid) -> Self {
        // --- TODO: Implement asset name to u32 ID resolution here ---
        // Placeholder implementation:
        let internal_asset_id = if api_req.asset == "ETH-USD" { 202 } else { 0 }; 
        // -----------------------------------------------------------------
        
        WireCancelRequestCloid {
            asset: internal_asset_id,
            // CLOID must be converted to its String representation for serialization.
            cloid: api_req.cloid.to_string(), 
        }
    }
}
