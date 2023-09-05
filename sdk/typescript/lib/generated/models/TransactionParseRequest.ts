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
import type { TransactionFormatOptions } from './TransactionFormatOptions';
import {
    TransactionFormatOptionsFromJSON,
    TransactionFormatOptionsFromJSONTyped,
    TransactionFormatOptionsToJSON,
} from './TransactionFormatOptions';

/**
 * 
 * @export
 * @interface TransactionParseRequest
 */
export interface TransactionParseRequest {
    /**
     * The logical name of the network
     * @type {string}
     * @memberof TransactionParseRequest
     */
    network: string;
    /**
     * A hex-encoded payload of a full transaction or a partial transaction - either a notarized transaction,
     * a signed transaction intent an unsigned transaction intent, or a ledger payload.
     * @type {string}
     * @memberof TransactionParseRequest
     */
    payload_hex: string;
    /**
     * The type of transaction payload that should be assumed. If omitted, "Any" is used - where the payload is
     * attempted to be parsed as each of the following in turn: Notarized, Signed, Unsigned, Ledger.
     * @type {string}
     * @memberof TransactionParseRequest
     */
    parse_mode?: TransactionParseRequestParseModeEnum;
    /**
     * The type of validation that should be performed, if the payload correctly decompiles as a Notarized Transaction.
     * This is only relevant for Notarized payloads. If omitted, "Static" is used.
     * @type {string}
     * @memberof TransactionParseRequest
     */
    validation_mode?: TransactionParseRequestValidationModeEnum;
    /**
     * The amount of information to return in the response.
     * "Basic" includes the type, validity information, and any relevant identifiers.
     * "Full" also includes the fully parsed information.
     * If omitted, "Full" is used.
     * @type {string}
     * @memberof TransactionParseRequest
     */
    response_mode?: TransactionParseRequestResponseModeEnum;
    /**
     * 
     * @type {TransactionFormatOptions}
     * @memberof TransactionParseRequest
     */
    transaction_format_options?: TransactionFormatOptions;
}


/**
 * @export
 */
export const TransactionParseRequestParseModeEnum = {
    Any: 'Any',
    Notarized: 'Notarized',
    Signed: 'Signed',
    Unsigned: 'Unsigned',
    Ledger: 'Ledger'
} as const;
export type TransactionParseRequestParseModeEnum = typeof TransactionParseRequestParseModeEnum[keyof typeof TransactionParseRequestParseModeEnum];

/**
 * @export
 */
export const TransactionParseRequestValidationModeEnum = {
    None: 'None',
    Static: 'Static',
    Full: 'Full'
} as const;
export type TransactionParseRequestValidationModeEnum = typeof TransactionParseRequestValidationModeEnum[keyof typeof TransactionParseRequestValidationModeEnum];

/**
 * @export
 */
export const TransactionParseRequestResponseModeEnum = {
    Basic: 'Basic',
    Full: 'Full'
} as const;
export type TransactionParseRequestResponseModeEnum = typeof TransactionParseRequestResponseModeEnum[keyof typeof TransactionParseRequestResponseModeEnum];


/**
 * Check if a given object implements the TransactionParseRequest interface.
 */
export function instanceOfTransactionParseRequest(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "network" in value;
    isInstance = isInstance && "payload_hex" in value;

    return isInstance;
}

export function TransactionParseRequestFromJSON(json: any): TransactionParseRequest {
    return TransactionParseRequestFromJSONTyped(json, false);
}

export function TransactionParseRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): TransactionParseRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'network': json['network'],
        'payload_hex': json['payload_hex'],
        'parse_mode': !exists(json, 'parse_mode') ? undefined : json['parse_mode'],
        'validation_mode': !exists(json, 'validation_mode') ? undefined : json['validation_mode'],
        'response_mode': !exists(json, 'response_mode') ? undefined : json['response_mode'],
        'transaction_format_options': !exists(json, 'transaction_format_options') ? undefined : TransactionFormatOptionsFromJSON(json['transaction_format_options']),
    };
}

export function TransactionParseRequestToJSON(value?: TransactionParseRequest | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'network': value.network,
        'payload_hex': value.payload_hex,
        'parse_mode': value.parse_mode,
        'validation_mode': value.validation_mode,
        'response_mode': value.response_mode,
        'transaction_format_options': TransactionFormatOptionsToJSON(value.transaction_format_options),
    };
}

