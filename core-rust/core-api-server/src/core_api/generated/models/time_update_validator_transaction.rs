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
pub struct TimeUpdateValidatorTransaction {
    #[serde(rename = "type")]
    pub _type: crate::core_api::generated::models::ValidatorTransactionType,
    #[serde(rename = "proposer_timestamp")]
    pub proposer_timestamp: Box<crate::core_api::generated::models::Instant>,
    /// An integer between `0` and `10^10`, marking the consensus epoch. Note that currently this is not the same as `scrypto_epoch`, but eventually will be. 
    #[serde(rename = "consensus_epoch")]
    pub consensus_epoch: i64,
    /// An integer between `0` and `10^10`, marking the consensus round in the epoch
    #[serde(rename = "round_in_epoch")]
    pub round_in_epoch: i64,
}

impl TimeUpdateValidatorTransaction {
    pub fn new(_type: crate::core_api::generated::models::ValidatorTransactionType, proposer_timestamp: crate::core_api::generated::models::Instant, consensus_epoch: i64, round_in_epoch: i64) -> TimeUpdateValidatorTransaction {
        TimeUpdateValidatorTransaction {
            _type,
            proposer_timestamp: Box::new(proposer_timestamp),
            consensus_epoch,
            round_in_epoch,
        }
    }
}


