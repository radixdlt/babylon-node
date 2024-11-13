/*
 * Radix Core API
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
 * 
 * Generated by: https://openapi-generator.tech
 */

/// NetworkConfigurationResponseWellKnownAddresses : Key addresses for this network, as per https://docs.radixdlt.com/docs/well-known-addresses.  Note that at Cuttlefish, some of these names have been updated elsewhere in the stack, but for backwards compatibility, we must use the old names here.  Notably: - `secp256k1_signature_virtual_badge` is now `secp256k1_signature_resource` elsewhere - `ed25519_signature_virtual_badge` is now `ed25519_signature_resource` elsewhere - `package_of_direct_caller_virtual_badge` is now `package_of_direct_caller_resource` elsewhere - `global_caller_virtual_badge` is now `global_caller_resource` elsewhere - `system_transaction_badge` is now `system_transaction_resource` elsewhere 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfigurationResponseWellKnownAddresses {
    #[serde(rename = "xrd")]
    pub xrd: String,
    #[serde(rename = "secp256k1_signature_virtual_badge")]
    pub secp256k1_signature_virtual_badge: String,
    #[serde(rename = "ed25519_signature_virtual_badge")]
    pub ed25519_signature_virtual_badge: String,
    #[serde(rename = "system_transaction_badge")]
    pub system_transaction_badge: String,
    #[serde(rename = "package_of_direct_caller_virtual_badge")]
    pub package_of_direct_caller_virtual_badge: String,
    #[serde(rename = "global_caller_virtual_badge")]
    pub global_caller_virtual_badge: String,
    #[serde(rename = "package_owner_badge")]
    pub package_owner_badge: String,
    #[serde(rename = "validator_owner_badge")]
    pub validator_owner_badge: String,
    #[serde(rename = "account_owner_badge")]
    pub account_owner_badge: String,
    #[serde(rename = "identity_owner_badge")]
    pub identity_owner_badge: String,
    #[serde(rename = "package_package")]
    pub package_package: String,
    #[serde(rename = "resource_package")]
    pub resource_package: String,
    #[serde(rename = "account_package")]
    pub account_package: String,
    #[serde(rename = "identity_package")]
    pub identity_package: String,
    #[serde(rename = "consensus_manager_package")]
    pub consensus_manager_package: String,
    #[serde(rename = "access_controller_package")]
    pub access_controller_package: String,
    #[serde(rename = "transaction_processor_package")]
    pub transaction_processor_package: String,
    #[serde(rename = "metadata_module_package")]
    pub metadata_module_package: String,
    #[serde(rename = "royalty_module_package")]
    pub royalty_module_package: String,
    #[serde(rename = "role_assignment_module_package")]
    pub role_assignment_module_package: String,
    #[serde(rename = "genesis_helper_package")]
    pub genesis_helper_package: String,
    #[serde(rename = "faucet_package")]
    pub faucet_package: String,
    #[serde(rename = "pool_package")]
    pub pool_package: String,
    #[serde(rename = "transaction_tracker_package", skip_serializing_if = "Option::is_none")]
    pub transaction_tracker_package: Option<String>,
    #[serde(rename = "locker_package", skip_serializing_if = "Option::is_none")]
    pub locker_package: Option<String>,
    #[serde(rename = "test_utils_package", skip_serializing_if = "Option::is_none")]
    pub test_utils_package: Option<String>,
    #[serde(rename = "consensus_manager")]
    pub consensus_manager: String,
    #[serde(rename = "genesis_helper")]
    pub genesis_helper: String,
    #[serde(rename = "faucet")]
    pub faucet: String,
    #[serde(rename = "transaction_tracker")]
    pub transaction_tracker: String,
}

impl NetworkConfigurationResponseWellKnownAddresses {
    /// Key addresses for this network, as per https://docs.radixdlt.com/docs/well-known-addresses.  Note that at Cuttlefish, some of these names have been updated elsewhere in the stack, but for backwards compatibility, we must use the old names here.  Notably: - `secp256k1_signature_virtual_badge` is now `secp256k1_signature_resource` elsewhere - `ed25519_signature_virtual_badge` is now `ed25519_signature_resource` elsewhere - `package_of_direct_caller_virtual_badge` is now `package_of_direct_caller_resource` elsewhere - `global_caller_virtual_badge` is now `global_caller_resource` elsewhere - `system_transaction_badge` is now `system_transaction_resource` elsewhere 
    pub fn new(xrd: String, secp256k1_signature_virtual_badge: String, ed25519_signature_virtual_badge: String, system_transaction_badge: String, package_of_direct_caller_virtual_badge: String, global_caller_virtual_badge: String, package_owner_badge: String, validator_owner_badge: String, account_owner_badge: String, identity_owner_badge: String, package_package: String, resource_package: String, account_package: String, identity_package: String, consensus_manager_package: String, access_controller_package: String, transaction_processor_package: String, metadata_module_package: String, royalty_module_package: String, role_assignment_module_package: String, genesis_helper_package: String, faucet_package: String, pool_package: String, consensus_manager: String, genesis_helper: String, faucet: String, transaction_tracker: String) -> NetworkConfigurationResponseWellKnownAddresses {
        NetworkConfigurationResponseWellKnownAddresses {
            xrd,
            secp256k1_signature_virtual_badge,
            ed25519_signature_virtual_badge,
            system_transaction_badge,
            package_of_direct_caller_virtual_badge,
            global_caller_virtual_badge,
            package_owner_badge,
            validator_owner_badge,
            account_owner_badge,
            identity_owner_badge,
            package_package,
            resource_package,
            account_package,
            identity_package,
            consensus_manager_package,
            access_controller_package,
            transaction_processor_package,
            metadata_module_package,
            royalty_module_package,
            role_assignment_module_package,
            genesis_helper_package,
            faucet_package,
            pool_package,
            transaction_tracker_package: None,
            locker_package: None,
            test_utils_package: None,
            consensus_manager,
            genesis_helper,
            faucet,
            transaction_tracker,
        }
    }
}


