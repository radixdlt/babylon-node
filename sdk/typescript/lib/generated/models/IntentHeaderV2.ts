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
import type { ScryptoInstant } from './ScryptoInstant';
import {
    ScryptoInstantFromJSON,
    ScryptoInstantFromJSONTyped,
    ScryptoInstantToJSON,
} from './ScryptoInstant';

/**
 * The metadata common to both transaction intents and subintents.
 * 
 * The `min_proposer_timestamp_inclusive` and `max_proposer_timestamp_exclusive` are both optional.
 * @export
 * @interface IntentHeaderV2
 */
export interface IntentHeaderV2 {
    /**
     * The logical id of the network
     * @type {number}
     * @memberof IntentHeaderV2
     */
    network_id: number;
    /**
     * An integer between `0` and `10^10`, marking the epoch from which the transaction can be submitted.
     * In the case of uncommitted transactions, a value of `10^10` indicates that the epoch was >= `10^10`.
     * @type {number}
     * @memberof IntentHeaderV2
     */
    start_epoch_inclusive: number;
    /**
     * An integer between `0` and `10^10`, marking the epoch from which the transaction will no longer be valid, and be rejected.
     * In the case of uncommitted transactions, a value of `10^10` indicates that the epoch was >= `10^10`.
     * @type {number}
     * @memberof IntentHeaderV2
     */
    end_epoch_exclusive: number;
    /**
     * 
     * @type {ScryptoInstant}
     * @memberof IntentHeaderV2
     */
    min_proposer_timestamp_inclusive?: ScryptoInstant;
    /**
     * 
     * @type {ScryptoInstant}
     * @memberof IntentHeaderV2
     */
    max_proposer_timestamp_exclusive?: ScryptoInstant;
    /**
     * The string representation of an integer between `0` and `2^64 - 1`, which can be chosen to ensure that
     * a unique intent can be created. It only needs to be unique for a particular intent content and epoch/timestamp,
     * and can be safely selected randomly to very high probability.
     * 
     * This field was called `nonce` in the V1 models, but was a misleading name, as it got confused with a
     * cryptographic nonce or an Ethereum-style nonce, and it is neither.
     * @type {string}
     * @memberof IntentHeaderV2
     */
    intent_discriminator: string;
}

/**
 * Check if a given object implements the IntentHeaderV2 interface.
 */
export function instanceOfIntentHeaderV2(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "network_id" in value;
    isInstance = isInstance && "start_epoch_inclusive" in value;
    isInstance = isInstance && "end_epoch_exclusive" in value;
    isInstance = isInstance && "intent_discriminator" in value;

    return isInstance;
}

export function IntentHeaderV2FromJSON(json: any): IntentHeaderV2 {
    return IntentHeaderV2FromJSONTyped(json, false);
}

export function IntentHeaderV2FromJSONTyped(json: any, ignoreDiscriminator: boolean): IntentHeaderV2 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'network_id': json['network_id'],
        'start_epoch_inclusive': json['start_epoch_inclusive'],
        'end_epoch_exclusive': json['end_epoch_exclusive'],
        'min_proposer_timestamp_inclusive': !exists(json, 'min_proposer_timestamp_inclusive') ? undefined : ScryptoInstantFromJSON(json['min_proposer_timestamp_inclusive']),
        'max_proposer_timestamp_exclusive': !exists(json, 'max_proposer_timestamp_exclusive') ? undefined : ScryptoInstantFromJSON(json['max_proposer_timestamp_exclusive']),
        'intent_discriminator': json['intent_discriminator'],
    };
}

export function IntentHeaderV2ToJSON(value?: IntentHeaderV2 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'network_id': value.network_id,
        'start_epoch_inclusive': value.start_epoch_inclusive,
        'end_epoch_exclusive': value.end_epoch_exclusive,
        'min_proposer_timestamp_inclusive': ScryptoInstantToJSON(value.min_proposer_timestamp_inclusive),
        'max_proposer_timestamp_exclusive': ScryptoInstantToJSON(value.max_proposer_timestamp_exclusive),
        'intent_discriminator': value.intent_discriminator,
    };
}
