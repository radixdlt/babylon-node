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
pub struct AddressType {
    #[serde(rename = "subtype")]
    pub subtype: Subtype,
    #[serde(rename = "hrp_prefix")]
    pub hrp_prefix: String,
    #[serde(rename = "entity_type")]
    pub entity_type: crate::core_api::generated::models::EntityType,
    #[serde(rename = "address_byte_prefix")]
    pub address_byte_prefix: i32,
    #[serde(rename = "address_byte_length")]
    pub address_byte_length: i32,
}

impl AddressType {
    pub fn new(subtype: Subtype, hrp_prefix: String, entity_type: crate::core_api::generated::models::EntityType, address_byte_prefix: i32, address_byte_length: i32) -> AddressType {
        AddressType {
            subtype,
            hrp_prefix,
            entity_type,
            address_byte_prefix,
            address_byte_length,
        }
    }
}

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum Subtype {
    #[serde(rename = "Resource")]
    Resource,
    #[serde(rename = "Package")]
    Package,
    #[serde(rename = "NormalComponent")]
    NormalComponent,
    #[serde(rename = "AccountComponent")]
    AccountComponent,
    #[serde(rename = "EcdsaSecp256k1VirtualAccountComponent")]
    EcdsaSecp256k1VirtualAccountComponent,
    #[serde(rename = "EddsaEd25519VirtualAccountComponent")]
    EddsaEd25519VirtualAccountComponent,
    #[serde(rename = "EpochManager")]
    EpochManager,
    #[serde(rename = "Clock")]
    Clock,
}

impl Default for Subtype {
    fn default() -> Subtype {
        Self::Resource
    }
}

