/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
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
 * @interface ParsedTransactionIntentIdentifiers
 */
export interface ParsedTransactionIntentIdentifiers {
    /**
     * The hex-encoded intent hash for a user transaction, also known as the transaction id.
     * This hash identifies the core content "intent" of the transaction. Each intent can only be committed once.
     * This hash gets signed by any signatories on the transaction, to create the signed intent.
     * @type {string}
     * @memberof ParsedTransactionIntentIdentifiers
     */
    intent_hash: string;
    /**
     * The Bech32m-encoded human readable `IntentHash`.
     * @type {string}
     * @memberof ParsedTransactionIntentIdentifiers
     */
    intent_hash_bech32m: string;
}

/**
 * Check if a given object implements the ParsedTransactionIntentIdentifiers interface.
 */
export function instanceOfParsedTransactionIntentIdentifiers(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "intent_hash" in value;
    isInstance = isInstance && "intent_hash_bech32m" in value;

    return isInstance;
}

export function ParsedTransactionIntentIdentifiersFromJSON(json: any): ParsedTransactionIntentIdentifiers {
    return ParsedTransactionIntentIdentifiersFromJSONTyped(json, false);
}

export function ParsedTransactionIntentIdentifiersFromJSONTyped(json: any, ignoreDiscriminator: boolean): ParsedTransactionIntentIdentifiers {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'intent_hash': json['intent_hash'],
        'intent_hash_bech32m': json['intent_hash_bech32m'],
    };
}

export function ParsedTransactionIntentIdentifiersToJSON(value?: ParsedTransactionIntentIdentifiers | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'intent_hash': value.intent_hash,
        'intent_hash_bech32m': value.intent_hash_bech32m,
    };
}

