/*
 * Radix Core API - Babylon (Bottlenose)
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// StreamProofsFilter : If not provided, defaults to \"Any\".


#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum StreamProofsFilter {
    #[serde(rename="Any")]
    StreamProofsFilterAny {
        #[serde(rename = "from_state_version", skip_serializing_if = "Option::is_none")]
        from_state_version: Option<i64>,
    },
    #[serde(rename="NewEpochs")]
    StreamProofsFilterNewEpochs {
        /// The first proof to be returned should be the proof starting this epoch. If empty, it starts from the first epoch proof after genesis. The network status endpoint can be used to find the current epoch.
        #[serde(rename = "from_epoch", skip_serializing_if = "Option::is_none")]
        from_epoch: Option<i64>,
    },
    #[serde(rename="ProtocolUpdateExecution")]
    StreamProofsFilterProtocolUpdateExecution {
        /// The protocol version name to filter to. 
        #[serde(rename = "protocol_version", skip_serializing_if = "Option::is_none")]
        protocol_version: Option<String>,
        #[serde(rename = "from_state_version", skip_serializing_if = "Option::is_none")]
        from_state_version: Option<i64>,
    },
    #[serde(rename="ProtocolUpdateInitializations")]
    StreamProofsFilterProtocolUpdateInitializations {
        #[serde(rename = "from_state_version", skip_serializing_if = "Option::is_none")]
        from_state_version: Option<i64>,
    },
}




