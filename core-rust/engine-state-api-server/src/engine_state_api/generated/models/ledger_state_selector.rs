/*
 * Engine State API (Beta)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.3.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// LedgerStateSelector : An optional specification of a historical ledger state at which to execute the request. The \"historical state\" feature (see the `db.historical_substate_values.enable` flag) must be enabled on the Node, and the requested point in history must be recent enough (in accordance with the Node's configured `state_hash_tree.state_version_history_length`). 


#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum LedgerStateSelector {
    #[serde(rename="ByStateVersion")]
    VersionLedgerStateSelector {
        #[serde(rename = "state_version")]
        state_version: i64,
    },
}




