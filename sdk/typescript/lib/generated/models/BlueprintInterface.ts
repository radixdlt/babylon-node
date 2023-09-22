/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.0
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
import type { FunctionSchema } from './FunctionSchema';
import {
    FunctionSchemaFromJSON,
    FunctionSchemaFromJSONTyped,
    FunctionSchemaToJSON,
} from './FunctionSchema';
import type { GenericTypeParameter } from './GenericTypeParameter';
import {
    GenericTypeParameterFromJSON,
    GenericTypeParameterFromJSONTyped,
    GenericTypeParameterToJSON,
} from './GenericTypeParameter';
import type { IndexedStateSchema } from './IndexedStateSchema';
import {
    IndexedStateSchemaFromJSON,
    IndexedStateSchemaFromJSONTyped,
    IndexedStateSchemaToJSON,
} from './IndexedStateSchema';
import type { ScopedTypeId } from './ScopedTypeId';
import {
    ScopedTypeIdFromJSON,
    ScopedTypeIdFromJSONTyped,
    ScopedTypeIdToJSON,
} from './ScopedTypeId';

/**
 * 
 * @export
 * @interface BlueprintInterface
 */
export interface BlueprintInterface {
    /**
     * 
     * @type {string}
     * @memberof BlueprintInterface
     */
    outer_blueprint?: string;
    /**
     * Generic (SBOR) type parameters which need to be filled by a concrete instance
     * of this blueprint.
     * @type {Array<GenericTypeParameter>}
     * @memberof BlueprintInterface
     */
    generic_type_parameters: Array<GenericTypeParameter>;
    /**
     * If true, an instantiation of this blueprint cannot be persisted. EG buckets and proofs are transient.
     * @type {boolean}
     * @memberof BlueprintInterface
     */
    is_transient: boolean;
    /**
     * 
     * @type {Array<string>}
     * @memberof BlueprintInterface
     */
    features: Array<string>;
    /**
     * 
     * @type {IndexedStateSchema}
     * @memberof BlueprintInterface
     */
    state: IndexedStateSchema;
    /**
     * A map from the function name to the FunctionSchema
     * @type {{ [key: string]: FunctionSchema; }}
     * @memberof BlueprintInterface
     */
    functions: { [key: string]: FunctionSchema; };
    /**
     * A map from the event name to the event payload type reference.
     * @type {{ [key: string]: BlueprintPayloadDef; }}
     * @memberof BlueprintInterface
     */
    events: { [key: string]: BlueprintPayloadDef; };
    /**
     * A map from the registered type name to the concrete type,
     * resolved against a schema from the package's schema partition.
     * @type {{ [key: string]: ScopedTypeId; }}
     * @memberof BlueprintInterface
     */
    types: { [key: string]: ScopedTypeId; };
}

/**
 * Check if a given object implements the BlueprintInterface interface.
 */
export function instanceOfBlueprintInterface(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "generic_type_parameters" in value;
    isInstance = isInstance && "is_transient" in value;
    isInstance = isInstance && "features" in value;
    isInstance = isInstance && "state" in value;
    isInstance = isInstance && "functions" in value;
    isInstance = isInstance && "events" in value;
    isInstance = isInstance && "types" in value;

    return isInstance;
}

export function BlueprintInterfaceFromJSON(json: any): BlueprintInterface {
    return BlueprintInterfaceFromJSONTyped(json, false);
}

export function BlueprintInterfaceFromJSONTyped(json: any, ignoreDiscriminator: boolean): BlueprintInterface {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'outer_blueprint': !exists(json, 'outer_blueprint') ? undefined : json['outer_blueprint'],
        'generic_type_parameters': ((json['generic_type_parameters'] as Array<any>).map(GenericTypeParameterFromJSON)),
        'is_transient': json['is_transient'],
        'features': json['features'],
        'state': IndexedStateSchemaFromJSON(json['state']),
        'functions': (mapValues(json['functions'], FunctionSchemaFromJSON)),
        'events': (mapValues(json['events'], BlueprintPayloadDefFromJSON)),
        'types': (mapValues(json['types'], ScopedTypeIdFromJSON)),
    };
}

export function BlueprintInterfaceToJSON(value?: BlueprintInterface | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'outer_blueprint': value.outer_blueprint,
        'generic_type_parameters': ((value.generic_type_parameters as Array<any>).map(GenericTypeParameterToJSON)),
        'is_transient': value.is_transient,
        'features': value.features,
        'state': IndexedStateSchemaToJSON(value.state),
        'functions': (mapValues(value.functions, FunctionSchemaToJSON)),
        'events': (mapValues(value.events, BlueprintPayloadDefToJSON)),
        'types': (mapValues(value.types, ScopedTypeIdToJSON)),
    };
}

