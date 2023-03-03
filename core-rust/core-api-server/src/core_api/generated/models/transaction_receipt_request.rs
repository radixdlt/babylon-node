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
pub struct TransactionReceiptRequest {
    /// The logical name of the network
    #[serde(rename = "network")]
    pub network: String,
    /// The hex-encoded transaction intent hash. This is known as the Intent Hash, Transaction ID or Transaction Identifier for user transactions. This hash is `Blake2b-256(compiled_intent)`
    #[serde(rename = "intent_hash")]
    pub intent_hash: String,
}

impl TransactionReceiptRequest {
    pub fn new(network: String, intent_hash: String) -> TransactionReceiptRequest {
        TransactionReceiptRequest {
            network,
            intent_hash,
        }
    }
}


