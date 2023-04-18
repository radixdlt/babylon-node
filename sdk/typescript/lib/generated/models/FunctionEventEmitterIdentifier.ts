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
import type { EntityReference } from './EntityReference';
import {
    EntityReferenceFromJSON,
    EntityReferenceFromJSONTyped,
    EntityReferenceToJSON,
} from './EntityReference';
import type { ModuleType } from './ModuleType';
import {
    ModuleTypeFromJSON,
    ModuleTypeFromJSONTyped,
    ModuleTypeToJSON,
} from './ModuleType';

/**
 * 
 * @export
 * @interface FunctionEventEmitterIdentifier
 */
export interface FunctionEventEmitterIdentifier {
    /**
     * 
     * @type {string}
     * @memberof FunctionEventEmitterIdentifier
     */
    type: FunctionEventEmitterIdentifierTypeEnum;
    /**
     * Blueprint name.
     * @type {string}
     * @memberof FunctionEventEmitterIdentifier
     */
    blueprint_name: string;
    /**
     * 
     * @type {EntityReference}
     * @memberof FunctionEventEmitterIdentifier
     */
    entity: EntityReference;
    /**
     * 
     * @type {ModuleType}
     * @memberof FunctionEventEmitterIdentifier
     */
    module_type: ModuleType;
}


/**
 * @export
 */
export const FunctionEventEmitterIdentifierTypeEnum = {
    Function: 'Function'
} as const;
export type FunctionEventEmitterIdentifierTypeEnum = typeof FunctionEventEmitterIdentifierTypeEnum[keyof typeof FunctionEventEmitterIdentifierTypeEnum];


/**
 * Check if a given object implements the FunctionEventEmitterIdentifier interface.
 */
export function instanceOfFunctionEventEmitterIdentifier(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "type" in value;
    isInstance = isInstance && "blueprint_name" in value;
    isInstance = isInstance && "entity" in value;
    isInstance = isInstance && "module_type" in value;

    return isInstance;
}

export function FunctionEventEmitterIdentifierFromJSON(json: any): FunctionEventEmitterIdentifier {
    return FunctionEventEmitterIdentifierFromJSONTyped(json, false);
}

export function FunctionEventEmitterIdentifierFromJSONTyped(json: any, ignoreDiscriminator: boolean): FunctionEventEmitterIdentifier {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'type': json['type'],
        'blueprint_name': json['blueprint_name'],
        'entity': EntityReferenceFromJSON(json['entity']),
        'module_type': ModuleTypeFromJSON(json['module_type']),
    };
}

export function FunctionEventEmitterIdentifierToJSON(value?: FunctionEventEmitterIdentifier | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'type': value.type,
        'blueprint_name': value.blueprint_name,
        'entity': EntityReferenceToJSON(value.entity),
        'module_type': ModuleTypeToJSON(value.module_type),
    };
}

