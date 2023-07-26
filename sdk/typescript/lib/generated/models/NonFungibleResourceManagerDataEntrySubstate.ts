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
import type { LocalNonFungibleKey } from './LocalNonFungibleKey';
import {
    LocalNonFungibleKeyFromJSON,
    LocalNonFungibleKeyFromJSONTyped,
    LocalNonFungibleKeyToJSON,
} from './LocalNonFungibleKey';
import type { NonFungibleResourceManagerDataEntryValue } from './NonFungibleResourceManagerDataEntryValue';
import {
    NonFungibleResourceManagerDataEntryValueFromJSON,
    NonFungibleResourceManagerDataEntryValueFromJSONTyped,
    NonFungibleResourceManagerDataEntryValueToJSON,
} from './NonFungibleResourceManagerDataEntryValue';

/**
 * 
 * @export
 * @interface NonFungibleResourceManagerDataEntrySubstate
 */
export interface NonFungibleResourceManagerDataEntrySubstate {
    /**
     * 
     * @type {string}
     * @memberof NonFungibleResourceManagerDataEntrySubstate
     */
    substate_type: NonFungibleResourceManagerDataEntrySubstateSubstateTypeEnum;
    /**
     * 
     * @type {boolean}
     * @memberof NonFungibleResourceManagerDataEntrySubstate
     */
    is_locked: boolean;
    /**
     * 
     * @type {LocalNonFungibleKey}
     * @memberof NonFungibleResourceManagerDataEntrySubstate
     */
    key: LocalNonFungibleKey;
    /**
     * 
     * @type {NonFungibleResourceManagerDataEntryValue}
     * @memberof NonFungibleResourceManagerDataEntrySubstate
     */
    value?: NonFungibleResourceManagerDataEntryValue;
}


/**
 * @export
 */
export const NonFungibleResourceManagerDataEntrySubstateSubstateTypeEnum = {
    NonFungibleResourceManagerDataEntry: 'NonFungibleResourceManagerDataEntry'
} as const;
export type NonFungibleResourceManagerDataEntrySubstateSubstateTypeEnum = typeof NonFungibleResourceManagerDataEntrySubstateSubstateTypeEnum[keyof typeof NonFungibleResourceManagerDataEntrySubstateSubstateTypeEnum];


/**
 * Check if a given object implements the NonFungibleResourceManagerDataEntrySubstate interface.
 */
export function instanceOfNonFungibleResourceManagerDataEntrySubstate(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "substate_type" in value;
    isInstance = isInstance && "is_locked" in value;
    isInstance = isInstance && "key" in value;

    return isInstance;
}

export function NonFungibleResourceManagerDataEntrySubstateFromJSON(json: any): NonFungibleResourceManagerDataEntrySubstate {
    return NonFungibleResourceManagerDataEntrySubstateFromJSONTyped(json, false);
}

export function NonFungibleResourceManagerDataEntrySubstateFromJSONTyped(json: any, ignoreDiscriminator: boolean): NonFungibleResourceManagerDataEntrySubstate {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'substate_type': json['substate_type'],
        'is_locked': json['is_locked'],
        'key': LocalNonFungibleKeyFromJSON(json['key']),
        'value': !exists(json, 'value') ? undefined : NonFungibleResourceManagerDataEntryValueFromJSON(json['value']),
    };
}

export function NonFungibleResourceManagerDataEntrySubstateToJSON(value?: NonFungibleResourceManagerDataEntrySubstate | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'substate_type': value.substate_type,
        'is_locked': value.is_locked,
        'key': LocalNonFungibleKeyToJSON(value.key),
        'value': NonFungibleResourceManagerDataEntryValueToJSON(value.value),
    };
}

