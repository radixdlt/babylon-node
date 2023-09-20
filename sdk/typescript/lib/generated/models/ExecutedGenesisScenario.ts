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
import type { ExecutedScenarioTransaction } from './ExecutedScenarioTransaction';
import {
    ExecutedScenarioTransactionFromJSON,
    ExecutedScenarioTransactionFromJSONTyped,
    ExecutedScenarioTransactionToJSON,
} from './ExecutedScenarioTransaction';

/**
 * 
 * @export
 * @interface ExecutedGenesisScenario
 */
export interface ExecutedGenesisScenario {
    /**
     * An index of the Scenario on the list of all Scenarios that were executed.
     * Note: the stored sequence numbers do not necessarily have to be consecutive (e.g. in a
     * case where some configured Scenario failed to execute or failed to write results to the
     * database).
     * @type {number}
     * @memberof ExecutedGenesisScenario
     */
    sequence_number: number;
    /**
     * 
     * @type {string}
     * @memberof ExecutedGenesisScenario
     */
    logical_name: string;
    /**
     * Transactions successfully committed by the Scenario.
     * @type {Array<ExecutedScenarioTransaction>}
     * @memberof ExecutedGenesisScenario
     */
    committed_transactions: Array<ExecutedScenarioTransaction>;
    /**
     * Well-named addresses touched/created by the Scenario, keyed by their name.
     * @type {{ [key: string]: string; }}
     * @memberof ExecutedGenesisScenario
     */
    addresses: { [key: string]: string; };
}

/**
 * Check if a given object implements the ExecutedGenesisScenario interface.
 */
export function instanceOfExecutedGenesisScenario(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "sequence_number" in value;
    isInstance = isInstance && "logical_name" in value;
    isInstance = isInstance && "committed_transactions" in value;
    isInstance = isInstance && "addresses" in value;

    return isInstance;
}

export function ExecutedGenesisScenarioFromJSON(json: any): ExecutedGenesisScenario {
    return ExecutedGenesisScenarioFromJSONTyped(json, false);
}

export function ExecutedGenesisScenarioFromJSONTyped(json: any, ignoreDiscriminator: boolean): ExecutedGenesisScenario {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'sequence_number': json['sequence_number'],
        'logical_name': json['logical_name'],
        'committed_transactions': ((json['committed_transactions'] as Array<any>).map(ExecutedScenarioTransactionFromJSON)),
        'addresses': json['addresses'],
    };
}

export function ExecutedGenesisScenarioToJSON(value?: ExecutedGenesisScenario | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'sequence_number': value.sequence_number,
        'logical_name': value.logical_name,
        'committed_transactions': ((value.committed_transactions as Array<any>).map(ExecutedScenarioTransactionToJSON)),
        'addresses': value.addresses,
    };
}

