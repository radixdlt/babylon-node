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
pub struct ParsedSignedTransactionIntentAllOfIdentifiers {
    /// The hex-encoded transaction intent hash. This is known as the Intent Hash, Transaction ID or Transaction Identifier for user transactions. This hash is `SHA256(SHA256(compiled_intent))`
    #[serde(rename = "intent_hash")]
    pub intent_hash: String,
    /// The hex-encoded signed transaction hash. This is known as the Signed Transaction Hash or Signatures Hash. This is the hash which is signed as part of notarization. This hash is `SHA256(SHA256(compiled_signed_transaction))`
    #[serde(rename = "signatures_hash")]
    pub signatures_hash: String,
}

impl ParsedSignedTransactionIntentAllOfIdentifiers {
    pub fn new(intent_hash: String, signatures_hash: String) -> ParsedSignedTransactionIntentAllOfIdentifiers {
        ParsedSignedTransactionIntentAllOfIdentifiers {
            intent_hash,
            signatures_hash,
        }
    }
}


