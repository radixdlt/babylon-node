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
import type { BootLoaderModuleFieldSystemBootValue } from './BootLoaderModuleFieldSystemBootValue';
import {
    BootLoaderModuleFieldSystemBootValueFromJSON,
    BootLoaderModuleFieldSystemBootValueFromJSONTyped,
    BootLoaderModuleFieldSystemBootValueToJSON,
} from './BootLoaderModuleFieldSystemBootValue';

/**
 * 
 * @export
 * @interface BootLoaderModuleFieldSystemBootSubstate
 */
export interface BootLoaderModuleFieldSystemBootSubstate {
    /**
     * 
     * @type {string}
     * @memberof BootLoaderModuleFieldSystemBootSubstate
     */
    substate_type: BootLoaderModuleFieldSystemBootSubstateSubstateTypeEnum;
    /**
     * 
     * @type {boolean}
     * @memberof BootLoaderModuleFieldSystemBootSubstate
     */
    is_locked: boolean;
    /**
     * 
     * @type {BootLoaderModuleFieldSystemBootValue}
     * @memberof BootLoaderModuleFieldSystemBootSubstate
     */
    value: BootLoaderModuleFieldSystemBootValue;
}


/**
 * @export
 */
export const BootLoaderModuleFieldSystemBootSubstateSubstateTypeEnum = {
    BootLoaderModuleFieldSystemBoot: 'BootLoaderModuleFieldSystemBoot'
} as const;
export type BootLoaderModuleFieldSystemBootSubstateSubstateTypeEnum = typeof BootLoaderModuleFieldSystemBootSubstateSubstateTypeEnum[keyof typeof BootLoaderModuleFieldSystemBootSubstateSubstateTypeEnum];


/**
 * Check if a given object implements the BootLoaderModuleFieldSystemBootSubstate interface.
 */
export function instanceOfBootLoaderModuleFieldSystemBootSubstate(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "substate_type" in value;
    isInstance = isInstance && "is_locked" in value;
    isInstance = isInstance && "value" in value;

    return isInstance;
}

export function BootLoaderModuleFieldSystemBootSubstateFromJSON(json: any): BootLoaderModuleFieldSystemBootSubstate {
    return BootLoaderModuleFieldSystemBootSubstateFromJSONTyped(json, false);
}

export function BootLoaderModuleFieldSystemBootSubstateFromJSONTyped(json: any, ignoreDiscriminator: boolean): BootLoaderModuleFieldSystemBootSubstate {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'substate_type': json['substate_type'],
        'is_locked': json['is_locked'],
        'value': BootLoaderModuleFieldSystemBootValueFromJSON(json['value']),
    };
}

export function BootLoaderModuleFieldSystemBootSubstateToJSON(value?: BootLoaderModuleFieldSystemBootSubstate | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'substate_type': value.substate_type,
        'is_locked': value.is_locked,
        'value': BootLoaderModuleFieldSystemBootValueToJSON(value.value),
    };
}

