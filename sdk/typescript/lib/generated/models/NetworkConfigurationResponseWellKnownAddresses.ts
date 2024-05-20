/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon (Bottlenose)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.1
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
/**
 * Key addresses for this network.
 * @export
 * @interface NetworkConfigurationResponseWellKnownAddresses
 */
export interface NetworkConfigurationResponseWellKnownAddresses {
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    xrd: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    secp256k1_signature_virtual_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    ed25519_signature_virtual_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    package_of_direct_caller_virtual_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    global_caller_virtual_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    system_transaction_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    package_owner_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    validator_owner_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    account_owner_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    identity_owner_badge: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    package_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    resource_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    account_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    identity_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    consensus_manager_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    access_controller_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    transaction_processor_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    metadata_module_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    royalty_module_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    role_assignment_module_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    genesis_helper_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    faucet_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    pool_package: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    locker_package?: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    consensus_manager: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    genesis_helper: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    faucet: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    transaction_tracker: string;
}

/**
 * Check if a given object implements the NetworkConfigurationResponseWellKnownAddresses interface.
 */
export function instanceOfNetworkConfigurationResponseWellKnownAddresses(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "xrd" in value;
    isInstance = isInstance && "secp256k1_signature_virtual_badge" in value;
    isInstance = isInstance && "ed25519_signature_virtual_badge" in value;
    isInstance = isInstance && "package_of_direct_caller_virtual_badge" in value;
    isInstance = isInstance && "global_caller_virtual_badge" in value;
    isInstance = isInstance && "system_transaction_badge" in value;
    isInstance = isInstance && "package_owner_badge" in value;
    isInstance = isInstance && "validator_owner_badge" in value;
    isInstance = isInstance && "account_owner_badge" in value;
    isInstance = isInstance && "identity_owner_badge" in value;
    isInstance = isInstance && "package_package" in value;
    isInstance = isInstance && "resource_package" in value;
    isInstance = isInstance && "account_package" in value;
    isInstance = isInstance && "identity_package" in value;
    isInstance = isInstance && "consensus_manager_package" in value;
    isInstance = isInstance && "access_controller_package" in value;
    isInstance = isInstance && "transaction_processor_package" in value;
    isInstance = isInstance && "metadata_module_package" in value;
    isInstance = isInstance && "royalty_module_package" in value;
    isInstance = isInstance && "role_assignment_module_package" in value;
    isInstance = isInstance && "genesis_helper_package" in value;
    isInstance = isInstance && "faucet_package" in value;
    isInstance = isInstance && "pool_package" in value;
    isInstance = isInstance && "consensus_manager" in value;
    isInstance = isInstance && "genesis_helper" in value;
    isInstance = isInstance && "faucet" in value;
    isInstance = isInstance && "transaction_tracker" in value;

    return isInstance;
}

export function NetworkConfigurationResponseWellKnownAddressesFromJSON(json: any): NetworkConfigurationResponseWellKnownAddresses {
    return NetworkConfigurationResponseWellKnownAddressesFromJSONTyped(json, false);
}

export function NetworkConfigurationResponseWellKnownAddressesFromJSONTyped(json: any, ignoreDiscriminator: boolean): NetworkConfigurationResponseWellKnownAddresses {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'xrd': json['xrd'],
        'secp256k1_signature_virtual_badge': json['secp256k1_signature_virtual_badge'],
        'ed25519_signature_virtual_badge': json['ed25519_signature_virtual_badge'],
        'package_of_direct_caller_virtual_badge': json['package_of_direct_caller_virtual_badge'],
        'global_caller_virtual_badge': json['global_caller_virtual_badge'],
        'system_transaction_badge': json['system_transaction_badge'],
        'package_owner_badge': json['package_owner_badge'],
        'validator_owner_badge': json['validator_owner_badge'],
        'account_owner_badge': json['account_owner_badge'],
        'identity_owner_badge': json['identity_owner_badge'],
        'package_package': json['package_package'],
        'resource_package': json['resource_package'],
        'account_package': json['account_package'],
        'identity_package': json['identity_package'],
        'consensus_manager_package': json['consensus_manager_package'],
        'access_controller_package': json['access_controller_package'],
        'transaction_processor_package': json['transaction_processor_package'],
        'metadata_module_package': json['metadata_module_package'],
        'royalty_module_package': json['royalty_module_package'],
        'role_assignment_module_package': json['role_assignment_module_package'],
        'genesis_helper_package': json['genesis_helper_package'],
        'faucet_package': json['faucet_package'],
        'pool_package': json['pool_package'],
        'locker_package': !exists(json, 'locker_package') ? undefined : json['locker_package'],
        'consensus_manager': json['consensus_manager'],
        'genesis_helper': json['genesis_helper'],
        'faucet': json['faucet'],
        'transaction_tracker': json['transaction_tracker'],
    };
}

export function NetworkConfigurationResponseWellKnownAddressesToJSON(value?: NetworkConfigurationResponseWellKnownAddresses | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'xrd': value.xrd,
        'secp256k1_signature_virtual_badge': value.secp256k1_signature_virtual_badge,
        'ed25519_signature_virtual_badge': value.ed25519_signature_virtual_badge,
        'package_of_direct_caller_virtual_badge': value.package_of_direct_caller_virtual_badge,
        'global_caller_virtual_badge': value.global_caller_virtual_badge,
        'system_transaction_badge': value.system_transaction_badge,
        'package_owner_badge': value.package_owner_badge,
        'validator_owner_badge': value.validator_owner_badge,
        'account_owner_badge': value.account_owner_badge,
        'identity_owner_badge': value.identity_owner_badge,
        'package_package': value.package_package,
        'resource_package': value.resource_package,
        'account_package': value.account_package,
        'identity_package': value.identity_package,
        'consensus_manager_package': value.consensus_manager_package,
        'access_controller_package': value.access_controller_package,
        'transaction_processor_package': value.transaction_processor_package,
        'metadata_module_package': value.metadata_module_package,
        'royalty_module_package': value.royalty_module_package,
        'role_assignment_module_package': value.role_assignment_module_package,
        'genesis_helper_package': value.genesis_helper_package,
        'faucet_package': value.faucet_package,
        'pool_package': value.pool_package,
        'locker_package': value.locker_package,
        'consensus_manager': value.consensus_manager,
        'genesis_helper': value.genesis_helper,
        'faucet': value.faucet,
        'transaction_tracker': value.transaction_tracker,
    };
}

