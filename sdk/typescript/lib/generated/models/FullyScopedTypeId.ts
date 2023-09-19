/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.1
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { LocalTypeId } from './LocalTypeId';
import {
    LocalTypeIdFromJSON,
    LocalTypeIdFromJSONTyped,
    LocalTypeIdToJSON,
} from './LocalTypeId';

/**
 * An identifier for a type in the context of a schema in an entity's schema partition.
 * 
 * Note - this type provides a schema context even for well-known types where this context
 * is effectively irrelevant.
 * @export
 * @interface FullyScopedTypeId
 */
export interface FullyScopedTypeId {
    /**
     * Bech32m-encoded human readable version of the entity's address (ie the entity's node id)
     * @type {string}
     * @memberof FullyScopedTypeId
     */
    entity_address: string;
    /**
     * The hex-encoded schema hash, capturing the identity of an SBOR schema.
     * @type {string}
     * @memberof FullyScopedTypeId
     */
    schema_hash: string;
    /**
     * 
     * @type {LocalTypeId}
     * @memberof FullyScopedTypeId
     */
    local_type_id: LocalTypeId;
}

/**
 * Check if a given object implements the FullyScopedTypeId interface.
 */
export function instanceOfFullyScopedTypeId(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "entity_address" in value;
    isInstance = isInstance && "schema_hash" in value;
    isInstance = isInstance && "local_type_id" in value;

    return isInstance;
}

export function FullyScopedTypeIdFromJSON(json: any): FullyScopedTypeId {
    return FullyScopedTypeIdFromJSONTyped(json, false);
}

export function FullyScopedTypeIdFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullyScopedTypeId {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'entity_address': json['entity_address'],
        'schema_hash': json['schema_hash'],
        'local_type_id': LocalTypeIdFromJSON(json['local_type_id']),
    };
}

export function FullyScopedTypeIdToJSON(value?: FullyScopedTypeId | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'entity_address': value.entity_address,
        'schema_hash': value.schema_hash,
        'local_type_id': LocalTypeIdToJSON(value.local_type_id),
    };
}
