/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
/**
 * 
 * @export
 * @interface EcdsaSecp256k1Signature
 */
export interface EcdsaSecp256k1Signature {
    /**
     * 
     * @type {string}
     * @memberof EcdsaSecp256k1Signature
     */
    key_type: EcdsaSecp256k1SignatureKeyTypeEnum;
    /**
     * A hex-encoded recoverable ECDSA Secp256k1 signature (65 bytes). The first byte is the recovery id, the remaining 64 bytes are the compact signature, ie `CONCAT(R, s)` where `R` and `s` are each 32-bytes in padded big-endian format.
     * @type {string}
     * @memberof EcdsaSecp256k1Signature
     */
    signature_hex: string;
}


/**
 * @export
 */
export const EcdsaSecp256k1SignatureKeyTypeEnum = {
    EcdsaSecp256k1: 'EcdsaSecp256k1'
} as const;
export type EcdsaSecp256k1SignatureKeyTypeEnum = typeof EcdsaSecp256k1SignatureKeyTypeEnum[keyof typeof EcdsaSecp256k1SignatureKeyTypeEnum];


/**
 * Check if a given object implements the EcdsaSecp256k1Signature interface.
 */
export function instanceOfEcdsaSecp256k1Signature(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "key_type" in value;
    isInstance = isInstance && "signature_hex" in value;

    return isInstance;
}

export function EcdsaSecp256k1SignatureFromJSON(json: any): EcdsaSecp256k1Signature {
    return EcdsaSecp256k1SignatureFromJSONTyped(json, false);
}

export function EcdsaSecp256k1SignatureFromJSONTyped(json: any, ignoreDiscriminator: boolean): EcdsaSecp256k1Signature {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'key_type': json['key_type'],
        'signature_hex': json['signature_hex'],
    };
}

export function EcdsaSecp256k1SignatureToJSON(value?: EcdsaSecp256k1Signature | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'key_type': value.key_type,
        'signature_hex': value.signature_hex,
    };
}

