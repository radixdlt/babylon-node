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
import type { LocalNonFungibleKey } from './LocalNonFungibleKey';
import {
    LocalNonFungibleKeyFromJSON,
    LocalNonFungibleKeyFromJSONTyped,
    LocalNonFungibleKeyToJSON,
} from './LocalNonFungibleKey';
import type { NonFungibleVaultContentsIndexEntryValue } from './NonFungibleVaultContentsIndexEntryValue';
import {
    NonFungibleVaultContentsIndexEntryValueFromJSON,
    NonFungibleVaultContentsIndexEntryValueFromJSONTyped,
    NonFungibleVaultContentsIndexEntryValueToJSON,
} from './NonFungibleVaultContentsIndexEntryValue';

/**
 * 
 * @export
 * @interface NonFungibleVaultContentsIndexEntrySubstate
 */
export interface NonFungibleVaultContentsIndexEntrySubstate {
    /**
     * 
     * @type {string}
     * @memberof NonFungibleVaultContentsIndexEntrySubstate
     */
    substate_type: NonFungibleVaultContentsIndexEntrySubstateSubstateTypeEnum;
    /**
     * 
     * @type {boolean}
     * @memberof NonFungibleVaultContentsIndexEntrySubstate
     */
    is_locked: boolean;
    /**
     * 
     * @type {LocalNonFungibleKey}
     * @memberof NonFungibleVaultContentsIndexEntrySubstate
     */
    key: LocalNonFungibleKey;
    /**
     * 
     * @type {NonFungibleVaultContentsIndexEntryValue}
     * @memberof NonFungibleVaultContentsIndexEntrySubstate
     */
    value: NonFungibleVaultContentsIndexEntryValue;
}


/**
 * @export
 */
export const NonFungibleVaultContentsIndexEntrySubstateSubstateTypeEnum = {
    NonFungibleVaultContentsIndexEntry: 'NonFungibleVaultContentsIndexEntry'
} as const;
export type NonFungibleVaultContentsIndexEntrySubstateSubstateTypeEnum = typeof NonFungibleVaultContentsIndexEntrySubstateSubstateTypeEnum[keyof typeof NonFungibleVaultContentsIndexEntrySubstateSubstateTypeEnum];


/**
 * Check if a given object implements the NonFungibleVaultContentsIndexEntrySubstate interface.
 */
export function instanceOfNonFungibleVaultContentsIndexEntrySubstate(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "substate_type" in value;
    isInstance = isInstance && "is_locked" in value;
    isInstance = isInstance && "key" in value;
    isInstance = isInstance && "value" in value;

    return isInstance;
}

export function NonFungibleVaultContentsIndexEntrySubstateFromJSON(json: any): NonFungibleVaultContentsIndexEntrySubstate {
    return NonFungibleVaultContentsIndexEntrySubstateFromJSONTyped(json, false);
}

export function NonFungibleVaultContentsIndexEntrySubstateFromJSONTyped(json: any, ignoreDiscriminator: boolean): NonFungibleVaultContentsIndexEntrySubstate {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'substate_type': json['substate_type'],
        'is_locked': json['is_locked'],
        'key': LocalNonFungibleKeyFromJSON(json['key']),
        'value': NonFungibleVaultContentsIndexEntryValueFromJSON(json['value']),
    };
}

export function NonFungibleVaultContentsIndexEntrySubstateToJSON(value?: NonFungibleVaultContentsIndexEntrySubstate | null): any {
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
        'value': NonFungibleVaultContentsIndexEntryValueToJSON(value.value),
    };
}

