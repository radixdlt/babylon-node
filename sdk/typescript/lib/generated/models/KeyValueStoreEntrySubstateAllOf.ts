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
import type { DataStruct } from './DataStruct';
import {
    DataStructFromJSON,
    DataStructFromJSONTyped,
    DataStructToJSON,
} from './DataStruct';
import type { NonFungibleId } from './NonFungibleId';
import {
    NonFungibleIdFromJSON,
    NonFungibleIdFromJSONTyped,
    NonFungibleIdToJSON,
} from './NonFungibleId';

/**
 * 
 * @export
 * @interface KeyValueStoreEntrySubstateAllOf
 */
export interface KeyValueStoreEntrySubstateAllOf {
    /**
     * The hex-encoded bytes of its key
     * @type {string}
     * @memberof KeyValueStoreEntrySubstateAllOf
     */
    key_hex: string;
    /**
     * 
     * @type {NonFungibleId}
     * @memberof KeyValueStoreEntrySubstateAllOf
     */
    key_non_fungible_local_id?: NonFungibleId;
    /**
     * 
     * @type {boolean}
     * @memberof KeyValueStoreEntrySubstateAllOf
     */
    is_deleted: boolean;
    /**
     * 
     * @type {DataStruct}
     * @memberof KeyValueStoreEntrySubstateAllOf
     */
    data_struct?: DataStruct;
    /**
     * 
     * @type {string}
     * @memberof KeyValueStoreEntrySubstateAllOf
     */
    substate_type?: KeyValueStoreEntrySubstateAllOfSubstateTypeEnum;
}


/**
 * @export
 */
export const KeyValueStoreEntrySubstateAllOfSubstateTypeEnum = {
    KeyValueStoreEntry: 'KeyValueStoreEntry'
} as const;
export type KeyValueStoreEntrySubstateAllOfSubstateTypeEnum = typeof KeyValueStoreEntrySubstateAllOfSubstateTypeEnum[keyof typeof KeyValueStoreEntrySubstateAllOfSubstateTypeEnum];


/**
 * Check if a given object implements the KeyValueStoreEntrySubstateAllOf interface.
 */
export function instanceOfKeyValueStoreEntrySubstateAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "key_hex" in value;
    isInstance = isInstance && "is_deleted" in value;

    return isInstance;
}

export function KeyValueStoreEntrySubstateAllOfFromJSON(json: any): KeyValueStoreEntrySubstateAllOf {
    return KeyValueStoreEntrySubstateAllOfFromJSONTyped(json, false);
}

export function KeyValueStoreEntrySubstateAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): KeyValueStoreEntrySubstateAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'key_hex': json['key_hex'],
        'key_non_fungible_local_id': !exists(json, 'key_non_fungible_local_id') ? undefined : NonFungibleIdFromJSON(json['key_non_fungible_local_id']),
        'is_deleted': json['is_deleted'],
        'data_struct': !exists(json, 'data_struct') ? undefined : DataStructFromJSON(json['data_struct']),
        'substate_type': !exists(json, 'substate_type') ? undefined : json['substate_type'],
    };
}

export function KeyValueStoreEntrySubstateAllOfToJSON(value?: KeyValueStoreEntrySubstateAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'key_hex': value.key_hex,
        'key_non_fungible_local_id': NonFungibleIdToJSON(value.key_non_fungible_local_id),
        'is_deleted': value.is_deleted,
        'data_struct': DataStructToJSON(value.data_struct),
        'substate_type': value.substate_type,
    };
}
