/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon (Bottlenose)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.1
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
import type { FunctionAuthType } from './FunctionAuthType';
import {
    FunctionAuthTypeFromJSON,
    FunctionAuthTypeFromJSONTyped,
    FunctionAuthTypeToJSON,
} from './FunctionAuthType';
import type { MethodAuthType } from './MethodAuthType';
import {
    MethodAuthTypeFromJSON,
    MethodAuthTypeFromJSONTyped,
    MethodAuthTypeToJSON,
} from './MethodAuthType';
import type { StaticRoleDefinitionAuthTemplate } from './StaticRoleDefinitionAuthTemplate';
import {
    StaticRoleDefinitionAuthTemplateFromJSON,
    StaticRoleDefinitionAuthTemplateFromJSONTyped,
    StaticRoleDefinitionAuthTemplateToJSON,
} from './StaticRoleDefinitionAuthTemplate';

/**
 * 
 * @export
 * @interface AuthConfig
 */
export interface AuthConfig {
    /**
     * 
     * @type {FunctionAuthType}
     * @memberof AuthConfig
     */
    function_auth_type: FunctionAuthType;
    /**
     * A map from a function name to AccessRule.
     * Only exists if `function_auth_type` is set to `FunctionAccessRules`.
     * @type {{ [key: string]: AccessRule; }}
     * @memberof AuthConfig
     */
    function_access_rules?: { [key: string]: AccessRule; };
    /**
     * 
     * @type {MethodAuthType}
     * @memberof AuthConfig
     */
    method_auth_type: MethodAuthType;
    /**
     * 
     * @type {StaticRoleDefinitionAuthTemplate}
     * @memberof AuthConfig
     */
    method_roles?: StaticRoleDefinitionAuthTemplate;
}

/**
 * Check if a given object implements the AuthConfig interface.
 */
export function instanceOfAuthConfig(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "function_auth_type" in value;
    isInstance = isInstance && "method_auth_type" in value;

    return isInstance;
}

export function AuthConfigFromJSON(json: any): AuthConfig {
    return AuthConfigFromJSONTyped(json, false);
}

export function AuthConfigFromJSONTyped(json: any, ignoreDiscriminator: boolean): AuthConfig {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'function_auth_type': FunctionAuthTypeFromJSON(json['function_auth_type']),
        'function_access_rules': !exists(json, 'function_access_rules') ? undefined : (mapValues(json['function_access_rules'], AccessRuleFromJSON)),
        'method_auth_type': MethodAuthTypeFromJSON(json['method_auth_type']),
        'method_roles': !exists(json, 'method_roles') ? undefined : StaticRoleDefinitionAuthTemplateFromJSON(json['method_roles']),
    };
}

export function AuthConfigToJSON(value?: AuthConfig | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'function_auth_type': FunctionAuthTypeToJSON(value.function_auth_type),
        'function_access_rules': value.function_access_rules === undefined ? undefined : (mapValues(value.function_access_rules, AccessRuleToJSON)),
        'method_auth_type': MethodAuthTypeToJSON(value.method_auth_type),
        'method_roles': StaticRoleDefinitionAuthTemplateToJSON(value.method_roles),
    };
}

