/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon (Anemone)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.1.3
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { EcdsaSecp256k1Signature } from './EcdsaSecp256k1Signature';
import {
    EcdsaSecp256k1SignatureFromJSON,
    EcdsaSecp256k1SignatureFromJSONTyped,
    EcdsaSecp256k1SignatureToJSON,
} from './EcdsaSecp256k1Signature';

/**
 * 
 * @export
 * @interface EcdsaSecp256k1SignatureWithPublicKey
 */
export interface EcdsaSecp256k1SignatureWithPublicKey {
    /**
     * 
     * @type {string}
     * @memberof EcdsaSecp256k1SignatureWithPublicKey
     */
    key_type: EcdsaSecp256k1SignatureWithPublicKeyKeyTypeEnum;
    /**
     * 
     * @type {EcdsaSecp256k1Signature}
     * @memberof EcdsaSecp256k1SignatureWithPublicKey
     */
    recoverable_signature: EcdsaSecp256k1Signature;
}


/**
 * @export
 */
export const EcdsaSecp256k1SignatureWithPublicKeyKeyTypeEnum = {
    EcdsaSecp256k1: 'EcdsaSecp256k1'
} as const;
export type EcdsaSecp256k1SignatureWithPublicKeyKeyTypeEnum = typeof EcdsaSecp256k1SignatureWithPublicKeyKeyTypeEnum[keyof typeof EcdsaSecp256k1SignatureWithPublicKeyKeyTypeEnum];


/**
 * Check if a given object implements the EcdsaSecp256k1SignatureWithPublicKey interface.
 */
export function instanceOfEcdsaSecp256k1SignatureWithPublicKey(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "key_type" in value;
    isInstance = isInstance && "recoverable_signature" in value;

    return isInstance;
}

export function EcdsaSecp256k1SignatureWithPublicKeyFromJSON(json: any): EcdsaSecp256k1SignatureWithPublicKey {
    return EcdsaSecp256k1SignatureWithPublicKeyFromJSONTyped(json, false);
}

export function EcdsaSecp256k1SignatureWithPublicKeyFromJSONTyped(json: any, ignoreDiscriminator: boolean): EcdsaSecp256k1SignatureWithPublicKey {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'key_type': json['key_type'],
        'recoverable_signature': EcdsaSecp256k1SignatureFromJSON(json['recoverable_signature']),
    };
}

export function EcdsaSecp256k1SignatureWithPublicKeyToJSON(value?: EcdsaSecp256k1SignatureWithPublicKey | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'key_type': value.key_type,
        'recoverable_signature': EcdsaSecp256k1SignatureToJSON(value.recoverable_signature),
    };
}

