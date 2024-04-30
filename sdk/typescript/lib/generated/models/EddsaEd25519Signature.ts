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
/**
 * 
 * @export
 * @interface EddsaEd25519Signature
 */
export interface EddsaEd25519Signature {
    /**
     * 
     * @type {string}
     * @memberof EddsaEd25519Signature
     */
    key_type: EddsaEd25519SignatureKeyTypeEnum;
    /**
     * A hex-encoded EdDSA Ed25519 signature (64 bytes). This is `CONCAT(R, s)` where `R` and `s` are each 32-bytes in padded big-endian format.
     * @type {string}
     * @memberof EddsaEd25519Signature
     */
    signature_hex: string;
}


/**
 * @export
 */
export const EddsaEd25519SignatureKeyTypeEnum = {
    EddsaEd25519: 'EddsaEd25519'
} as const;
export type EddsaEd25519SignatureKeyTypeEnum = typeof EddsaEd25519SignatureKeyTypeEnum[keyof typeof EddsaEd25519SignatureKeyTypeEnum];


/**
 * Check if a given object implements the EddsaEd25519Signature interface.
 */
export function instanceOfEddsaEd25519Signature(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "key_type" in value;
    isInstance = isInstance && "signature_hex" in value;

    return isInstance;
}

export function EddsaEd25519SignatureFromJSON(json: any): EddsaEd25519Signature {
    return EddsaEd25519SignatureFromJSONTyped(json, false);
}

export function EddsaEd25519SignatureFromJSONTyped(json: any, ignoreDiscriminator: boolean): EddsaEd25519Signature {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'key_type': json['key_type'],
        'signature_hex': json['signature_hex'],
    };
}

export function EddsaEd25519SignatureToJSON(value?: EddsaEd25519Signature | null): any {
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

