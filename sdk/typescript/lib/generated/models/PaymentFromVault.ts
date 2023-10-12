/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.4
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

/**
 * 
 * @export
 * @interface PaymentFromVault
 */
export interface PaymentFromVault {
    /**
     * 
     * @type {EntityReference}
     * @memberof PaymentFromVault
     */
    vault_entity: EntityReference;
    /**
     * The string-encoded decimal representing the amount of fee in XRD paid by this vault.
     * A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`.
     * @type {string}
     * @memberof PaymentFromVault
     */
    xrd_amount: string;
}

/**
 * Check if a given object implements the PaymentFromVault interface.
 */
export function instanceOfPaymentFromVault(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "vault_entity" in value;
    isInstance = isInstance && "xrd_amount" in value;

    return isInstance;
}

export function PaymentFromVaultFromJSON(json: any): PaymentFromVault {
    return PaymentFromVaultFromJSONTyped(json, false);
}

export function PaymentFromVaultFromJSONTyped(json: any, ignoreDiscriminator: boolean): PaymentFromVault {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'vault_entity': EntityReferenceFromJSON(json['vault_entity']),
        'xrd_amount': json['xrd_amount'],
    };
}

export function PaymentFromVaultToJSON(value?: PaymentFromVault | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'vault_entity': EntityReferenceToJSON(value.vault_entity),
        'xrd_amount': value.xrd_amount,
    };
}

