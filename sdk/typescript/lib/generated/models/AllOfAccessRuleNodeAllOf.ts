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
import type { AccessRuleNode } from './AccessRuleNode';
import {
    AccessRuleNodeFromJSON,
    AccessRuleNodeFromJSONTyped,
    AccessRuleNodeToJSON,
} from './AccessRuleNode';

/**
 * 
 * @export
 * @interface AllOfAccessRuleNodeAllOf
 */
export interface AllOfAccessRuleNodeAllOf {
    /**
     * 
     * @type {Array<AccessRuleNode>}
     * @memberof AllOfAccessRuleNodeAllOf
     */
    access_rules: Array<AccessRuleNode>;
    /**
     * 
     * @type {string}
     * @memberof AllOfAccessRuleNodeAllOf
     */
    type?: AllOfAccessRuleNodeAllOfTypeEnum;
}


/**
 * @export
 */
export const AllOfAccessRuleNodeAllOfTypeEnum = {
    AllOf: 'AllOf'
} as const;
export type AllOfAccessRuleNodeAllOfTypeEnum = typeof AllOfAccessRuleNodeAllOfTypeEnum[keyof typeof AllOfAccessRuleNodeAllOfTypeEnum];


/**
 * Check if a given object implements the AllOfAccessRuleNodeAllOf interface.
 */
export function instanceOfAllOfAccessRuleNodeAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "access_rules" in value;

    return isInstance;
}

export function AllOfAccessRuleNodeAllOfFromJSON(json: any): AllOfAccessRuleNodeAllOf {
    return AllOfAccessRuleNodeAllOfFromJSONTyped(json, false);
}

export function AllOfAccessRuleNodeAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): AllOfAccessRuleNodeAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'access_rules': ((json['access_rules'] as Array<any>).map(AccessRuleNodeFromJSON)),
        'type': !exists(json, 'type') ? undefined : json['type'],
    };
}

export function AllOfAccessRuleNodeAllOfToJSON(value?: AllOfAccessRuleNodeAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'access_rules': ((value.access_rules as Array<any>).map(AccessRuleNodeToJSON)),
        'type': value.type,
    };
}

