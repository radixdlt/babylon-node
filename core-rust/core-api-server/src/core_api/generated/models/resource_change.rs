/*
 * Babylon Core API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ResourceChange {
    /// The Bech32m-encoded human readable version of the resource address
    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(rename = "component_entity")]
    pub component_entity: Box<crate::core_api::generated::models::EntityReference>,
    #[serde(rename = "vault_entity")]
    pub vault_entity: Box<crate::core_api::generated::models::EntityReference>,
    /// A decimal-string-encoded integer between `0` and `2^255 - 1`, which represents the total number of `10^(-18)` subunits in the XRD amount put or taken from the vault 
    #[serde(rename = "amount_attos")]
    pub amount_attos: String,
}

impl ResourceChange {
    pub fn new(resource_address: String, component_entity: crate::core_api::generated::models::EntityReference, vault_entity: crate::core_api::generated::models::EntityReference, amount_attos: String) -> ResourceChange {
        ResourceChange {
            resource_address,
            component_entity: Box::new(component_entity),
            vault_entity: Box::new(vault_entity),
            amount_attos,
        }
    }
}


