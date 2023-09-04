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
import type { EntityReference } from './EntityReference';
import {
    EntityReferenceFromJSON,
    EntityReferenceFromJSONTyped,
    EntityReferenceToJSON,
} from './EntityReference';
import type { PrimaryRoleRecoveryAttempt } from './PrimaryRoleRecoveryAttempt';
import {
    PrimaryRoleRecoveryAttemptFromJSON,
    PrimaryRoleRecoveryAttemptFromJSONTyped,
    PrimaryRoleRecoveryAttemptToJSON,
} from './PrimaryRoleRecoveryAttempt';
import type { RecoveryRoleRecoveryAttempt } from './RecoveryRoleRecoveryAttempt';
import {
    RecoveryRoleRecoveryAttemptFromJSON,
    RecoveryRoleRecoveryAttemptFromJSONTyped,
    RecoveryRoleRecoveryAttemptToJSON,
} from './RecoveryRoleRecoveryAttempt';

/**
 * 
 * @export
 * @interface AccessControllerFieldStateValue
 */
export interface AccessControllerFieldStateValue {
    /**
     * 
     * @type {EntityReference}
     * @memberof AccessControllerFieldStateValue
     */
    controlled_vault: EntityReference;
    /**
     * An integer between `0` and `2^32 - 1`, specifying the amount of time (in minutes) that
     * it takes for timed recovery to be done. When not present, then timed recovery can not be
     * performed through this access controller.
     * @type {number}
     * @memberof AccessControllerFieldStateValue
     */
    timed_recovery_delay_minutes?: number;
    /**
     * The Bech32m-encoded human readable version of the resource address
     * @type {string}
     * @memberof AccessControllerFieldStateValue
     */
    recovery_badge_resource_address: string;
    /**
     * Whether the primary role is currently locked.
     * @type {boolean}
     * @memberof AccessControllerFieldStateValue
     */
    is_primary_role_locked: boolean;
    /**
     * 
     * @type {PrimaryRoleRecoveryAttempt}
     * @memberof AccessControllerFieldStateValue
     */
    primary_role_recovery_attempt?: PrimaryRoleRecoveryAttempt;
    /**
     * Whether the primary role badge withdraw is currently being attempted.
     * @type {boolean}
     * @memberof AccessControllerFieldStateValue
     */
    has_primary_role_badge_withdraw_attempt: boolean;
    /**
     * 
     * @type {RecoveryRoleRecoveryAttempt}
     * @memberof AccessControllerFieldStateValue
     */
    recovery_role_recovery_attempt?: RecoveryRoleRecoveryAttempt;
    /**
     * Whether the recovery role badge withdraw is currently being attempted.
     * @type {boolean}
     * @memberof AccessControllerFieldStateValue
     */
    has_recovery_role_badge_withdraw_attempt: boolean;
}

/**
 * Check if a given object implements the AccessControllerFieldStateValue interface.
 */
export function instanceOfAccessControllerFieldStateValue(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "controlled_vault" in value;
    isInstance = isInstance && "recovery_badge_resource_address" in value;
    isInstance = isInstance && "is_primary_role_locked" in value;
    isInstance = isInstance && "has_primary_role_badge_withdraw_attempt" in value;
    isInstance = isInstance && "has_recovery_role_badge_withdraw_attempt" in value;

    return isInstance;
}

export function AccessControllerFieldStateValueFromJSON(json: any): AccessControllerFieldStateValue {
    return AccessControllerFieldStateValueFromJSONTyped(json, false);
}

export function AccessControllerFieldStateValueFromJSONTyped(json: any, ignoreDiscriminator: boolean): AccessControllerFieldStateValue {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'controlled_vault': EntityReferenceFromJSON(json['controlled_vault']),
        'timed_recovery_delay_minutes': !exists(json, 'timed_recovery_delay_minutes') ? undefined : json['timed_recovery_delay_minutes'],
        'recovery_badge_resource_address': json['recovery_badge_resource_address'],
        'is_primary_role_locked': json['is_primary_role_locked'],
        'primary_role_recovery_attempt': !exists(json, 'primary_role_recovery_attempt') ? undefined : PrimaryRoleRecoveryAttemptFromJSON(json['primary_role_recovery_attempt']),
        'has_primary_role_badge_withdraw_attempt': json['has_primary_role_badge_withdraw_attempt'],
        'recovery_role_recovery_attempt': !exists(json, 'recovery_role_recovery_attempt') ? undefined : RecoveryRoleRecoveryAttemptFromJSON(json['recovery_role_recovery_attempt']),
        'has_recovery_role_badge_withdraw_attempt': json['has_recovery_role_badge_withdraw_attempt'],
    };
}

export function AccessControllerFieldStateValueToJSON(value?: AccessControllerFieldStateValue | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'controlled_vault': EntityReferenceToJSON(value.controlled_vault),
        'timed_recovery_delay_minutes': value.timed_recovery_delay_minutes,
        'recovery_badge_resource_address': value.recovery_badge_resource_address,
        'is_primary_role_locked': value.is_primary_role_locked,
        'primary_role_recovery_attempt': PrimaryRoleRecoveryAttemptToJSON(value.primary_role_recovery_attempt),
        'has_primary_role_badge_withdraw_attempt': value.has_primary_role_badge_withdraw_attempt,
        'recovery_role_recovery_attempt': RecoveryRoleRecoveryAttemptToJSON(value.recovery_role_recovery_attempt),
        'has_recovery_role_badge_withdraw_attempt': value.has_recovery_role_badge_withdraw_attempt,
    };
}

