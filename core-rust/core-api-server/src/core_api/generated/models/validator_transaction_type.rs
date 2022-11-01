/*
 * Babylon Core API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// ValidatorTransactionType : The type of the validator transaction

/// The type of the validator transaction
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum ValidatorTransactionType {
    #[serde(rename = "EpochUpdate")]
    EpochUpdate,

}

impl ToString for ValidatorTransactionType {
    fn to_string(&self) -> String {
        match self {
            Self::EpochUpdate => String::from("EpochUpdate"),
        }
    }
}

impl Default for ValidatorTransactionType {
    fn default() -> ValidatorTransactionType {
        Self::EpochUpdate
    }
}




