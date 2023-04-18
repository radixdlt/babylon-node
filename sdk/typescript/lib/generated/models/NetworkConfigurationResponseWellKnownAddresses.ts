/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet V1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.3.0
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
    clock: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    ecdsa_secp256k1: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    eddsa_ed25519: string;
    /**
     * 
     * @type {string}
     * @memberof NetworkConfigurationResponseWellKnownAddresses
     */
    epoch_manager: string;
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
    xrd: string;
}

/**
 * Check if a given object implements the NetworkConfigurationResponseWellKnownAddresses interface.
 */
export function instanceOfNetworkConfigurationResponseWellKnownAddresses(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "clock" in value;
    isInstance = isInstance && "ecdsa_secp256k1" in value;
    isInstance = isInstance && "eddsa_ed25519" in value;
    isInstance = isInstance && "epoch_manager" in value;
    isInstance = isInstance && "faucet" in value;
    isInstance = isInstance && "xrd" in value;

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
        
        'clock': json['clock'],
        'ecdsa_secp256k1': json['ecdsa_secp256k1'],
        'eddsa_ed25519': json['eddsa_ed25519'],
        'epoch_manager': json['epoch_manager'],
        'faucet': json['faucet'],
        'xrd': json['xrd'],
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
        
        'clock': value.clock,
        'ecdsa_secp256k1': value.ecdsa_secp256k1,
        'eddsa_ed25519': value.eddsa_ed25519,
        'epoch_manager': value.epoch_manager,
        'faucet': value.faucet,
        'xrd': value.xrd,
    };
}

