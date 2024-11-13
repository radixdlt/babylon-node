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
import type { MethodAccessibility } from './MethodAccessibility';
import {
    MethodAccessibilityFromJSON,
    MethodAccessibilityFromJSONTyped,
    MethodAccessibilityToJSON,
} from './MethodAccessibility';
import type { RoleDetails } from './RoleDetails';
import {
    RoleDetailsFromJSON,
    RoleDetailsFromJSONTyped,
    RoleDetailsToJSON,
} from './RoleDetails';
import type { RoleSpecification } from './RoleSpecification';
import {
    RoleSpecificationFromJSON,
    RoleSpecificationFromJSONTyped,
    RoleSpecificationToJSON,
} from './RoleSpecification';

/**
 * 
 * @export
 * @interface StaticRoleDefinitionAuthTemplate
 */
export interface StaticRoleDefinitionAuthTemplate {
    /**
     * 
     * @type {RoleSpecification}
     * @memberof StaticRoleDefinitionAuthTemplate
     */
    role_specification: RoleSpecification;
    /**
     * A map from role name to role details
     * @type {{ [key: string]: RoleDetails; }}
     * @memberof StaticRoleDefinitionAuthTemplate
     */
    roles?: { [key: string]: RoleDetails; };
    /**
     * A map from a method identifier to MethodAccessibility
     * @type {{ [key: string]: MethodAccessibility; }}
     * @memberof StaticRoleDefinitionAuthTemplate
     */
    method_accessibility_map: { [key: string]: MethodAccessibility; };
}

/**
 * Check if a given object implements the StaticRoleDefinitionAuthTemplate interface.
 */
export function instanceOfStaticRoleDefinitionAuthTemplate(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "role_specification" in value;
    isInstance = isInstance && "method_accessibility_map" in value;

    return isInstance;
}

export function StaticRoleDefinitionAuthTemplateFromJSON(json: any): StaticRoleDefinitionAuthTemplate {
    return StaticRoleDefinitionAuthTemplateFromJSONTyped(json, false);
}

export function StaticRoleDefinitionAuthTemplateFromJSONTyped(json: any, ignoreDiscriminator: boolean): StaticRoleDefinitionAuthTemplate {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'role_specification': RoleSpecificationFromJSON(json['role_specification']),
        'roles': !exists(json, 'roles') ? undefined : (mapValues(json['roles'], RoleDetailsFromJSON)),
        'method_accessibility_map': (mapValues(json['method_accessibility_map'], MethodAccessibilityFromJSON)),
    };
}

export function StaticRoleDefinitionAuthTemplateToJSON(value?: StaticRoleDefinitionAuthTemplate | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'role_specification': RoleSpecificationToJSON(value.role_specification),
        'roles': value.roles === undefined ? undefined : (mapValues(value.roles, RoleDetailsToJSON)),
        'method_accessibility_map': (mapValues(value.method_accessibility_map, MethodAccessibilityToJSON)),
    };
}

