/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet V1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.3.0
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
import type { GroupedAuthEntry } from './GroupedAuthEntry';
import {
    GroupedAuthEntryFromJSON,
    GroupedAuthEntryFromJSONTyped,
    GroupedAuthEntryToJSON,
} from './GroupedAuthEntry';
import type { MethodAuthEntry } from './MethodAuthEntry';
import {
    MethodAuthEntryFromJSON,
    MethodAuthEntryFromJSONTyped,
    MethodAuthEntryToJSON,
} from './MethodAuthEntry';
import type { MethodAuthMutabilityEntry } from './MethodAuthMutabilityEntry';
import {
    MethodAuthMutabilityEntryFromJSON,
    MethodAuthMutabilityEntryFromJSONTyped,
    MethodAuthMutabilityEntryToJSON,
} from './MethodAuthMutabilityEntry';

/**
 * 
 * @export
 * @interface AccessRules
 */
export interface AccessRules {
    /**
     * 
     * @type {Array<MethodAuthEntry>}
     * @memberof AccessRules
     */
    method_auth: Array<MethodAuthEntry>;
    /**
     * 
     * @type {Array<GroupedAuthEntry>}
     * @memberof AccessRules
     */
    grouped_auth: Array<GroupedAuthEntry>;
    /**
     * 
     * @type {AccessRule}
     * @memberof AccessRules
     */
    default_auth: AccessRule;
    /**
     * 
     * @type {Array<MethodAuthMutabilityEntry>}
     * @memberof AccessRules
     */
    method_auth_mutability: Array<MethodAuthMutabilityEntry>;
    /**
     * 
     * @type {Array<GroupedAuthEntry>}
     * @memberof AccessRules
     */
    grouped_auth_mutability: Array<GroupedAuthEntry>;
    /**
     * 
     * @type {AccessRule}
     * @memberof AccessRules
     */
    default_auth_mutability: AccessRule;
}

/**
 * Check if a given object implements the AccessRules interface.
 */
export function instanceOfAccessRules(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "method_auth" in value;
    isInstance = isInstance && "grouped_auth" in value;
    isInstance = isInstance && "default_auth" in value;
    isInstance = isInstance && "method_auth_mutability" in value;
    isInstance = isInstance && "grouped_auth_mutability" in value;
    isInstance = isInstance && "default_auth_mutability" in value;

    return isInstance;
}

export function AccessRulesFromJSON(json: any): AccessRules {
    return AccessRulesFromJSONTyped(json, false);
}

export function AccessRulesFromJSONTyped(json: any, ignoreDiscriminator: boolean): AccessRules {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'method_auth': ((json['method_auth'] as Array<any>).map(MethodAuthEntryFromJSON)),
        'grouped_auth': ((json['grouped_auth'] as Array<any>).map(GroupedAuthEntryFromJSON)),
        'default_auth': AccessRuleFromJSON(json['default_auth']),
        'method_auth_mutability': ((json['method_auth_mutability'] as Array<any>).map(MethodAuthMutabilityEntryFromJSON)),
        'grouped_auth_mutability': ((json['grouped_auth_mutability'] as Array<any>).map(GroupedAuthEntryFromJSON)),
        'default_auth_mutability': AccessRuleFromJSON(json['default_auth_mutability']),
    };
}

export function AccessRulesToJSON(value?: AccessRules | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'method_auth': ((value.method_auth as Array<any>).map(MethodAuthEntryToJSON)),
        'grouped_auth': ((value.grouped_auth as Array<any>).map(GroupedAuthEntryToJSON)),
        'default_auth': AccessRuleToJSON(value.default_auth),
        'method_auth_mutability': ((value.method_auth_mutability as Array<any>).map(MethodAuthMutabilityEntryToJSON)),
        'grouped_auth_mutability': ((value.grouped_auth_mutability as Array<any>).map(GroupedAuthEntryToJSON)),
        'default_auth_mutability': AccessRuleToJSON(value.default_auth_mutability),
    };
}
