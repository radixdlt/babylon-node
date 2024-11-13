/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { IntentCoreV2 } from './IntentCoreV2';
import {
    IntentCoreV2FromJSON,
    IntentCoreV2FromJSONTyped,
    IntentCoreV2ToJSON,
} from './IntentCoreV2';
import type { SubintentV2 } from './SubintentV2';
import {
    SubintentV2FromJSON,
    SubintentV2FromJSONTyped,
    SubintentV2ToJSON,
} from './SubintentV2';
import type { TransactionHeaderV2 } from './TransactionHeaderV2';
import {
    TransactionHeaderV2FromJSON,
    TransactionHeaderV2FromJSONTyped,
    TransactionHeaderV2ToJSON,
} from './TransactionHeaderV2';

/**
 * 
 * @export
 * @interface TransactionIntentV2
 */
export interface TransactionIntentV2 {
    /**
     * The hex-encoded transaction intent hash for a user transaction, also known as the transaction id.
     * This hash identifies the core "intent" of the transaction. Each transaction intent can only be committed once.
     * This hash gets signed by any signatories on the transaction, to create the signed intent.
     * @type {string}
     * @memberof TransactionIntentV2
     */
    hash: string;
    /**
     * The Bech32m-encoded human readable `TransactionIntentHash`.
     * @type {string}
     * @memberof TransactionIntentV2
     */
    hash_bech32m: string;
    /**
     * 
     * @type {TransactionHeaderV2}
     * @memberof TransactionIntentV2
     */
    transaction_header: TransactionHeaderV2;
    /**
     * 
     * @type {IntentCoreV2}
     * @memberof TransactionIntentV2
     */
    root_intent_core: IntentCoreV2;
    /**
     * 
     * @type {Array<SubintentV2>}
     * @memberof TransactionIntentV2
     */
    non_root_subintents: Array<SubintentV2>;
}

/**
 * Check if a given object implements the TransactionIntentV2 interface.
 */
export function instanceOfTransactionIntentV2(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "hash" in value;
    isInstance = isInstance && "hash_bech32m" in value;
    isInstance = isInstance && "transaction_header" in value;
    isInstance = isInstance && "root_intent_core" in value;
    isInstance = isInstance && "non_root_subintents" in value;

    return isInstance;
}

export function TransactionIntentV2FromJSON(json: any): TransactionIntentV2 {
    return TransactionIntentV2FromJSONTyped(json, false);
}

export function TransactionIntentV2FromJSONTyped(json: any, ignoreDiscriminator: boolean): TransactionIntentV2 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'hash': json['hash'],
        'hash_bech32m': json['hash_bech32m'],
        'transaction_header': TransactionHeaderV2FromJSON(json['transaction_header']),
        'root_intent_core': IntentCoreV2FromJSON(json['root_intent_core']),
        'non_root_subintents': ((json['non_root_subintents'] as Array<any>).map(SubintentV2FromJSON)),
    };
}

export function TransactionIntentV2ToJSON(value?: TransactionIntentV2 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'hash': value.hash,
        'hash_bech32m': value.hash_bech32m,
        'transaction_header': TransactionHeaderV2ToJSON(value.transaction_header),
        'root_intent_core': IntentCoreV2ToJSON(value.root_intent_core),
        'non_root_subintents': ((value.non_root_subintents as Array<any>).map(SubintentV2ToJSON)),
    };
}
