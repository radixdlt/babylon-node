/*
 * Babylon Core API
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.2.0
 * 
 * Generated by: https://openapi-generator.tech
 */


/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum DynamicResourceDescriptorListType {
    #[serde(rename = "List")]
    List,
    #[serde(rename = "SchemaPath")]
    SchemaPath,

}

impl ToString for DynamicResourceDescriptorListType {
    fn to_string(&self) -> String {
        match self {
            Self::List => String::from("List"),
            Self::SchemaPath => String::from("SchemaPath"),
        }
    }
}

impl Default for DynamicResourceDescriptorListType {
    fn default() -> DynamicResourceDescriptorListType {
        Self::List
    }
}




