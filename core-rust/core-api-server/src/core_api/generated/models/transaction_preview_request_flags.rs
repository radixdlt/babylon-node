/*
 * Babylon Core API
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.3.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct TransactionPreviewRequestFlags {
    #[serde(rename = "unlimited_loan")]
    pub unlimited_loan: bool,
    #[serde(rename = "assume_all_signature_proofs")]
    pub assume_all_signature_proofs: bool,
    #[serde(rename = "permit_duplicate_intent_hash")]
    pub permit_duplicate_intent_hash: bool,
    #[serde(rename = "permit_invalid_header_epoch")]
    pub permit_invalid_header_epoch: bool,
}

impl TransactionPreviewRequestFlags {
    pub fn new(unlimited_loan: bool, assume_all_signature_proofs: bool, permit_duplicate_intent_hash: bool, permit_invalid_header_epoch: bool) -> TransactionPreviewRequestFlags {
        TransactionPreviewRequestFlags {
            unlimited_loan,
            assume_all_signature_proofs,
            permit_duplicate_intent_hash,
            permit_invalid_header_epoch,
        }
    }
}


