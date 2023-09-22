/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { EcdsaSecp256k1PublicKey } from './EcdsaSecp256k1PublicKey';
import {
    EcdsaSecp256k1PublicKeyFromJSON,
    EcdsaSecp256k1PublicKeyFromJSONTyped,
    EcdsaSecp256k1PublicKeyToJSON,
} from './EcdsaSecp256k1PublicKey';
import type { EntityReference } from './EntityReference';
import {
    EntityReferenceFromJSON,
    EntityReferenceFromJSONTyped,
    EntityReferenceToJSON,
} from './EntityReference';
import type { PendingOwnerStakeWithdrawal } from './PendingOwnerStakeWithdrawal';
import {
    PendingOwnerStakeWithdrawalFromJSON,
    PendingOwnerStakeWithdrawalFromJSONTyped,
    PendingOwnerStakeWithdrawalToJSON,
} from './PendingOwnerStakeWithdrawal';
import type { SubstateKey } from './SubstateKey';
import {
    SubstateKeyFromJSON,
    SubstateKeyFromJSONTyped,
    SubstateKeyToJSON,
} from './SubstateKey';
import type { ValidatorFeeChangeRequest } from './ValidatorFeeChangeRequest';
import {
    ValidatorFeeChangeRequestFromJSON,
    ValidatorFeeChangeRequestFromJSONTyped,
    ValidatorFeeChangeRequestToJSON,
} from './ValidatorFeeChangeRequest';

/**
 * 
 * @export
 * @interface ValidatorFieldStateValue
 */
export interface ValidatorFieldStateValue {
    /**
     * 
     * @type {SubstateKey}
     * @memberof ValidatorFieldStateValue
     */
    sorted_key?: SubstateKey;
    /**
     * 
     * @type {EcdsaSecp256k1PublicKey}
     * @memberof ValidatorFieldStateValue
     */
    public_key: EcdsaSecp256k1PublicKey;
    /**
     * 
     * @type {boolean}
     * @memberof ValidatorFieldStateValue
     */
    is_registered: boolean;
    /**
     * 
     * @type {boolean}
     * @memberof ValidatorFieldStateValue
     */
    accepts_delegated_stake: boolean;
    /**
     * A string-encoded fixed-precision decimal to 18 decimal places.
     * A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`.
     * @type {string}
     * @memberof ValidatorFieldStateValue
     */
    validator_fee_factor: string;
    /**
     * 
     * @type {ValidatorFeeChangeRequest}
     * @memberof ValidatorFieldStateValue
     */
    validator_fee_change_request?: ValidatorFeeChangeRequest;
    /**
     * The Bech32m-encoded human readable version of the resource address
     * @type {string}
     * @memberof ValidatorFieldStateValue
     */
    stake_unit_resource_address: string;
    /**
     * 
     * @type {EntityReference}
     * @memberof ValidatorFieldStateValue
     */
    stake_xrd_vault: EntityReference;
    /**
     * The Bech32m-encoded human readable version of the resource address
     * @type {string}
     * @memberof ValidatorFieldStateValue
     */
    claim_token_resource_address: string;
    /**
     * 
     * @type {EntityReference}
     * @memberof ValidatorFieldStateValue
     */
    pending_xrd_withdraw_vault: EntityReference;
    /**
     * 
     * @type {EntityReference}
     * @memberof ValidatorFieldStateValue
     */
    locked_owner_stake_unit_vault: EntityReference;
    /**
     * 
     * @type {EntityReference}
     * @memberof ValidatorFieldStateValue
     */
    pending_owner_stake_unit_unlock_vault: EntityReference;
    /**
     * 
     * @type {Array<PendingOwnerStakeWithdrawal>}
     * @memberof ValidatorFieldStateValue
     */
    pending_owner_stake_unit_withdrawals: Array<PendingOwnerStakeWithdrawal>;
    /**
     * A string-encoded fixed-precision decimal to 18 decimal places.
     * A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`.
     * @type {string}
     * @memberof ValidatorFieldStateValue
     */
    already_unlocked_owner_stake_unit_amount: string;
}

/**
 * Check if a given object implements the ValidatorFieldStateValue interface.
 */
export function instanceOfValidatorFieldStateValue(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "public_key" in value;
    isInstance = isInstance && "is_registered" in value;
    isInstance = isInstance && "accepts_delegated_stake" in value;
    isInstance = isInstance && "validator_fee_factor" in value;
    isInstance = isInstance && "stake_unit_resource_address" in value;
    isInstance = isInstance && "stake_xrd_vault" in value;
    isInstance = isInstance && "claim_token_resource_address" in value;
    isInstance = isInstance && "pending_xrd_withdraw_vault" in value;
    isInstance = isInstance && "locked_owner_stake_unit_vault" in value;
    isInstance = isInstance && "pending_owner_stake_unit_unlock_vault" in value;
    isInstance = isInstance && "pending_owner_stake_unit_withdrawals" in value;
    isInstance = isInstance && "already_unlocked_owner_stake_unit_amount" in value;

    return isInstance;
}

export function ValidatorFieldStateValueFromJSON(json: any): ValidatorFieldStateValue {
    return ValidatorFieldStateValueFromJSONTyped(json, false);
}

export function ValidatorFieldStateValueFromJSONTyped(json: any, ignoreDiscriminator: boolean): ValidatorFieldStateValue {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'sorted_key': !exists(json, 'sorted_key') ? undefined : SubstateKeyFromJSON(json['sorted_key']),
        'public_key': EcdsaSecp256k1PublicKeyFromJSON(json['public_key']),
        'is_registered': json['is_registered'],
        'accepts_delegated_stake': json['accepts_delegated_stake'],
        'validator_fee_factor': json['validator_fee_factor'],
        'validator_fee_change_request': !exists(json, 'validator_fee_change_request') ? undefined : ValidatorFeeChangeRequestFromJSON(json['validator_fee_change_request']),
        'stake_unit_resource_address': json['stake_unit_resource_address'],
        'stake_xrd_vault': EntityReferenceFromJSON(json['stake_xrd_vault']),
        'claim_token_resource_address': json['claim_token_resource_address'],
        'pending_xrd_withdraw_vault': EntityReferenceFromJSON(json['pending_xrd_withdraw_vault']),
        'locked_owner_stake_unit_vault': EntityReferenceFromJSON(json['locked_owner_stake_unit_vault']),
        'pending_owner_stake_unit_unlock_vault': EntityReferenceFromJSON(json['pending_owner_stake_unit_unlock_vault']),
        'pending_owner_stake_unit_withdrawals': ((json['pending_owner_stake_unit_withdrawals'] as Array<any>).map(PendingOwnerStakeWithdrawalFromJSON)),
        'already_unlocked_owner_stake_unit_amount': json['already_unlocked_owner_stake_unit_amount'],
    };
}

export function ValidatorFieldStateValueToJSON(value?: ValidatorFieldStateValue | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'sorted_key': SubstateKeyToJSON(value.sorted_key),
        'public_key': EcdsaSecp256k1PublicKeyToJSON(value.public_key),
        'is_registered': value.is_registered,
        'accepts_delegated_stake': value.accepts_delegated_stake,
        'validator_fee_factor': value.validator_fee_factor,
        'validator_fee_change_request': ValidatorFeeChangeRequestToJSON(value.validator_fee_change_request),
        'stake_unit_resource_address': value.stake_unit_resource_address,
        'stake_xrd_vault': EntityReferenceToJSON(value.stake_xrd_vault),
        'claim_token_resource_address': value.claim_token_resource_address,
        'pending_xrd_withdraw_vault': EntityReferenceToJSON(value.pending_xrd_withdraw_vault),
        'locked_owner_stake_unit_vault': EntityReferenceToJSON(value.locked_owner_stake_unit_vault),
        'pending_owner_stake_unit_unlock_vault': EntityReferenceToJSON(value.pending_owner_stake_unit_unlock_vault),
        'pending_owner_stake_unit_withdrawals': ((value.pending_owner_stake_unit_withdrawals as Array<any>).map(PendingOwnerStakeWithdrawalToJSON)),
        'already_unlocked_owner_stake_unit_amount': value.already_unlocked_owner_stake_unit_amount,
    };
}

