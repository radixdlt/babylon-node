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
import type { BlueprintPayloadDef } from './BlueprintPayloadDef';
import {
    BlueprintPayloadDefFromJSON,
    BlueprintPayloadDefFromJSONTyped,
    BlueprintPayloadDefToJSON,
} from './BlueprintPayloadDef';
import type { FieldSchemaFeatureCondition } from './FieldSchemaFeatureCondition';
import {
    FieldSchemaFeatureConditionFromJSON,
    FieldSchemaFeatureConditionFromJSONTyped,
    FieldSchemaFeatureConditionToJSON,
} from './FieldSchemaFeatureCondition';

/**
 * 
 * @export
 * @interface FieldSchema
 */
export interface FieldSchema {
    /**
     * 
     * @type {BlueprintPayloadDef}
     * @memberof FieldSchema
     */
    field_type_ref: BlueprintPayloadDef;
    /**
     * 
     * @type {FieldSchemaFeatureCondition}
     * @memberof FieldSchema
     */
    condition?: FieldSchemaFeatureCondition;
    /**
     * The hex-encoded default value of this field. Only present if this field is transient.
     * @type {string}
     * @memberof FieldSchema
     */
    transient_default_value_hex?: string;
}

/**
 * Check if a given object implements the FieldSchema interface.
 */
export function instanceOfFieldSchema(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "field_type_ref" in value;

    return isInstance;
}

export function FieldSchemaFromJSON(json: any): FieldSchema {
    return FieldSchemaFromJSONTyped(json, false);
}

export function FieldSchemaFromJSONTyped(json: any, ignoreDiscriminator: boolean): FieldSchema {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'field_type_ref': BlueprintPayloadDefFromJSON(json['field_type_ref']),
        'condition': !exists(json, 'condition') ? undefined : FieldSchemaFeatureConditionFromJSON(json['condition']),
        'transient_default_value_hex': !exists(json, 'transient_default_value_hex') ? undefined : json['transient_default_value_hex'],
    };
}

export function FieldSchemaToJSON(value?: FieldSchema | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'field_type_ref': BlueprintPayloadDefToJSON(value.field_type_ref),
        'condition': FieldSchemaFeatureConditionToJSON(value.condition),
        'transient_default_value_hex': value.transient_default_value_hex,
    };
}

