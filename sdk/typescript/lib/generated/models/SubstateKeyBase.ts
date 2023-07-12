/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet V2
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.4.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { SubstateKeyType } from './SubstateKeyType';
import {
    SubstateKeyTypeFromJSON,
    SubstateKeyTypeFromJSONTyped,
    SubstateKeyTypeToJSON,
} from './SubstateKeyType';

/**
 * 
 * @export
 * @interface SubstateKeyBase
 */
export interface SubstateKeyBase {
    /**
     * 
     * @type {SubstateKeyType}
     * @memberof SubstateKeyBase
     */
    key_type: SubstateKeyType;
    /**
     * The hex-encoded bytes of the partially-hashed DB sort key, under the given entity partition
     * @type {string}
     * @memberof SubstateKeyBase
     */
    db_sort_key_hex: string;
}

/**
 * Check if a given object implements the SubstateKeyBase interface.
 */
export function instanceOfSubstateKeyBase(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "key_type" in value;
    isInstance = isInstance && "db_sort_key_hex" in value;

    return isInstance;
}

export function SubstateKeyBaseFromJSON(json: any): SubstateKeyBase {
    return SubstateKeyBaseFromJSONTyped(json, false);
}

export function SubstateKeyBaseFromJSONTyped(json: any, ignoreDiscriminator: boolean): SubstateKeyBase {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'key_type': SubstateKeyTypeFromJSON(json['key_type']),
        'db_sort_key_hex': json['db_sort_key_hex'],
    };
}

export function SubstateKeyBaseToJSON(value?: SubstateKeyBase | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'key_type': SubstateKeyTypeToJSON(value.key_type),
        'db_sort_key_hex': value.db_sort_key_hex,
    };
}
