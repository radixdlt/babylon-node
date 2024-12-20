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
import type { NotarizedTransactionV2 } from './NotarizedTransactionV2';
import {
    NotarizedTransactionV2FromJSON,
    NotarizedTransactionV2FromJSONTyped,
    NotarizedTransactionV2ToJSON,
} from './NotarizedTransactionV2';

/**
 * 
 * @export
 * @interface UserLedgerTransactionV2
 */
export interface UserLedgerTransactionV2 {
    /**
     * 
     * @type {string}
     * @memberof UserLedgerTransactionV2
     */
    type: UserLedgerTransactionV2TypeEnum;
    /**
     * The hex-encoded full ledger transaction payload. Only returned if enabled in TransactionFormatOptions on your request.
     * @type {string}
     * @memberof UserLedgerTransactionV2
     */
    payload_hex?: string;
    /**
     * 
     * @type {NotarizedTransactionV2}
     * @memberof UserLedgerTransactionV2
     */
    notarized_transaction: NotarizedTransactionV2;
}


/**
 * @export
 */
export const UserLedgerTransactionV2TypeEnum = {
    UserV2: 'UserV2'
} as const;
export type UserLedgerTransactionV2TypeEnum = typeof UserLedgerTransactionV2TypeEnum[keyof typeof UserLedgerTransactionV2TypeEnum];


/**
 * Check if a given object implements the UserLedgerTransactionV2 interface.
 */
export function instanceOfUserLedgerTransactionV2(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "type" in value;
    isInstance = isInstance && "notarized_transaction" in value;

    return isInstance;
}

export function UserLedgerTransactionV2FromJSON(json: any): UserLedgerTransactionV2 {
    return UserLedgerTransactionV2FromJSONTyped(json, false);
}

export function UserLedgerTransactionV2FromJSONTyped(json: any, ignoreDiscriminator: boolean): UserLedgerTransactionV2 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'type': json['type'],
        'payload_hex': !exists(json, 'payload_hex') ? undefined : json['payload_hex'],
        'notarized_transaction': NotarizedTransactionV2FromJSON(json['notarized_transaction']),
    };
}

export function UserLedgerTransactionV2ToJSON(value?: UserLedgerTransactionV2 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'type': value.type,
        'payload_hex': value.payload_hex,
        'notarized_transaction': NotarizedTransactionV2ToJSON(value.notarized_transaction),
    };
}

