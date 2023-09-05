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
import type { NonFungibleVaultFieldFrozenStatusValue } from './NonFungibleVaultFieldFrozenStatusValue';
import {
    NonFungibleVaultFieldFrozenStatusValueFromJSON,
    NonFungibleVaultFieldFrozenStatusValueFromJSONTyped,
    NonFungibleVaultFieldFrozenStatusValueToJSON,
} from './NonFungibleVaultFieldFrozenStatusValue';

/**
 * 
 * @export
 * @interface NonFungibleVaultFieldFrozenStatusSubstateAllOf
 */
export interface NonFungibleVaultFieldFrozenStatusSubstateAllOf {
    /**
     * 
     * @type {NonFungibleVaultFieldFrozenStatusValue}
     * @memberof NonFungibleVaultFieldFrozenStatusSubstateAllOf
     */
    value: NonFungibleVaultFieldFrozenStatusValue;
    /**
     * 
     * @type {string}
     * @memberof NonFungibleVaultFieldFrozenStatusSubstateAllOf
     */
    substate_type?: NonFungibleVaultFieldFrozenStatusSubstateAllOfSubstateTypeEnum;
}


/**
 * @export
 */
export const NonFungibleVaultFieldFrozenStatusSubstateAllOfSubstateTypeEnum = {
    NonFungibleVaultFieldFrozenStatus: 'NonFungibleVaultFieldFrozenStatus'
} as const;
export type NonFungibleVaultFieldFrozenStatusSubstateAllOfSubstateTypeEnum = typeof NonFungibleVaultFieldFrozenStatusSubstateAllOfSubstateTypeEnum[keyof typeof NonFungibleVaultFieldFrozenStatusSubstateAllOfSubstateTypeEnum];


/**
 * Check if a given object implements the NonFungibleVaultFieldFrozenStatusSubstateAllOf interface.
 */
export function instanceOfNonFungibleVaultFieldFrozenStatusSubstateAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "value" in value;

    return isInstance;
}

export function NonFungibleVaultFieldFrozenStatusSubstateAllOfFromJSON(json: any): NonFungibleVaultFieldFrozenStatusSubstateAllOf {
    return NonFungibleVaultFieldFrozenStatusSubstateAllOfFromJSONTyped(json, false);
}

export function NonFungibleVaultFieldFrozenStatusSubstateAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): NonFungibleVaultFieldFrozenStatusSubstateAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'value': NonFungibleVaultFieldFrozenStatusValueFromJSON(json['value']),
        'substate_type': !exists(json, 'substate_type') ? undefined : json['substate_type'],
    };
}

export function NonFungibleVaultFieldFrozenStatusSubstateAllOfToJSON(value?: NonFungibleVaultFieldFrozenStatusSubstateAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'value': NonFungibleVaultFieldFrozenStatusValueToJSON(value.value),
        'substate_type': value.substate_type,
    };
}

