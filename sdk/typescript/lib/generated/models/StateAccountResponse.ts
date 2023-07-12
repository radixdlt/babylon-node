/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet V2
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.4.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { Substate } from './Substate';
import {
    SubstateFromJSON,
    SubstateFromJSONTyped,
    SubstateToJSON,
} from './Substate';
import type { VaultBalance } from './VaultBalance';
import {
    VaultBalanceFromJSON,
    VaultBalanceFromJSONTyped,
    VaultBalanceToJSON,
} from './VaultBalance';

/**
 * 
 * @export
 * @interface StateAccountResponse
 */
export interface StateAccountResponse {
    /**
     * 
     * @type {Substate}
     * @memberof StateAccountResponse
     */
    info: Substate;
    /**
     * 
     * @type {Substate}
     * @memberof StateAccountResponse
     */
    owner_role: Substate;
    /**
     * 
     * @type {Substate}
     * @memberof StateAccountResponse
     */
    state: Substate;
    /**
     * All of the account's vaults
     * @type {Array<VaultBalance>}
     * @memberof StateAccountResponse
     */
    vaults: Array<VaultBalance>;
}

/**
 * Check if a given object implements the StateAccountResponse interface.
 */
export function instanceOfStateAccountResponse(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "info" in value;
    isInstance = isInstance && "owner_role" in value;
    isInstance = isInstance && "state" in value;
    isInstance = isInstance && "vaults" in value;

    return isInstance;
}

export function StateAccountResponseFromJSON(json: any): StateAccountResponse {
    return StateAccountResponseFromJSONTyped(json, false);
}

export function StateAccountResponseFromJSONTyped(json: any, ignoreDiscriminator: boolean): StateAccountResponse {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'info': SubstateFromJSON(json['info']),
        'owner_role': SubstateFromJSON(json['owner_role']),
        'state': SubstateFromJSON(json['state']),
        'vaults': ((json['vaults'] as Array<any>).map(VaultBalanceFromJSON)),
    };
}

export function StateAccountResponseToJSON(value?: StateAccountResponse | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'info': SubstateToJSON(value.info),
        'owner_role': SubstateToJSON(value.owner_role),
        'state': SubstateToJSON(value.state),
        'vaults': ((value.vaults as Array<any>).map(VaultBalanceToJSON)),
    };
}
