/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.3.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { IntentSignatures } from './IntentSignatures';
import {
    IntentSignaturesFromJSON,
    IntentSignaturesFromJSONTyped,
    IntentSignaturesToJSON,
} from './IntentSignatures';
import type { TransactionIntentV2 } from './TransactionIntentV2';
import {
    TransactionIntentV2FromJSON,
    TransactionIntentV2FromJSONTyped,
    TransactionIntentV2ToJSON,
} from './TransactionIntentV2';

/**
 * 
 * @export
 * @interface SignedTransactionIntentV2
 */
export interface SignedTransactionIntentV2 {
    /**
     * The hex-encoded signed intent hash for a user transaction.
     * This hash identifies the transaction intent, plus additional signatures.
     * This hash is signed by the notary, to create the submittable `NotarizedTransaction`.
     * @type {string}
     * @memberof SignedTransactionIntentV2
     */
    hash: string;
    /**
     * The Bech32m-encoded human readable `SignedTransactionIntentHash`.
     * @type {string}
     * @memberof SignedTransactionIntentV2
     */
    hash_bech32m: string;
    /**
     * 
     * @type {TransactionIntentV2}
     * @memberof SignedTransactionIntentV2
     */
    transaction_intent: TransactionIntentV2;
    /**
     * 
     * @type {IntentSignatures}
     * @memberof SignedTransactionIntentV2
     */
    transaction_intent_signatures: IntentSignatures;
    /**
     * This gives the signatures for each subintent in `non_root_subintents` in `TransactionIntentV2`.
     * For committed transactions, these arrays are of equal length and correspond one-to-one in order.
     * @type {Array<IntentSignatures>}
     * @memberof SignedTransactionIntentV2
     */
    non_root_subintent_signatures: Array<IntentSignatures>;
}

/**
 * Check if a given object implements the SignedTransactionIntentV2 interface.
 */
export function instanceOfSignedTransactionIntentV2(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "hash" in value;
    isInstance = isInstance && "hash_bech32m" in value;
    isInstance = isInstance && "transaction_intent" in value;
    isInstance = isInstance && "transaction_intent_signatures" in value;
    isInstance = isInstance && "non_root_subintent_signatures" in value;

    return isInstance;
}

export function SignedTransactionIntentV2FromJSON(json: any): SignedTransactionIntentV2 {
    return SignedTransactionIntentV2FromJSONTyped(json, false);
}

export function SignedTransactionIntentV2FromJSONTyped(json: any, ignoreDiscriminator: boolean): SignedTransactionIntentV2 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'hash': json['hash'],
        'hash_bech32m': json['hash_bech32m'],
        'transaction_intent': TransactionIntentV2FromJSON(json['transaction_intent']),
        'transaction_intent_signatures': IntentSignaturesFromJSON(json['transaction_intent_signatures']),
        'non_root_subintent_signatures': ((json['non_root_subintent_signatures'] as Array<any>).map(IntentSignaturesFromJSON)),
    };
}

export function SignedTransactionIntentV2ToJSON(value?: SignedTransactionIntentV2 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'hash': value.hash,
        'hash_bech32m': value.hash_bech32m,
        'transaction_intent': TransactionIntentV2ToJSON(value.transaction_intent),
        'transaction_intent_signatures': IntentSignaturesToJSON(value.transaction_intent_signatures),
        'non_root_subintent_signatures': ((value.non_root_subintent_signatures as Array<any>).map(IntentSignaturesToJSON)),
    };
}

