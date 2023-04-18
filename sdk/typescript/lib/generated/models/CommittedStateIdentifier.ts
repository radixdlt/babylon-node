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
/**
 * 
 * @export
 * @interface CommittedStateIdentifier
 */
export interface CommittedStateIdentifier {
    /**
     * The hex-encoded transaction accumulator hash. This hash captures the order of all transactions on ledger.
     * This hash is `ACC_{N+1} = Blake2b-256(CONCAT(ACC_N, LEDGER_HASH_{N}))`, starting with `ACC_0 = 000..000` the pre-genesis accumulator.
     * @type {string}
     * @memberof CommittedStateIdentifier
     */
    accumulator_hash: string;
    /**
     * An integer between `0` and `10^13`, representing the state version. The state version increments with each transaction, starting at `0` pre-genesis.
     * @type {number}
     * @memberof CommittedStateIdentifier
     */
    state_version: number;
}

/**
 * Check if a given object implements the CommittedStateIdentifier interface.
 */
export function instanceOfCommittedStateIdentifier(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "accumulator_hash" in value;
    isInstance = isInstance && "state_version" in value;

    return isInstance;
}

export function CommittedStateIdentifierFromJSON(json: any): CommittedStateIdentifier {
    return CommittedStateIdentifierFromJSONTyped(json, false);
}

export function CommittedStateIdentifierFromJSONTyped(json: any, ignoreDiscriminator: boolean): CommittedStateIdentifier {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'accumulator_hash': json['accumulator_hash'],
        'state_version': json['state_version'],
    };
}

export function CommittedStateIdentifierToJSON(value?: CommittedStateIdentifier | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'accumulator_hash': value.accumulator_hash,
        'state_version': value.state_version,
    };
}

