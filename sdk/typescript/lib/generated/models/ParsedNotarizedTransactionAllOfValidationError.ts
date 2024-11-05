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
/**
 * If the transaction is known to not be valid, this gives a reason.
 * Different levels of validation are performed, dependent on the validation mode.
 * Note that, even if validation mode is Static or Full, the transaction may
 * still be rejected or fail due to issues at runtime (e.g. if the loan cannot be repaid).
 * @export
 * @interface ParsedNotarizedTransactionAllOfValidationError
 */
export interface ParsedNotarizedTransactionAllOfValidationError {
    /**
     * The error message.
     * @type {string}
     * @memberof ParsedNotarizedTransactionAllOfValidationError
     */
    reason: string;
    /**
     * Whether the error is known to be permanent, or not.
     * This relates to whether the transaction would be rejected permanently or temporarily if submitted.
     * @type {boolean}
     * @memberof ParsedNotarizedTransactionAllOfValidationError
     */
    is_permanent: boolean;
}

/**
 * Check if a given object implements the ParsedNotarizedTransactionAllOfValidationError interface.
 */
export function instanceOfParsedNotarizedTransactionAllOfValidationError(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "reason" in value;
    isInstance = isInstance && "is_permanent" in value;

    return isInstance;
}

export function ParsedNotarizedTransactionAllOfValidationErrorFromJSON(json: any): ParsedNotarizedTransactionAllOfValidationError {
    return ParsedNotarizedTransactionAllOfValidationErrorFromJSONTyped(json, false);
}

export function ParsedNotarizedTransactionAllOfValidationErrorFromJSONTyped(json: any, ignoreDiscriminator: boolean): ParsedNotarizedTransactionAllOfValidationError {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'reason': json['reason'],
        'is_permanent': json['is_permanent'],
    };
}

export function ParsedNotarizedTransactionAllOfValidationErrorToJSON(value?: ParsedNotarizedTransactionAllOfValidationError | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'reason': value.reason,
        'is_permanent': value.is_permanent,
    };
}

