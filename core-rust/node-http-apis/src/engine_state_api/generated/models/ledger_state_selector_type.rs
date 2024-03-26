/*
 * Engine State API
 *
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */


/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum LedgerStateSelectorType {
    #[serde(rename = "ByStateVersion")]
    ByStateVersion,

}

impl ToString for LedgerStateSelectorType {
    fn to_string(&self) -> String {
        match self {
            Self::ByStateVersion => String::from("ByStateVersion"),
        }
    }
}

impl Default for LedgerStateSelectorType {
    fn default() -> LedgerStateSelectorType {
        Self::ByStateVersion
    }
}




