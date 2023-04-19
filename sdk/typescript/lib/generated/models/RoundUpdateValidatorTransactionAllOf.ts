/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet V1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.3.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { Instant } from './Instant';
import {
    InstantFromJSON,
    InstantFromJSONTyped,
    InstantToJSON,
} from './Instant';

/**
 * 
 * @export
 * @interface RoundUpdateValidatorTransactionAllOf
 */
export interface RoundUpdateValidatorTransactionAllOf {
    /**
     * 
     * @type {Instant}
     * @memberof RoundUpdateValidatorTransactionAllOf
     */
    proposer_timestamp: Instant;
    /**
     * An integer between `0` and `10^10`, marking the consensus epoch.
     * @type {number}
     * @memberof RoundUpdateValidatorTransactionAllOf
     */
    consensus_epoch: number;
    /**
     * An integer between `0` and `10^10`, marking the consensus round in the epoch
     * @type {number}
     * @memberof RoundUpdateValidatorTransactionAllOf
     */
    round_in_epoch: number;
    /**
     * 
     * @type {string}
     * @memberof RoundUpdateValidatorTransactionAllOf
     */
    type?: RoundUpdateValidatorTransactionAllOfTypeEnum;
}


/**
 * @export
 */
export const RoundUpdateValidatorTransactionAllOfTypeEnum = {
    RoundUpdate: 'RoundUpdate'
} as const;
export type RoundUpdateValidatorTransactionAllOfTypeEnum = typeof RoundUpdateValidatorTransactionAllOfTypeEnum[keyof typeof RoundUpdateValidatorTransactionAllOfTypeEnum];


/**
 * Check if a given object implements the RoundUpdateValidatorTransactionAllOf interface.
 */
export function instanceOfRoundUpdateValidatorTransactionAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "proposer_timestamp" in value;
    isInstance = isInstance && "consensus_epoch" in value;
    isInstance = isInstance && "round_in_epoch" in value;

    return isInstance;
}

export function RoundUpdateValidatorTransactionAllOfFromJSON(json: any): RoundUpdateValidatorTransactionAllOf {
    return RoundUpdateValidatorTransactionAllOfFromJSONTyped(json, false);
}

export function RoundUpdateValidatorTransactionAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): RoundUpdateValidatorTransactionAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'proposer_timestamp': InstantFromJSON(json['proposer_timestamp']),
        'consensus_epoch': json['consensus_epoch'],
        'round_in_epoch': json['round_in_epoch'],
        'type': !exists(json, 'type') ? undefined : json['type'],
    };
}

export function RoundUpdateValidatorTransactionAllOfToJSON(value?: RoundUpdateValidatorTransactionAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'proposer_timestamp': InstantToJSON(value.proposer_timestamp),
        'consensus_epoch': value.consensus_epoch,
        'round_in_epoch': value.round_in_epoch,
        'type': value.type,
    };
}
