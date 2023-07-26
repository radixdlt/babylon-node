/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet v3
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the second release candidate of the Radix Babylon network (\"RCnet v3\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { Signature } from './Signature';
import {
    SignatureFromJSON,
    SignatureFromJSONTyped,
    SignatureToJSON,
} from './Signature';
import type { SignedTransactionIntent } from './SignedTransactionIntent';
import {
    SignedTransactionIntentFromJSON,
    SignedTransactionIntentFromJSONTyped,
    SignedTransactionIntentToJSON,
} from './SignedTransactionIntent';

/**
 * 
 * @export
 * @interface NotarizedTransaction
 */
export interface NotarizedTransaction {
    /**
     * The hex-encoded notarized transaction hash for a user transaction.
     * This hash identifies the full submittable notarized transaction - ie the signed intent, plus the notary signature.
     * @type {string}
     * @memberof NotarizedTransaction
     */
    hash: string;
    /**
     * The hex-encoded full notarized transaction payload. Returning this can be disabled in TransactionFormatOptions on your request (default true).
     * @type {string}
     * @memberof NotarizedTransaction
     */
    payload_hex?: string;
    /**
     * 
     * @type {SignedTransactionIntent}
     * @memberof NotarizedTransaction
     */
    signed_intent: SignedTransactionIntent;
    /**
     * 
     * @type {Signature}
     * @memberof NotarizedTransaction
     */
    notary_signature: Signature;
}

/**
 * Check if a given object implements the NotarizedTransaction interface.
 */
export function instanceOfNotarizedTransaction(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "hash" in value;
    isInstance = isInstance && "signed_intent" in value;
    isInstance = isInstance && "notary_signature" in value;

    return isInstance;
}

export function NotarizedTransactionFromJSON(json: any): NotarizedTransaction {
    return NotarizedTransactionFromJSONTyped(json, false);
}

export function NotarizedTransactionFromJSONTyped(json: any, ignoreDiscriminator: boolean): NotarizedTransaction {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'hash': json['hash'],
        'payload_hex': !exists(json, 'payload_hex') ? undefined : json['payload_hex'],
        'signed_intent': SignedTransactionIntentFromJSON(json['signed_intent']),
        'notary_signature': SignatureFromJSON(json['notary_signature']),
    };
}

export function NotarizedTransactionToJSON(value?: NotarizedTransaction | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'hash': value.hash,
        'payload_hex': value.payload_hex,
        'signed_intent': SignedTransactionIntentToJSON(value.signed_intent),
        'notary_signature': SignatureToJSON(value.notary_signature),
    };
}

