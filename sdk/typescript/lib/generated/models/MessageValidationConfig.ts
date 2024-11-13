/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.3.0
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
 * @interface MessageValidationConfig
 */
export interface MessageValidationConfig {
    /**
     * 
     * @type {string}
     * @memberof MessageValidationConfig
     */
    max_plaintext_message_length: string;
    /**
     * 
     * @type {string}
     * @memberof MessageValidationConfig
     */
    max_encrypted_message_length: string;
    /**
     * 
     * @type {string}
     * @memberof MessageValidationConfig
     */
    max_mime_type_length: string;
    /**
     * 
     * @type {string}
     * @memberof MessageValidationConfig
     */
    max_decryptors: string;
}

/**
 * Check if a given object implements the MessageValidationConfig interface.
 */
export function instanceOfMessageValidationConfig(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "max_plaintext_message_length" in value;
    isInstance = isInstance && "max_encrypted_message_length" in value;
    isInstance = isInstance && "max_mime_type_length" in value;
    isInstance = isInstance && "max_decryptors" in value;

    return isInstance;
}

export function MessageValidationConfigFromJSON(json: any): MessageValidationConfig {
    return MessageValidationConfigFromJSONTyped(json, false);
}

export function MessageValidationConfigFromJSONTyped(json: any, ignoreDiscriminator: boolean): MessageValidationConfig {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'max_plaintext_message_length': json['max_plaintext_message_length'],
        'max_encrypted_message_length': json['max_encrypted_message_length'],
        'max_mime_type_length': json['max_mime_type_length'],
        'max_decryptors': json['max_decryptors'],
    };
}

export function MessageValidationConfigToJSON(value?: MessageValidationConfig | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'max_plaintext_message_length': value.max_plaintext_message_length,
        'max_encrypted_message_length': value.max_encrypted_message_length,
        'max_mime_type_length': value.max_mime_type_length,
        'max_decryptors': value.max_decryptors,
    };
}

