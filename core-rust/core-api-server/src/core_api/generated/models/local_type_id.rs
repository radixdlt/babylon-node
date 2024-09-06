/*
 * Radix Core API
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.2
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct LocalTypeId {
    /// The location against which to resolve this type reference.
    #[serde(rename = "kind")]
    pub kind: Kind,
    /// A reference to a type, interpreted according to `kind`: - If `WellKnown`, then it is a pointer to a well known scrypto type with that ID, - If `SchemaLocal`, then it is an index into the given schema. 
    #[serde(rename = "id")]
    pub id: i64,
    #[serde(rename = "as_sbor")]
    pub as_sbor: Box<crate::core_api::generated::models::SborData>,
}

impl LocalTypeId {
    pub fn new(kind: Kind, id: i64, as_sbor: crate::core_api::generated::models::SborData) -> LocalTypeId {
        LocalTypeId {
            kind,
            id,
            as_sbor: Box::new(as_sbor),
        }
    }
}

/// The location against which to resolve this type reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    #[serde(rename = "WellKnown")]
    WellKnown,
    #[serde(rename = "SchemaLocal")]
    SchemaLocal,
}

impl Default for Kind {
    fn default() -> Kind {
        Self::WellKnown
    }
}

