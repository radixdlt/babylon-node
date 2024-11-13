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
import type { LtsCommittedTransactionStatus } from './LtsCommittedTransactionStatus';
import {
    LtsCommittedTransactionStatusFromJSON,
    LtsCommittedTransactionStatusFromJSONTyped,
    LtsCommittedTransactionStatusToJSON,
} from './LtsCommittedTransactionStatus';
import type { LtsEntityFungibleBalanceChanges } from './LtsEntityFungibleBalanceChanges';
import {
    LtsEntityFungibleBalanceChangesFromJSON,
    LtsEntityFungibleBalanceChangesFromJSONTyped,
    LtsEntityFungibleBalanceChangesToJSON,
} from './LtsEntityFungibleBalanceChanges';
import type { LtsEntityNonFungibleBalanceChanges } from './LtsEntityNonFungibleBalanceChanges';
import {
    LtsEntityNonFungibleBalanceChangesFromJSON,
    LtsEntityNonFungibleBalanceChangesFromJSONTyped,
    LtsEntityNonFungibleBalanceChangesToJSON,
} from './LtsEntityNonFungibleBalanceChanges';
import type { LtsResultantAccountFungibleBalances } from './LtsResultantAccountFungibleBalances';
import {
    LtsResultantAccountFungibleBalancesFromJSON,
    LtsResultantAccountFungibleBalancesFromJSONTyped,
    LtsResultantAccountFungibleBalancesToJSON,
} from './LtsResultantAccountFungibleBalances';
import type { TransactionIdentifiers } from './TransactionIdentifiers';
import {
    TransactionIdentifiersFromJSON,
    TransactionIdentifiersFromJSONTyped,
    TransactionIdentifiersToJSON,
} from './TransactionIdentifiers';

/**
 * For the given transaction, contains the status, total fee summary and individual entity resource balance changes.
 * The balance changes accounts for the fee payments as well.
 * @export
 * @interface LtsCommittedTransactionOutcome
 */
export interface LtsCommittedTransactionOutcome {
    /**
     * 
     * @type {number}
     * @memberof LtsCommittedTransactionOutcome
     */
    state_version: number;
    /**
     * An integer between `0` and `10^14`, marking the proposer timestamp in ms.
     * @type {number}
     * @memberof LtsCommittedTransactionOutcome
     */
    proposer_timestamp_ms: number;
    /**
     * The hex-encoded transaction accumulator hash. This hash captures the order of all transactions on ledger.
     * This hash is `ACC_{N+1} = combine(ACC_N, LEDGER_HASH_{N}))` (where `combine()` is an arbitrary deterministic function we use).
     * @type {string}
     * @memberof LtsCommittedTransactionOutcome
     */
    accumulator_hash: string;
    /**
     * 
     * @type {TransactionIdentifiers}
     * @memberof LtsCommittedTransactionOutcome
     */
    user_transaction_identifiers?: TransactionIdentifiers;
    /**
     * 
     * @type {LtsCommittedTransactionStatus}
     * @memberof LtsCommittedTransactionOutcome
     */
    status: LtsCommittedTransactionStatus;
    /**
     * A list of all fungible balance updates which occurred in this transaction, aggregated by the global entity (such as account)
     * which owns the vaults which were updated.
     * @type {Array<LtsEntityFungibleBalanceChanges>}
     * @memberof LtsCommittedTransactionOutcome
     */
    fungible_entity_balance_changes: Array<LtsEntityFungibleBalanceChanges>;
    /**
     * Non fungible changes per entity and resource
     * @type {Array<LtsEntityNonFungibleBalanceChanges>}
     * @memberof LtsCommittedTransactionOutcome
     */
    non_fungible_entity_balance_changes: Array<LtsEntityNonFungibleBalanceChanges>;
    /**
     * A list of the resultant fungible account balances for any balances which changed in this transaction.
     * Only balances for accounts are returned, not any other kind of entity.
     * @type {Array<LtsResultantAccountFungibleBalances>}
     * @memberof LtsCommittedTransactionOutcome
     */
    resultant_account_fungible_balances: Array<LtsResultantAccountFungibleBalances>;
    /**
     * The string-encoded decimal representing the total amount of XRD paid as fee (execution, validator tip and royalties).
     * A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`.
     * @type {string}
     * @memberof LtsCommittedTransactionOutcome
     */
    total_fee: string;
}

/**
 * Check if a given object implements the LtsCommittedTransactionOutcome interface.
 */
export function instanceOfLtsCommittedTransactionOutcome(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "state_version" in value;
    isInstance = isInstance && "proposer_timestamp_ms" in value;
    isInstance = isInstance && "accumulator_hash" in value;
    isInstance = isInstance && "status" in value;
    isInstance = isInstance && "fungible_entity_balance_changes" in value;
    isInstance = isInstance && "non_fungible_entity_balance_changes" in value;
    isInstance = isInstance && "resultant_account_fungible_balances" in value;
    isInstance = isInstance && "total_fee" in value;

    return isInstance;
}

export function LtsCommittedTransactionOutcomeFromJSON(json: any): LtsCommittedTransactionOutcome {
    return LtsCommittedTransactionOutcomeFromJSONTyped(json, false);
}

export function LtsCommittedTransactionOutcomeFromJSONTyped(json: any, ignoreDiscriminator: boolean): LtsCommittedTransactionOutcome {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'state_version': json['state_version'],
        'proposer_timestamp_ms': json['proposer_timestamp_ms'],
        'accumulator_hash': json['accumulator_hash'],
        'user_transaction_identifiers': !exists(json, 'user_transaction_identifiers') ? undefined : TransactionIdentifiersFromJSON(json['user_transaction_identifiers']),
        'status': LtsCommittedTransactionStatusFromJSON(json['status']),
        'fungible_entity_balance_changes': ((json['fungible_entity_balance_changes'] as Array<any>).map(LtsEntityFungibleBalanceChangesFromJSON)),
        'non_fungible_entity_balance_changes': ((json['non_fungible_entity_balance_changes'] as Array<any>).map(LtsEntityNonFungibleBalanceChangesFromJSON)),
        'resultant_account_fungible_balances': ((json['resultant_account_fungible_balances'] as Array<any>).map(LtsResultantAccountFungibleBalancesFromJSON)),
        'total_fee': json['total_fee'],
    };
}

export function LtsCommittedTransactionOutcomeToJSON(value?: LtsCommittedTransactionOutcome | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'state_version': value.state_version,
        'proposer_timestamp_ms': value.proposer_timestamp_ms,
        'accumulator_hash': value.accumulator_hash,
        'user_transaction_identifiers': TransactionIdentifiersToJSON(value.user_transaction_identifiers),
        'status': LtsCommittedTransactionStatusToJSON(value.status),
        'fungible_entity_balance_changes': ((value.fungible_entity_balance_changes as Array<any>).map(LtsEntityFungibleBalanceChangesToJSON)),
        'non_fungible_entity_balance_changes': ((value.non_fungible_entity_balance_changes as Array<any>).map(LtsEntityNonFungibleBalanceChangesToJSON)),
        'resultant_account_fungible_balances': ((value.resultant_account_fungible_balances as Array<any>).map(LtsResultantAccountFungibleBalancesToJSON)),
        'total_fee': value.total_fee,
    };
}

