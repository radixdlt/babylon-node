/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { EcdsaSecp256k1PublicKey } from './EcdsaSecp256k1PublicKey';
import {
    EcdsaSecp256k1PublicKeyFromJSON,
    EcdsaSecp256k1PublicKeyFromJSONTyped,
    EcdsaSecp256k1PublicKeyToJSON,
} from './EcdsaSecp256k1PublicKey';

/**
 * 
 * @export
 * @interface ActiveValidator
 */
export interface ActiveValidator {
    /**
     * The Bech32m-encoded human readable version of the component address
     * @type {string}
     * @memberof ActiveValidator
     */
    address: string;
    /**
     * 
     * @type {EcdsaSecp256k1PublicKey}
     * @memberof ActiveValidator
     */
    key: EcdsaSecp256k1PublicKey;
    /**
     * A string-encoded decimal representing the validator's voting power for this epoch.
     * This is a snapshot of the amount of XRD staked to the validator at the start of the epoch.
     * @type {string}
     * @memberof ActiveValidator
     */
    stake: string;
}

/**
 * Check if a given object implements the ActiveValidator interface.
 */
export function instanceOfActiveValidator(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "address" in value;
    isInstance = isInstance && "key" in value;
    isInstance = isInstance && "stake" in value;

    return isInstance;
}

export function ActiveValidatorFromJSON(json: any): ActiveValidator {
    return ActiveValidatorFromJSONTyped(json, false);
}

export function ActiveValidatorFromJSONTyped(json: any, ignoreDiscriminator: boolean): ActiveValidator {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'address': json['address'],
        'key': EcdsaSecp256k1PublicKeyFromJSON(json['key']),
        'stake': json['stake'],
    };
}

export function ActiveValidatorToJSON(value?: ActiveValidator | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'address': value.address,
        'key': EcdsaSecp256k1PublicKeyToJSON(value.key),
        'stake': value.stake,
    };
}

