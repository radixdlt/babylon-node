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
import type { PlaintextMessageContent } from './PlaintextMessageContent';
import {
    PlaintextMessageContentFromJSON,
    PlaintextMessageContentFromJSONTyped,
    PlaintextMessageContentToJSON,
} from './PlaintextMessageContent';

/**
 * An unencrypted message.
 * @export
 * @interface PlaintextTransactionMessageAllOf
 */
export interface PlaintextTransactionMessageAllOf {
    /**
     * Intended to represent the RFC 2046 MIME type of the `content`.
     * A client cannot trust that this field is a valid mime type - in particular, the
     * choice between `String` or `Binary` representation of the content is not enforced by
     * this `mime_type`.
     * @type {string}
     * @memberof PlaintextTransactionMessageAllOf
     */
    mime_type: string;
    /**
     * 
     * @type {PlaintextMessageContent}
     * @memberof PlaintextTransactionMessageAllOf
     */
    content: PlaintextMessageContent;
    /**
     * 
     * @type {string}
     * @memberof PlaintextTransactionMessageAllOf
     */
    type?: PlaintextTransactionMessageAllOfTypeEnum;
}


/**
 * @export
 */
export const PlaintextTransactionMessageAllOfTypeEnum = {
    Plaintext: 'Plaintext'
} as const;
export type PlaintextTransactionMessageAllOfTypeEnum = typeof PlaintextTransactionMessageAllOfTypeEnum[keyof typeof PlaintextTransactionMessageAllOfTypeEnum];


/**
 * Check if a given object implements the PlaintextTransactionMessageAllOf interface.
 */
export function instanceOfPlaintextTransactionMessageAllOf(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "mime_type" in value;
    isInstance = isInstance && "content" in value;

    return isInstance;
}

export function PlaintextTransactionMessageAllOfFromJSON(json: any): PlaintextTransactionMessageAllOf {
    return PlaintextTransactionMessageAllOfFromJSONTyped(json, false);
}

export function PlaintextTransactionMessageAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): PlaintextTransactionMessageAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'mime_type': json['mime_type'],
        'content': PlaintextMessageContentFromJSON(json['content']),
        'type': !exists(json, 'type') ? undefined : json['type'],
    };
}

export function PlaintextTransactionMessageAllOfToJSON(value?: PlaintextTransactionMessageAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'mime_type': value.mime_type,
        'content': PlaintextMessageContentToJSON(value.content),
        'type': value.type,
    };
}

