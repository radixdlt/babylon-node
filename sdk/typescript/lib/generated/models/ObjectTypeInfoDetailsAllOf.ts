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
import type { BlueprintInfo } from './BlueprintInfo';
import {
    BlueprintInfoFromJSON,
    BlueprintInfoFromJSONTyped,
    BlueprintInfoToJSON,
} from './BlueprintInfo';
import type { ModuleVersion } from './ModuleVersion';
import {
    ModuleVersionFromJSON,
    ModuleVersionFromJSONTyped,
    ModuleVersionToJSON,
} from './ModuleVersion';

/**
 * 
 * @export
 * @interface ObjectTypeInfoDetailsAllOf
 */
export interface ObjectTypeInfoDetailsAllOf {
    /**
     * 
     * @type {Array<ModuleVersion>}
     * @memberof ObjectTypeInfoDetailsAllOf
     */
    module_versions: Array<ModuleVersion>;
    /**
     * 
     * @type {BlueprintInfo}
     * @memberof ObjectTypeInfoDetailsAllOf
     */
    blueprint_info: BlueprintInfo;
    /**
     * 
     * @type {boolean}
     * @memberof ObjectTypeInfoDetailsAllOf
     */
    global: boolean;
    /**
     * 
     * @type {string}
     * @memberof ObjectTypeInfoDetailsAllOf
     */
    type?: ObjectTypeInfoDetailsAllOfTypeEnum;
}


/**
 * @export
 */
export const ObjectTypeInfoDetailsAllOfTypeEnum = {
    Object: 'Object'
} as const;
export type ObjectTypeInfoDetailsAllOfTypeEnum = typeof ObjectTypeInfoDetailsAllOfTypeEnum[keyof typeof ObjectTypeInfoDetailsAllOfTypeEnum];


/**
 * Check if a given object implements the ObjectTypeInfoDetailsAllOf interface.
 */
export function instanceOfObjectTypeInfoDetailsAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "module_versions" in value;
    isInstance = isInstance && "blueprint_info" in value;
    isInstance = isInstance && "global" in value;

    return isInstance;
}

export function ObjectTypeInfoDetailsAllOfFromJSON(json: any): ObjectTypeInfoDetailsAllOf {
    return ObjectTypeInfoDetailsAllOfFromJSONTyped(json, false);
}

export function ObjectTypeInfoDetailsAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): ObjectTypeInfoDetailsAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'module_versions': ((json['module_versions'] as Array<any>).map(ModuleVersionFromJSON)),
        'blueprint_info': BlueprintInfoFromJSON(json['blueprint_info']),
        'global': json['global'],
        'type': !exists(json, 'type') ? undefined : json['type'],
    };
}

export function ObjectTypeInfoDetailsAllOfToJSON(value?: ObjectTypeInfoDetailsAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'module_versions': ((value.module_versions as Array<any>).map(ModuleVersionToJSON)),
        'blueprint_info': BlueprintInfoToJSON(value.blueprint_info),
        'global': value.global,
        'type': value.type,
    };
}

