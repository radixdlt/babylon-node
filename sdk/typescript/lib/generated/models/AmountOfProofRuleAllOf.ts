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
/**
 * 
 * @export
 * @interface AmountOfProofRuleAllOf
 */
export interface AmountOfProofRuleAllOf {
    /**
     * 
     * @type {string}
     * @memberof AmountOfProofRuleAllOf
     */
    amount: string;
    /**
     * The Bech32m-encoded human readable version of the resource address
     * @type {string}
     * @memberof AmountOfProofRuleAllOf
     */
    resource: string;
    /**
     * 
     * @type {string}
     * @memberof AmountOfProofRuleAllOf
     */
    type?: AmountOfProofRuleAllOfTypeEnum;
}


/**
 * @export
 */
export const AmountOfProofRuleAllOfTypeEnum = {
    AmountOf: 'AmountOf'
} as const;
export type AmountOfProofRuleAllOfTypeEnum = typeof AmountOfProofRuleAllOfTypeEnum[keyof typeof AmountOfProofRuleAllOfTypeEnum];


/**
 * Check if a given object implements the AmountOfProofRuleAllOf interface.
 */
export function instanceOfAmountOfProofRuleAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "amount" in value;
    isInstance = isInstance && "resource" in value;

    return isInstance;
}

export function AmountOfProofRuleAllOfFromJSON(json: any): AmountOfProofRuleAllOf {
    return AmountOfProofRuleAllOfFromJSONTyped(json, false);
}

export function AmountOfProofRuleAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): AmountOfProofRuleAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'amount': json['amount'],
        'resource': json['resource'],
        'type': !exists(json, 'type') ? undefined : json['type'],
    };
}

export function AmountOfProofRuleAllOfToJSON(value?: AmountOfProofRuleAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'amount': value.amount,
        'resource': value.resource,
        'type': value.type,
    };
}

