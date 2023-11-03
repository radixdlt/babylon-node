/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.4
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
 * @interface BrowseEntityIteratorRequest
 */
export interface BrowseEntityIteratorRequest {
    /**
     * The logical name of the network
     * @type {string}
     * @memberof BrowseEntityIteratorRequest
     */
    network: string;
    /**
     * A maximum number of items to be included in the paged listing response.
     * By default, each paged listing endpoint imposes its own limit on the number of returned
     * items (which may even be driven dynamically by system load, etc). This client-provided
     * maximum page size simply adds a further constraint (i.e. can only lower down the number
     * of returned items).
     * @type {number}
     * @memberof BrowseEntityIteratorRequest
     */
    max_page_size?: number;
    /**
     * An opaque string conveying the information on where the next page of results starts.
     * It is returned in every paged listing response (except for the last page), and it can be
     * passed in every paged listing request (in order to begin listing from where the previous
     * response ended).
     * @type {string}
     * @memberof BrowseEntityIteratorRequest
     */
    continuation_token?: string;
}

/**
 * Check if a given object implements the BrowseEntityIteratorRequest interface.
 */
export function instanceOfBrowseEntityIteratorRequest(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "network" in value;

    return isInstance;
}

export function BrowseEntityIteratorRequestFromJSON(json: any): BrowseEntityIteratorRequest {
    return BrowseEntityIteratorRequestFromJSONTyped(json, false);
}

export function BrowseEntityIteratorRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): BrowseEntityIteratorRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'network': json['network'],
        'max_page_size': !exists(json, 'max_page_size') ? undefined : json['max_page_size'],
        'continuation_token': !exists(json, 'continuation_token') ? undefined : json['continuation_token'],
    };
}

export function BrowseEntityIteratorRequestToJSON(value?: BrowseEntityIteratorRequest | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'network': value.network,
        'max_page_size': value.max_page_size,
        'continuation_token': value.continuation_token,
    };
}

