/*
 * Babylon Core API - RCnet V2
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.4.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct SubstateId {
    #[serde(rename = "entity_type")]
    pub entity_type: crate::core_api::generated::models::EntityType,
    /// The hex-encoded bytes of the entity id.
    #[serde(rename = "entity_id_hex")]
    pub entity_id_hex: String,
    #[serde(rename = "module_type")]
    pub module_type: crate::core_api::generated::models::SysModuleType,
    #[serde(rename = "substate_type")]
    pub substate_type: crate::core_api::generated::models::SubstateType,
    /// The hex-encoded bytes of the substate key, under the entity
    #[serde(rename = "substate_key_hex")]
    pub substate_key_hex: String,
}

impl SubstateId {
    pub fn new(entity_type: crate::core_api::generated::models::EntityType, entity_id_hex: String, module_type: crate::core_api::generated::models::SysModuleType, substate_type: crate::core_api::generated::models::SubstateType, substate_key_hex: String) -> SubstateId {
        SubstateId {
            entity_type,
            entity_id_hex,
            module_type,
            substate_type,
            substate_key_hex,
        }
    }
}


