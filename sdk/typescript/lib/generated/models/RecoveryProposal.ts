/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.2
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { AccessRule } from './AccessRule';
import {
    AccessRuleFromJSON,
    AccessRuleFromJSONTyped,
    AccessRuleToJSON,
} from './AccessRule';

/**
 * 
 * @export
 * @interface RecoveryProposal
 */
export interface RecoveryProposal {
    /**
     * 
     * @type {AccessRule}
     * @memberof RecoveryProposal
     */
    primary_role: AccessRule;
    /**
     * 
     * @type {AccessRule}
     * @memberof RecoveryProposal
     */
    recovery_role: AccessRule;
    /**
     * 
     * @type {AccessRule}
     * @memberof RecoveryProposal
     */
    confirmation_role: AccessRule;
    /**
     * An integer between `0` and `2^32 - 1`, specifying the optional proposal delay of timed recoveries.
     * @type {number}
     * @memberof RecoveryProposal
     */
    timed_recovery_delay_minutes?: number;
}

/**
 * Check if a given object implements the RecoveryProposal interface.
 */
export function instanceOfRecoveryProposal(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "primary_role" in value;
    isInstance = isInstance && "recovery_role" in value;
    isInstance = isInstance && "confirmation_role" in value;

    return isInstance;
}

export function RecoveryProposalFromJSON(json: any): RecoveryProposal {
    return RecoveryProposalFromJSONTyped(json, false);
}

export function RecoveryProposalFromJSONTyped(json: any, ignoreDiscriminator: boolean): RecoveryProposal {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'primary_role': AccessRuleFromJSON(json['primary_role']),
        'recovery_role': AccessRuleFromJSON(json['recovery_role']),
        'confirmation_role': AccessRuleFromJSON(json['confirmation_role']),
        'timed_recovery_delay_minutes': !exists(json, 'timed_recovery_delay_minutes') ? undefined : json['timed_recovery_delay_minutes'],
    };
}

export function RecoveryProposalToJSON(value?: RecoveryProposal | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'primary_role': AccessRuleToJSON(value.primary_role),
        'recovery_role': AccessRuleToJSON(value.recovery_role),
        'confirmation_role': AccessRuleToJSON(value.confirmation_role),
        'timed_recovery_delay_minutes': value.timed_recovery_delay_minutes,
    };
}

