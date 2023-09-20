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
import type { ActiveValidatorIndex } from './ActiveValidatorIndex';
import {
    ActiveValidatorIndexFromJSON,
    ActiveValidatorIndexFromJSONTyped,
    ActiveValidatorIndexToJSON,
} from './ActiveValidatorIndex';

/**
 * 
 * @export
 * @interface LeaderProposalHistory
 */
export interface LeaderProposalHistory {
    /**
     * The validators which were leaders of the "gap" rounds (i.e. since the previous `RoundUpdateValidatorTransaction` - which means that this list will contain exactly `current.round - previous.round - 1` elements). The validators on this list should be penalized during emissions at the end of the epoch.
     * @type {Array<ActiveValidatorIndex>}
     * @memberof LeaderProposalHistory
     */
    gap_round_leaders: Array<ActiveValidatorIndex>;
    /**
     * 
     * @type {ActiveValidatorIndex}
     * @memberof LeaderProposalHistory
     */
    current_leader: ActiveValidatorIndex;
    /**
     * Whether the concluded round was conducted in a "fallback" mode (i.e. indicating a fault of the current leader). When `true`, the `current_leader` should be penalized during emissions in the same way as `gap_round_leaders`. When `false`, the `current_leader` is considered to have made this round's proposal successfully.
     * @type {boolean}
     * @memberof LeaderProposalHistory
     */
    is_fallback: boolean;
}

/**
 * Check if a given object implements the LeaderProposalHistory interface.
 */
export function instanceOfLeaderProposalHistory(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "gap_round_leaders" in value;
    isInstance = isInstance && "current_leader" in value;
    isInstance = isInstance && "is_fallback" in value;

    return isInstance;
}

export function LeaderProposalHistoryFromJSON(json: any): LeaderProposalHistory {
    return LeaderProposalHistoryFromJSONTyped(json, false);
}

export function LeaderProposalHistoryFromJSONTyped(json: any, ignoreDiscriminator: boolean): LeaderProposalHistory {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'gap_round_leaders': ((json['gap_round_leaders'] as Array<any>).map(ActiveValidatorIndexFromJSON)),
        'current_leader': ActiveValidatorIndexFromJSON(json['current_leader']),
        'is_fallback': json['is_fallback'],
    };
}

export function LeaderProposalHistoryToJSON(value?: LeaderProposalHistory | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'gap_round_leaders': ((value.gap_round_leaders as Array<any>).map(ActiveValidatorIndexToJSON)),
        'current_leader': ActiveValidatorIndexToJSON(value.current_leader),
        'is_fallback': value.is_fallback,
    };
}

