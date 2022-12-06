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
pub struct EpochUpdateValidatorTransaction {
    #[serde(rename = "type")]
    pub _type: crate::core_api::generated::models::ValidatorTransactionType,
    /// An integer between `0` and `10^10`, marking the new epoch. Note that currently this is not the same as `consensus_epoch`, but eventually will be. 
    #[serde(rename = "scrypto_epoch")]
    pub scrypto_epoch: i64,
}

impl EpochUpdateValidatorTransaction {
    pub fn new(_type: crate::core_api::generated::models::ValidatorTransactionType, scrypto_epoch: i64) -> EpochUpdateValidatorTransaction {
        EpochUpdateValidatorTransaction {
            _type,
            scrypto_epoch,
        }
    }
}


