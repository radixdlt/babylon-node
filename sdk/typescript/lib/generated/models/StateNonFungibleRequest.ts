/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.3.0
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
 * @interface StateNonFungibleRequest
 */
export interface StateNonFungibleRequest {
    /**
     * The logical name of the network
     * @type {string}
     * @memberof StateNonFungibleRequest
     */
    network: string;
    /**
     * The Bech32m-encoded human readable version of the resource's global address
     * @type {string}
     * @memberof StateNonFungibleRequest
     */
    resource_address: string;
    /**
     * The simple string representation of the non-fungible id.
     * * For string ids, this is `<the-string-id>`
     * * For integer ids, this is `#the-integer-id#`
     * * For bytes ids, this is `[the-lower-case-hex-representation]`
     * * For RUID ids, this is `{...-...-...-...}` where `...` are each 16 hex characters.
     * A given non-fungible resource has a fixed `NonFungibleIdType`, so this representation uniquely identifies this non-fungible
     * under the given resource address.
     * @type {string}
     * @memberof StateNonFungibleRequest
     */
    non_fungible_id: string;
}

/**
 * Check if a given object implements the StateNonFungibleRequest interface.
 */
export function instanceOfStateNonFungibleRequest(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "network" in value;
    isInstance = isInstance && "resource_address" in value;
    isInstance = isInstance && "non_fungible_id" in value;

    return isInstance;
}

export function StateNonFungibleRequestFromJSON(json: any): StateNonFungibleRequest {
    return StateNonFungibleRequestFromJSONTyped(json, false);
}

export function StateNonFungibleRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): StateNonFungibleRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'network': json['network'],
        'resource_address': json['resource_address'],
        'non_fungible_id': json['non_fungible_id'],
    };
}

export function StateNonFungibleRequestToJSON(value?: StateNonFungibleRequest | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'network': value.network,
        'resource_address': value.resource_address,
        'non_fungible_id': value.non_fungible_id,
    };
}

