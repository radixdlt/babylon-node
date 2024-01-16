/*
 * Browse API
 *
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// SystemType : Determines whether an entity is an object or a key-value store. This categorization is introduced only for convenience / API discoverability (e.g. some endpoints are suited for working only with objects). In fact, the `SystemType` can also be derived from `EntityType`. 

/// Determines whether an entity is an object or a key-value store. This categorization is introduced only for convenience / API discoverability (e.g. some endpoints are suited for working only with objects). In fact, the `SystemType` can also be derived from `EntityType`. 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum SystemType {
    #[serde(rename = "Object")]
    Object,
    #[serde(rename = "KeyValueStore")]
    KeyValueStore,

}

impl ToString for SystemType {
    fn to_string(&self) -> String {
        match self {
            Self::Object => String::from("Object"),
            Self::KeyValueStore => String::from("KeyValueStore"),
        }
    }
}

impl Default for SystemType {
    fn default() -> SystemType {
        Self::Object
    }
}




