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
pub struct V0CommittedTransactionRequest {
    /// The hex-encoded transaction intent hash. This is known as the Intent Hash, Transaction ID or Transaction Identifier for user transactions. This hash is `SHA256(SHA256(compiled_intent))`
    #[serde(rename = "intent_hash")]
    pub intent_hash: String,
}

impl V0CommittedTransactionRequest {
    pub fn new(intent_hash: String) -> V0CommittedTransactionRequest {
        V0CommittedTransactionRequest {
            intent_hash,
        }
    }
}


