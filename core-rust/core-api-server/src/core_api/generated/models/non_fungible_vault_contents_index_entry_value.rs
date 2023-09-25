/*
 * Radix Core API - Babylon
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// NonFungibleVaultContentsIndexEntryValue : This object is empty, and always present on this substate. Note that when a non-fungible is withdrawn from the vault, the full substate is deleted, which is marked by a DeletedSubstate action (rather than deletion of the value property in an UpdateSubstate action). This is because this is an Index entry, not a KeyValueStore entry. 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct NonFungibleVaultContentsIndexEntryValue {
    /// This is a dummy property which is always set to true and means nothing. It exists just to ensure this object has a well-defined type in OpenAPI schemas. 
    #[serde(rename = "is_present")]
    pub is_present: bool,
}

impl NonFungibleVaultContentsIndexEntryValue {
    /// This object is empty, and always present on this substate. Note that when a non-fungible is withdrawn from the vault, the full substate is deleted, which is marked by a DeletedSubstate action (rather than deletion of the value property in an UpdateSubstate action). This is because this is an Index entry, not a KeyValueStore entry. 
    pub fn new(is_present: bool) -> NonFungibleVaultContentsIndexEntryValue {
        NonFungibleVaultContentsIndexEntryValue {
            is_present,
        }
    }
}


