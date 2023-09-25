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
/**
 * 
 * @export
 * @interface ResourceRequirement
 */
export interface ResourceRequirement {
    /**
     * 
     * @type {string}
     * @memberof ResourceRequirement
     */
    type: ResourceRequirementTypeEnum;
    /**
     * The Bech32m-encoded human readable version of the resource address
     * @type {string}
     * @memberof ResourceRequirement
     */
    resource: string;
}


/**
 * @export
 */
export const ResourceRequirementTypeEnum = {
    Resource: 'Resource'
} as const;
export type ResourceRequirementTypeEnum = typeof ResourceRequirementTypeEnum[keyof typeof ResourceRequirementTypeEnum];


/**
 * Check if a given object implements the ResourceRequirement interface.
 */
export function instanceOfResourceRequirement(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "type" in value;
    isInstance = isInstance && "resource" in value;

    return isInstance;
}

export function ResourceRequirementFromJSON(json: any): ResourceRequirement {
    return ResourceRequirementFromJSONTyped(json, false);
}

export function ResourceRequirementFromJSONTyped(json: any, ignoreDiscriminator: boolean): ResourceRequirement {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'type': json['type'],
        'resource': json['resource'],
    };
}

export function ResourceRequirementToJSON(value?: ResourceRequirement | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'type': value.type,
        'resource': value.resource,
    };
}

