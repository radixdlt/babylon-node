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
import type { DetailedBlueprintInfo } from './DetailedBlueprintInfo';
import {
    DetailedBlueprintInfoFromJSON,
    DetailedBlueprintInfoFromJSONTyped,
    DetailedBlueprintInfoToJSON,
} from './DetailedBlueprintInfo';
import type { LedgerStateSummary } from './LedgerStateSummary';
import {
    LedgerStateSummaryFromJSON,
    LedgerStateSummaryFromJSONTyped,
    LedgerStateSummaryToJSON,
} from './LedgerStateSummary';

/**
 * 
 * @export
 * @interface BrowseBlueprintInfoResponse
 */
export interface BrowseBlueprintInfoResponse {
    /**
     * 
     * @type {LedgerStateSummary}
     * @memberof BrowseBlueprintInfoResponse
     */
    at_ledger_state: LedgerStateSummary;
    /**
     * 
     * @type {DetailedBlueprintInfo}
     * @memberof BrowseBlueprintInfoResponse
     */
    info: DetailedBlueprintInfo;
}

/**
 * Check if a given object implements the BrowseBlueprintInfoResponse interface.
 */
export function instanceOfBrowseBlueprintInfoResponse(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "at_ledger_state" in value;
    isInstance = isInstance && "info" in value;

    return isInstance;
}

export function BrowseBlueprintInfoResponseFromJSON(json: any): BrowseBlueprintInfoResponse {
    return BrowseBlueprintInfoResponseFromJSONTyped(json, false);
}

export function BrowseBlueprintInfoResponseFromJSONTyped(json: any, ignoreDiscriminator: boolean): BrowseBlueprintInfoResponse {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'at_ledger_state': LedgerStateSummaryFromJSON(json['at_ledger_state']),
        'info': DetailedBlueprintInfoFromJSON(json['info']),
    };
}

export function BrowseBlueprintInfoResponseToJSON(value?: BrowseBlueprintInfoResponse | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'at_ledger_state': LedgerStateSummaryToJSON(value.at_ledger_state),
        'info': DetailedBlueprintInfoToJSON(value.info),
    };
}
