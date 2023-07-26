/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet v3
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the second release candidate of the Radix Babylon network (\"RCnet v3\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { EntityReference } from './EntityReference';
import {
    EntityReferenceFromJSON,
    EntityReferenceFromJSONTyped,
    EntityReferenceToJSON,
} from './EntityReference';
import type { ObjectModuleId } from './ObjectModuleId';
import {
    ObjectModuleIdFromJSON,
    ObjectModuleIdFromJSONTyped,
    ObjectModuleIdToJSON,
} from './ObjectModuleId';

/**
 * 
 * @export
 * @interface FunctionEventEmitterIdentifierAllOf
 */
export interface FunctionEventEmitterIdentifierAllOf {
    /**
     * 
     * @type {EntityReference}
     * @memberof FunctionEventEmitterIdentifierAllOf
     */
    entity: EntityReference;
    /**
     * 
     * @type {ObjectModuleId}
     * @memberof FunctionEventEmitterIdentifierAllOf
     */
    object_module_id: ObjectModuleId;
    /**
     * Blueprint name.
     * @type {string}
     * @memberof FunctionEventEmitterIdentifierAllOf
     */
    blueprint_name: string;
    /**
     * 
     * @type {string}
     * @memberof FunctionEventEmitterIdentifierAllOf
     */
    type?: FunctionEventEmitterIdentifierAllOfTypeEnum;
}


/**
 * @export
 */
export const FunctionEventEmitterIdentifierAllOfTypeEnum = {
    Function: 'Function'
} as const;
export type FunctionEventEmitterIdentifierAllOfTypeEnum = typeof FunctionEventEmitterIdentifierAllOfTypeEnum[keyof typeof FunctionEventEmitterIdentifierAllOfTypeEnum];


/**
 * Check if a given object implements the FunctionEventEmitterIdentifierAllOf interface.
 */
export function instanceOfFunctionEventEmitterIdentifierAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "entity" in value;
    isInstance = isInstance && "object_module_id" in value;
    isInstance = isInstance && "blueprint_name" in value;

    return isInstance;
}

export function FunctionEventEmitterIdentifierAllOfFromJSON(json: any): FunctionEventEmitterIdentifierAllOf {
    return FunctionEventEmitterIdentifierAllOfFromJSONTyped(json, false);
}

export function FunctionEventEmitterIdentifierAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): FunctionEventEmitterIdentifierAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'entity': EntityReferenceFromJSON(json['entity']),
        'object_module_id': ObjectModuleIdFromJSON(json['object_module_id']),
        'blueprint_name': json['blueprint_name'],
        'type': !exists(json, 'type') ? undefined : json['type'],
    };
}

export function FunctionEventEmitterIdentifierAllOfToJSON(value?: FunctionEventEmitterIdentifierAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'entity': EntityReferenceToJSON(value.entity),
        'object_module_id': ObjectModuleIdToJSON(value.object_module_id),
        'blueprint_name': value.blueprint_name,
        'type': value.type,
    };
}

