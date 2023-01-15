/*
 * Babylon Core API
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.2.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct NonFungibleDynamicResourceDescriptorAllOf {
    /// The Bech32m-encoded human readable version of the resource address
    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(rename = "non_fungible_id")]
    pub non_fungible_id: Box<crate::core_api::generated::models::NonFungibleId>,
}

impl NonFungibleDynamicResourceDescriptorAllOf {
    pub fn new(resource_address: String, non_fungible_id: crate::core_api::generated::models::NonFungibleId) -> NonFungibleDynamicResourceDescriptorAllOf {
        NonFungibleDynamicResourceDescriptorAllOf {
            resource_address,
            non_fungible_id: Box::new(non_fungible_id),
        }
    }
}


