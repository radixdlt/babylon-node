/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
/**
 * Represents a proof from the execution of a non-genesis protocol update.
 * The execution of a protocol update is organised into batch groups, and
 * then these batch groups are organised into batches, with each batch committed
 * atomically.
 * 
 * NOTE: Some of these values may be placeholder values for protocol updates pre-Cuttlefish
 * on nodes which haven't resynced since Cuttlefish. In particular, the following values might be
 * invalid on such nodesL
 * 
 * * `config_hash` (placeholder of all zeros)
 * * `batch_group_idx` (placeholder of 0)
 * * `batch_group_name` (placeholder of "")
 * * `batch_idx` (whatever the non-grouped index was)
 * * `batch_name` (placeholder of "")
 * * `is_end_of_update` (placeholder of false)
 * @export
 * @interface ProtocolUpdateLedgerProofOriginAllOf
 */
export interface ProtocolUpdateLedgerProofOriginAllOf {
    /**
     * 
     * @type {string}
     * @memberof ProtocolUpdateLedgerProofOriginAllOf
     */
    protocol_version_name: string;
    /**
     * 
     * @type {string}
     * @memberof ProtocolUpdateLedgerProofOriginAllOf
     */
    config_hash: string;
    /**
     * 
     * @type {number}
     * @memberof ProtocolUpdateLedgerProofOriginAllOf
     */
    batch_group_idx: number;
    /**
     * 
     * @type {string}
     * @memberof ProtocolUpdateLedgerProofOriginAllOf
     */
    batch_group_name: string;
    /**
     * 
     * @type {number}
     * @memberof ProtocolUpdateLedgerProofOriginAllOf
     */
    batch_idx: number;
    /**
     * 
     * @type {string}
     * @memberof ProtocolUpdateLedgerProofOriginAllOf
     */
    batch_name: string;
    /**
     * 
     * @type {boolean}
     * @memberof ProtocolUpdateLedgerProofOriginAllOf
     */
    is_end_of_update: boolean;
    /**
     * 
     * @type {string}
     * @memberof ProtocolUpdateLedgerProofOriginAllOf
     */
    type?: ProtocolUpdateLedgerProofOriginAllOfTypeEnum;
}


/**
 * @export
 */
export const ProtocolUpdateLedgerProofOriginAllOfTypeEnum = {
    ProtocolUpdate: 'ProtocolUpdate'
} as const;
export type ProtocolUpdateLedgerProofOriginAllOfTypeEnum = typeof ProtocolUpdateLedgerProofOriginAllOfTypeEnum[keyof typeof ProtocolUpdateLedgerProofOriginAllOfTypeEnum];


/**
 * Check if a given object implements the ProtocolUpdateLedgerProofOriginAllOf interface.
 */
export function instanceOfProtocolUpdateLedgerProofOriginAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "protocol_version_name" in value;
    isInstance = isInstance && "config_hash" in value;
    isInstance = isInstance && "batch_group_idx" in value;
    isInstance = isInstance && "batch_group_name" in value;
    isInstance = isInstance && "batch_idx" in value;
    isInstance = isInstance && "batch_name" in value;
    isInstance = isInstance && "is_end_of_update" in value;

    return isInstance;
}

export function ProtocolUpdateLedgerProofOriginAllOfFromJSON(json: any): ProtocolUpdateLedgerProofOriginAllOf {
    return ProtocolUpdateLedgerProofOriginAllOfFromJSONTyped(json, false);
}

export function ProtocolUpdateLedgerProofOriginAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): ProtocolUpdateLedgerProofOriginAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'protocol_version_name': json['protocol_version_name'],
        'config_hash': json['config_hash'],
        'batch_group_idx': json['batch_group_idx'],
        'batch_group_name': json['batch_group_name'],
        'batch_idx': json['batch_idx'],
        'batch_name': json['batch_name'],
        'is_end_of_update': json['is_end_of_update'],
        'type': !exists(json, 'type') ? undefined : json['type'],
    };
}

export function ProtocolUpdateLedgerProofOriginAllOfToJSON(value?: ProtocolUpdateLedgerProofOriginAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'protocol_version_name': value.protocol_version_name,
        'config_hash': value.config_hash,
        'batch_group_idx': value.batch_group_idx,
        'batch_group_name': value.batch_group_name,
        'batch_idx': value.batch_idx,
        'batch_name': value.batch_name,
        'is_end_of_update': value.is_end_of_update,
        'type': value.type,
    };
}

