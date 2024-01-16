/*
 * Browse API
 *
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */


/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum BlueprintFunctionAuthorizationType {
    #[serde(rename = "Public")]
    Public,
    #[serde(rename = "ByAccessRule")]
    ByAccessRule,
    #[serde(rename = "RootOnly")]
    RootOnly,

}

impl ToString for BlueprintFunctionAuthorizationType {
    fn to_string(&self) -> String {
        match self {
            Self::Public => String::from("Public"),
            Self::ByAccessRule => String::from("ByAccessRule"),
            Self::RootOnly => String::from("RootOnly"),
        }
    }
}

impl Default for BlueprintFunctionAuthorizationType {
    fn default() -> BlueprintFunctionAuthorizationType {
        Self::Public
    }
}




