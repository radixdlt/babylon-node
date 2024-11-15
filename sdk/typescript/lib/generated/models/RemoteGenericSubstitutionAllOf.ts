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
import type { BlueprintTypeIdentifier } from './BlueprintTypeIdentifier';
import {
    BlueprintTypeIdentifierFromJSON,
    BlueprintTypeIdentifierFromJSONTyped,
    BlueprintTypeIdentifierToJSON,
} from './BlueprintTypeIdentifier';
import type { FullyScopedTypeId } from './FullyScopedTypeId';
import {
    FullyScopedTypeIdFromJSON,
    FullyScopedTypeIdFromJSONTyped,
    FullyScopedTypeIdToJSON,
} from './FullyScopedTypeId';

/**
 * The generic substitution is provided remotely by a blueprint type.
 * The `resolved_full_type_id` is added by the node, and is always present in the model returned from the transaction stream API.
 * Other APIs may not resolve the type from the blueprint definition.
 * @export
 * @interface RemoteGenericSubstitutionAllOf
 */
export interface RemoteGenericSubstitutionAllOf {
    /**
     * 
     * @type {BlueprintTypeIdentifier}
     * @memberof RemoteGenericSubstitutionAllOf
     */
    blueprint_type_identifier: BlueprintTypeIdentifier;
    /**
     * 
     * @type {FullyScopedTypeId}
     * @memberof RemoteGenericSubstitutionAllOf
     */
    resolved_full_type_id?: FullyScopedTypeId;
    /**
     * 
     * @type {string}
     * @memberof RemoteGenericSubstitutionAllOf
     */
    type?: RemoteGenericSubstitutionAllOfTypeEnum;
}


/**
 * @export
 */
export const RemoteGenericSubstitutionAllOfTypeEnum = {
    Remote: 'Remote'
} as const;
export type RemoteGenericSubstitutionAllOfTypeEnum = typeof RemoteGenericSubstitutionAllOfTypeEnum[keyof typeof RemoteGenericSubstitutionAllOfTypeEnum];


/**
 * Check if a given object implements the RemoteGenericSubstitutionAllOf interface.
 */
export function instanceOfRemoteGenericSubstitutionAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "blueprint_type_identifier" in value;

    return isInstance;
}

export function RemoteGenericSubstitutionAllOfFromJSON(json: any): RemoteGenericSubstitutionAllOf {
    return RemoteGenericSubstitutionAllOfFromJSONTyped(json, false);
}

export function RemoteGenericSubstitutionAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): RemoteGenericSubstitutionAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'blueprint_type_identifier': BlueprintTypeIdentifierFromJSON(json['blueprint_type_identifier']),
        'resolved_full_type_id': !exists(json, 'resolved_full_type_id') ? undefined : FullyScopedTypeIdFromJSON(json['resolved_full_type_id']),
        'type': !exists(json, 'type') ? undefined : json['type'],
    };
}

export function RemoteGenericSubstitutionAllOfToJSON(value?: RemoteGenericSubstitutionAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'blueprint_type_identifier': BlueprintTypeIdentifierToJSON(value.blueprint_type_identifier),
        'resolved_full_type_id': FullyScopedTypeIdToJSON(value.resolved_full_type_id),
        'type': value.type,
    };
}

