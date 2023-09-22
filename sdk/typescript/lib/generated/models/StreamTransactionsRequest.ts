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
import type { SborFormatOptions } from './SborFormatOptions';
import {
    SborFormatOptionsFromJSON,
    SborFormatOptionsFromJSONTyped,
    SborFormatOptionsToJSON,
} from './SborFormatOptions';
import type { SubstateFormatOptions } from './SubstateFormatOptions';
import {
    SubstateFormatOptionsFromJSON,
    SubstateFormatOptionsFromJSONTyped,
    SubstateFormatOptionsToJSON,
} from './SubstateFormatOptions';
import type { TransactionFormatOptions } from './TransactionFormatOptions';
import {
    TransactionFormatOptionsFromJSON,
    TransactionFormatOptionsFromJSONTyped,
    TransactionFormatOptionsToJSON,
} from './TransactionFormatOptions';

/**
 * A request to retrieve a sublist of committed transactions from the ledger.
 * @export
 * @interface StreamTransactionsRequest
 */
export interface StreamTransactionsRequest {
    /**
     * The logical name of the network
     * @type {string}
     * @memberof StreamTransactionsRequest
     */
    network: string;
    /**
     * 
     * @type {number}
     * @memberof StreamTransactionsRequest
     */
    from_state_version: number;
    /**
     * The maximum number of transactions that will be returned.
     * @type {number}
     * @memberof StreamTransactionsRequest
     */
    limit: number;
    /**
     * 
     * @type {SborFormatOptions}
     * @memberof StreamTransactionsRequest
     */
    sbor_format_options?: SborFormatOptions;
    /**
     * 
     * @type {TransactionFormatOptions}
     * @memberof StreamTransactionsRequest
     */
    transaction_format_options?: TransactionFormatOptions;
    /**
     * 
     * @type {SubstateFormatOptions}
     * @memberof StreamTransactionsRequest
     */
    substate_format_options?: SubstateFormatOptions;
    /**
     * Whether to include LedgerProofs (default false)
     * @type {boolean}
     * @memberof StreamTransactionsRequest
     */
    include_proofs?: boolean;
}

/**
 * Check if a given object implements the StreamTransactionsRequest interface.
 */
export function instanceOfStreamTransactionsRequest(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "network" in value;
    isInstance = isInstance && "from_state_version" in value;
    isInstance = isInstance && "limit" in value;

    return isInstance;
}

export function StreamTransactionsRequestFromJSON(json: any): StreamTransactionsRequest {
    return StreamTransactionsRequestFromJSONTyped(json, false);
}

export function StreamTransactionsRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): StreamTransactionsRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'network': json['network'],
        'from_state_version': json['from_state_version'],
        'limit': json['limit'],
        'sbor_format_options': !exists(json, 'sbor_format_options') ? undefined : SborFormatOptionsFromJSON(json['sbor_format_options']),
        'transaction_format_options': !exists(json, 'transaction_format_options') ? undefined : TransactionFormatOptionsFromJSON(json['transaction_format_options']),
        'substate_format_options': !exists(json, 'substate_format_options') ? undefined : SubstateFormatOptionsFromJSON(json['substate_format_options']),
        'include_proofs': !exists(json, 'include_proofs') ? undefined : json['include_proofs'],
    };
}

export function StreamTransactionsRequestToJSON(value?: StreamTransactionsRequest | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'network': value.network,
        'from_state_version': value.from_state_version,
        'limit': value.limit,
        'sbor_format_options': SborFormatOptionsToJSON(value.sbor_format_options),
        'transaction_format_options': TransactionFormatOptionsToJSON(value.transaction_format_options),
        'substate_format_options': SubstateFormatOptionsToJSON(value.substate_format_options),
        'include_proofs': value.include_proofs,
    };
}

