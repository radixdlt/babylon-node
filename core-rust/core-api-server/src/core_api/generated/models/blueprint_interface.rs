/*
 * Babylon Core API - RCnet v3
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the second release candidate of the Radix Babylon network (\"RCnet v3\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct BlueprintInterface {
    #[serde(rename = "outer_blueprint", skip_serializing_if = "Option::is_none")]
    pub outer_blueprint: Option<String>,
    /// Generic (SBOR) type parameters which need to be filled by a concrete instance of this blueprint. 
    #[serde(rename = "generic_type_parameters")]
    pub generic_type_parameters: Vec<crate::core_api::generated::models::GenericTypeParameter>,
    /// If true, an instantiation of this blueprint cannot be persisted. EG buckets and proofs are transient.
    #[serde(rename = "is_transient")]
    pub is_transient: bool,
    #[serde(rename = "features")]
    pub features: Vec<String>,
    #[serde(rename = "state")]
    pub state: Box<crate::core_api::generated::models::IndexedStateSchema>,
    /// A map from the function name to the FunctionSchema
    #[serde(rename = "functions")]
    pub functions: ::utils::rust::prelude::IndexMap<String, crate::core_api::generated::models::FunctionSchema>,
    /// A map from the event name to the event payload type reference.
    #[serde(rename = "events")]
    pub events: ::utils::rust::prelude::IndexMap<String, crate::core_api::generated::models::BlueprintPayloadDef>,
}

impl BlueprintInterface {
    pub fn new(generic_type_parameters: Vec<crate::core_api::generated::models::GenericTypeParameter>, is_transient: bool, features: Vec<String>, state: crate::core_api::generated::models::IndexedStateSchema, functions: ::utils::rust::prelude::IndexMap<String, crate::core_api::generated::models::FunctionSchema>, events: ::utils::rust::prelude::IndexMap<String, crate::core_api::generated::models::BlueprintPayloadDef>) -> BlueprintInterface {
        BlueprintInterface {
            outer_blueprint: None,
            generic_type_parameters,
            is_transient,
            features,
            state: Box::new(state),
            functions,
            events,
        }
    }
}


