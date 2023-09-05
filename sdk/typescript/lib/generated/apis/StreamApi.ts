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


import * as runtime from '../runtime';
import type {
  BasicErrorResponse,
  StreamTransactionsRequest,
  StreamTransactionsResponse,
} from '../models';
import {
    BasicErrorResponseFromJSON,
    BasicErrorResponseToJSON,
    StreamTransactionsRequestFromJSON,
    StreamTransactionsRequestToJSON,
    StreamTransactionsResponseFromJSON,
    StreamTransactionsResponseToJSON,
} from '../models';

export interface StreamTransactionsPostRequest {
    streamTransactionsRequest: StreamTransactionsRequest;
}

/**
 * 
 */
export class StreamApi extends runtime.BaseAPI {

    /**
     * Returns the list of committed transactions. 
     * Get Committed Transactions
     */
    async streamTransactionsPostRaw(requestParameters: StreamTransactionsPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StreamTransactionsResponse>> {
        if (requestParameters.streamTransactionsRequest === null || requestParameters.streamTransactionsRequest === undefined) {
            throw new runtime.RequiredError('streamTransactionsRequest','Required parameter requestParameters.streamTransactionsRequest was null or undefined when calling streamTransactionsPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/stream/transactions`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StreamTransactionsRequestToJSON(requestParameters.streamTransactionsRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StreamTransactionsResponseFromJSON(jsonValue));
    }

    /**
     * Returns the list of committed transactions. 
     * Get Committed Transactions
     */
    async streamTransactionsPost(requestParameters: StreamTransactionsPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StreamTransactionsResponse> {
        const response = await this.streamTransactionsPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

}
