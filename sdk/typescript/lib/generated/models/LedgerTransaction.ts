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

import {
    GenesisLedgerTransaction,
    instanceOfGenesisLedgerTransaction,
    GenesisLedgerTransactionFromJSON,
    GenesisLedgerTransactionFromJSONTyped,
    GenesisLedgerTransactionToJSON,
} from './GenesisLedgerTransaction';
import {
    RoundUpdateLedgerTransaction,
    instanceOfRoundUpdateLedgerTransaction,
    RoundUpdateLedgerTransactionFromJSON,
    RoundUpdateLedgerTransactionFromJSONTyped,
    RoundUpdateLedgerTransactionToJSON,
} from './RoundUpdateLedgerTransaction';
import {
    UserLedgerTransaction,
    instanceOfUserLedgerTransaction,
    UserLedgerTransactionFromJSON,
    UserLedgerTransactionFromJSONTyped,
    UserLedgerTransactionToJSON,
} from './UserLedgerTransaction';

/**
 * @type LedgerTransaction
 * 
 * @export
 */
export type LedgerTransaction = { type: 'Genesis' } & GenesisLedgerTransaction | { type: 'RoundUpdate' } & RoundUpdateLedgerTransaction | { type: 'User' } & UserLedgerTransaction;

export function LedgerTransactionFromJSON(json: any): LedgerTransaction {
    return LedgerTransactionFromJSONTyped(json, false);
}

export function LedgerTransactionFromJSONTyped(json: any, ignoreDiscriminator: boolean): LedgerTransaction {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['type']) {
        case 'Genesis':
            return {...GenesisLedgerTransactionFromJSONTyped(json, true), type: 'Genesis'};
        case 'RoundUpdate':
            return {...RoundUpdateLedgerTransactionFromJSONTyped(json, true), type: 'RoundUpdate'};
        case 'User':
            return {...UserLedgerTransactionFromJSONTyped(json, true), type: 'User'};
        default:
            throw new Error(`No variant of LedgerTransaction exists with 'type=${json['type']}'`);
    }
}

export function LedgerTransactionToJSON(value?: LedgerTransaction | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    switch (value['type']) {
        case 'Genesis':
            return GenesisLedgerTransactionToJSON(value);
        case 'RoundUpdate':
            return RoundUpdateLedgerTransactionToJSON(value);
        case 'User':
            return UserLedgerTransactionToJSON(value);
        default:
            throw new Error(`No variant of LedgerTransaction exists with 'type=${value['type']}'`);
    }

}

