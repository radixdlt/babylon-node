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
import type { ObjectSubstateTypeReference } from './ObjectSubstateTypeReference';
import {
    ObjectSubstateTypeReferenceFromJSON,
    ObjectSubstateTypeReferenceFromJSONTyped,
    ObjectSubstateTypeReferenceToJSON,
} from './ObjectSubstateTypeReference';

/**
 * 
 * @export
 * @interface KeyValueBasedStructure
 */
export interface KeyValueBasedStructure {
    /**
     * 
     * @type {ObjectSubstateTypeReference}
     * @memberof KeyValueBasedStructure
     */
    key_schema: ObjectSubstateTypeReference;
    /**
     * 
     * @type {ObjectSubstateTypeReference}
     * @memberof KeyValueBasedStructure
     */
    value_schema: ObjectSubstateTypeReference;
}

/**
 * Check if a given object implements the KeyValueBasedStructure interface.
 */
export function instanceOfKeyValueBasedStructure(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "key_schema" in value;
    isInstance = isInstance && "value_schema" in value;

    return isInstance;
}

export function KeyValueBasedStructureFromJSON(json: any): KeyValueBasedStructure {
    return KeyValueBasedStructureFromJSONTyped(json, false);
}

export function KeyValueBasedStructureFromJSONTyped(json: any, ignoreDiscriminator: boolean): KeyValueBasedStructure {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'key_schema': ObjectSubstateTypeReferenceFromJSON(json['key_schema']),
        'value_schema': ObjectSubstateTypeReferenceFromJSON(json['value_schema']),
    };
}

export function KeyValueBasedStructureToJSON(value?: KeyValueBasedStructure | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'key_schema': ObjectSubstateTypeReferenceToJSON(value.key_schema),
        'value_schema': ObjectSubstateTypeReferenceToJSON(value.value_schema),
    };
}

