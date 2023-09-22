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
import type { LedgerHeaderSummary } from './LedgerHeaderSummary';
import {
    LedgerHeaderSummaryFromJSON,
    LedgerHeaderSummaryFromJSONTyped,
    LedgerHeaderSummaryToJSON,
} from './LedgerHeaderSummary';
import type { LtsFungibleResourceBalance } from './LtsFungibleResourceBalance';
import {
    LtsFungibleResourceBalanceFromJSON,
    LtsFungibleResourceBalanceFromJSONTyped,
    LtsFungibleResourceBalanceToJSON,
} from './LtsFungibleResourceBalance';

/**
 * 
 * @export
 * @interface LtsStateAccountFungibleResourceBalanceResponse
 */
export interface LtsStateAccountFungibleResourceBalanceResponse {
    /**
     * 
     * @type {number}
     * @memberof LtsStateAccountFungibleResourceBalanceResponse
     */
    state_version: number;
    /**
     * 
     * @type {LedgerHeaderSummary}
     * @memberof LtsStateAccountFungibleResourceBalanceResponse
     */
    ledger_header_summary: LedgerHeaderSummary;
    /**
     * The Bech32m-encoded human readable version of the account's address
     * @type {string}
     * @memberof LtsStateAccountFungibleResourceBalanceResponse
     */
    account_address: string;
    /**
     * 
     * @type {LtsFungibleResourceBalance}
     * @memberof LtsStateAccountFungibleResourceBalanceResponse
     */
    fungible_resource_balance: LtsFungibleResourceBalance;
}

/**
 * Check if a given object implements the LtsStateAccountFungibleResourceBalanceResponse interface.
 */
export function instanceOfLtsStateAccountFungibleResourceBalanceResponse(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "state_version" in value;
    isInstance = isInstance && "ledger_header_summary" in value;
    isInstance = isInstance && "account_address" in value;
    isInstance = isInstance && "fungible_resource_balance" in value;

    return isInstance;
}

export function LtsStateAccountFungibleResourceBalanceResponseFromJSON(json: any): LtsStateAccountFungibleResourceBalanceResponse {
    return LtsStateAccountFungibleResourceBalanceResponseFromJSONTyped(json, false);
}

export function LtsStateAccountFungibleResourceBalanceResponseFromJSONTyped(json: any, ignoreDiscriminator: boolean): LtsStateAccountFungibleResourceBalanceResponse {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'state_version': json['state_version'],
        'ledger_header_summary': LedgerHeaderSummaryFromJSON(json['ledger_header_summary']),
        'account_address': json['account_address'],
        'fungible_resource_balance': LtsFungibleResourceBalanceFromJSON(json['fungible_resource_balance']),
    };
}

export function LtsStateAccountFungibleResourceBalanceResponseToJSON(value?: LtsStateAccountFungibleResourceBalanceResponse | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'state_version': value.state_version,
        'ledger_header_summary': LedgerHeaderSummaryToJSON(value.ledger_header_summary),
        'account_address': value.account_address,
        'fungible_resource_balance': LtsFungibleResourceBalanceToJSON(value.fungible_resource_balance),
    };
}

