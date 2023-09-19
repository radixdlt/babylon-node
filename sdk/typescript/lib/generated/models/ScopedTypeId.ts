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
 * An identifier for a type in the context of a schema.
 * 
 * The location of the schema store to locate the schema is not included, and
 * is known from context. Currently the schema store will be in the
 * schema partition under a node (typically a package).
 * 
 * Note - this type provides scoping to a schema even for well-known types where
 * the schema is irrelevant.
 * @export
 * @interface ScopedTypeId
 */
export interface ScopedTypeId {
    /**
     * The hex-encoded schema hash, capturing the identity of an SBOR schema.
     * @type {string}
     * @memberof ScopedTypeId
     */
    schema_hash: string;
    /**
     * 
     * @type {LocalTypeId}
     * @memberof ScopedTypeId
     */
    local_type_id: LocalTypeId;
}

/**
 * Check if a given object implements the ScopedTypeId interface.
 */
export function instanceOfScopedTypeId(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "schema_hash" in value;
    isInstance = isInstance && "local_type_id" in value;

    return isInstance;
}

export function ScopedTypeIdFromJSON(json: any): ScopedTypeId {
    return ScopedTypeIdFromJSONTyped(json, false);
}

export function ScopedTypeIdFromJSONTyped(json: any, ignoreDiscriminator: boolean): ScopedTypeId {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'schema_hash': json['schema_hash'],
        'local_type_id': LocalTypeIdFromJSON(json['local_type_id']),
    };
}

export function ScopedTypeIdToJSON(value?: ScopedTypeId | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'schema_hash': value.schema_hash,
        'local_type_id': LocalTypeIdToJSON(value.local_type_id),
    };
}
