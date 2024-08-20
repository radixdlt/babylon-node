/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.2
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
import type { InstantMs } from './InstantMs';
import {
    InstantMsFromJSON,
    InstantMsFromJSONTyped,
    InstantMsToJSON,
} from './InstantMs';

/**
 * 
 * @export
 * @interface ConsensusManagerFieldStateValue
 */
export interface ConsensusManagerFieldStateValue {
    /**
     * An integer between `0` and `10^10`, marking the current epoch
     * @type {number}
     * @memberof ConsensusManagerFieldStateValue
     */
    epoch: number;
    /**
     * An integer between `0` and `10^10`, marking the current round in an epoch
     * @type {number}
     * @memberof ConsensusManagerFieldStateValue
     */
    round: number;
    /**
     * 
     * @type {boolean}
     * @memberof ConsensusManagerFieldStateValue
     */
    is_started: boolean;
    /**
     * 
     * @type {InstantMs}
     * @memberof ConsensusManagerFieldStateValue
     */
    effective_epoch_start: InstantMs;
    /**
     * 
     * @type {InstantMs}
     * @memberof ConsensusManagerFieldStateValue
     */
    actual_epoch_start: InstantMs;
    /**
     * 
     * @type {ActiveValidatorIndex}
     * @memberof ConsensusManagerFieldStateValue
     */
    current_leader?: ActiveValidatorIndex;
}

/**
 * Check if a given object implements the ConsensusManagerFieldStateValue interface.
 */
export function instanceOfConsensusManagerFieldStateValue(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "epoch" in value;
    isInstance = isInstance && "round" in value;
    isInstance = isInstance && "is_started" in value;
    isInstance = isInstance && "effective_epoch_start" in value;
    isInstance = isInstance && "actual_epoch_start" in value;

    return isInstance;
}

export function ConsensusManagerFieldStateValueFromJSON(json: any): ConsensusManagerFieldStateValue {
    return ConsensusManagerFieldStateValueFromJSONTyped(json, false);
}

export function ConsensusManagerFieldStateValueFromJSONTyped(json: any, ignoreDiscriminator: boolean): ConsensusManagerFieldStateValue {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'epoch': json['epoch'],
        'round': json['round'],
        'is_started': json['is_started'],
        'effective_epoch_start': InstantMsFromJSON(json['effective_epoch_start']),
        'actual_epoch_start': InstantMsFromJSON(json['actual_epoch_start']),
        'current_leader': !exists(json, 'current_leader') ? undefined : ActiveValidatorIndexFromJSON(json['current_leader']),
    };
}

export function ConsensusManagerFieldStateValueToJSON(value?: ConsensusManagerFieldStateValue | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'epoch': value.epoch,
        'round': value.round,
        'is_started': value.is_started,
        'effective_epoch_start': InstantMsToJSON(value.effective_epoch_start),
        'actual_epoch_start': InstantMsToJSON(value.actual_epoch_start),
        'current_leader': ActiveValidatorIndexToJSON(value.current_leader),
    };
}

