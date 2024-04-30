/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon (Anemone)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.1.3
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

/**
 * 
 * @export
 * @interface ResourceChange
 */
export interface ResourceChange {
    /**
     * The Bech32m-encoded human readable version of the resource address
     * @type {string}
     * @memberof ResourceChange
     */
    resource_address: string;
    /**
     * 
     * @type {EntityReference}
     * @memberof ResourceChange
     */
    component_entity: EntityReference;
    /**
     * 
     * @type {EntityReference}
     * @memberof ResourceChange
     */
    vault_entity: EntityReference;
    /**
     * The string-encoded decimal representing the XRD amount put or taken from the vault.
     * A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`.
     * @type {string}
     * @memberof ResourceChange
     */
    amount: string;
}

/**
 * Check if a given object implements the ResourceChange interface.
 */
export function instanceOfResourceChange(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "resource_address" in value;
    isInstance = isInstance && "component_entity" in value;
    isInstance = isInstance && "vault_entity" in value;
    isInstance = isInstance && "amount" in value;

    return isInstance;
}

export function ResourceChangeFromJSON(json: any): ResourceChange {
    return ResourceChangeFromJSONTyped(json, false);
}

export function ResourceChangeFromJSONTyped(json: any, ignoreDiscriminator: boolean): ResourceChange {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'resource_address': json['resource_address'],
        'component_entity': EntityReferenceFromJSON(json['component_entity']),
        'vault_entity': EntityReferenceFromJSON(json['vault_entity']),
        'amount': json['amount'],
    };
}

export function ResourceChangeToJSON(value?: ResourceChange | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'resource_address': value.resource_address,
        'component_entity': EntityReferenceToJSON(value.component_entity),
        'vault_entity': EntityReferenceToJSON(value.vault_entity),
        'amount': value.amount,
    };
}

