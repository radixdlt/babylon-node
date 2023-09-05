/*
 * Babylon Core API - RCnet v3.1
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// FullyScopedTypeId : An identifier for a type in the context of a schema in an entity's schema partition.  Note - this type provides a schema context even for well-known types where this context is effectively irrelevant. 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct FullyScopedTypeId {
    /// Bech32m-encoded human readable version of the entity's address (ie the entity's node id)
    #[serde(rename = "entity_address")]
    pub entity_address: String,
    /// The hex-encoded schema hash, capturing the identity of an SBOR schema.
    #[serde(rename = "schema_hash")]
    pub schema_hash: String,
    #[serde(rename = "local_type_id")]
    pub local_type_id: Box<crate::core_api::generated::models::LocalTypeId>,
}

impl FullyScopedTypeId {
    /// An identifier for a type in the context of a schema in an entity's schema partition.  Note - this type provides a schema context even for well-known types where this context is effectively irrelevant. 
    pub fn new(entity_address: String, schema_hash: String, local_type_id: crate::core_api::generated::models::LocalTypeId) -> FullyScopedTypeId {
        FullyScopedTypeId {
            entity_address,
            schema_hash,
            local_type_id: Box::new(local_type_id),
        }
    }
}


