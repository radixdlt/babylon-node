/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon (Anemone)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.1.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


import * as runtime from '../runtime';
import type {
  BasicErrorResponse,
  MempoolListRequest,
  MempoolListResponse,
  MempoolTransactionRequest,
  MempoolTransactionResponse,
} from '../models';
import {
    BasicErrorResponseFromJSON,
    BasicErrorResponseToJSON,
    MempoolListRequestFromJSON,
    MempoolListRequestToJSON,
    MempoolListResponseFromJSON,
    MempoolListResponseToJSON,
    MempoolTransactionRequestFromJSON,
    MempoolTransactionRequestToJSON,
    MempoolTransactionResponseFromJSON,
    MempoolTransactionResponseToJSON,
} from '../models';

export interface MempoolListPostRequest {
    mempoolListRequest: MempoolListRequest;
}

export interface MempoolTransactionPostRequest {
    mempoolTransactionRequest: MempoolTransactionRequest;
}

/**
 * 
 */
export class MempoolApi extends runtime.BaseAPI {

    /**
     * Returns the hashes of all the transactions currently in the mempool
     * Get Mempool List
     */
    async mempoolListPostRaw(requestParameters: MempoolListPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<MempoolListResponse>> {
        if (requestParameters.mempoolListRequest === null || requestParameters.mempoolListRequest === undefined) {
            throw new runtime.RequiredError('mempoolListRequest','Required parameter requestParameters.mempoolListRequest was null or undefined when calling mempoolListPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/mempool/list`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: MempoolListRequestToJSON(requestParameters.mempoolListRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => MempoolListResponseFromJSON(jsonValue));
    }

    /**
     * Returns the hashes of all the transactions currently in the mempool
     * Get Mempool List
     */
    async mempoolListPost(requestParameters: MempoolListPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<MempoolListResponse> {
        const response = await this.mempoolListPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Returns the payload of a transaction currently in the mempool
     * Get Mempool Transaction
     */
    async mempoolTransactionPostRaw(requestParameters: MempoolTransactionPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<MempoolTransactionResponse>> {
        if (requestParameters.mempoolTransactionRequest === null || requestParameters.mempoolTransactionRequest === undefined) {
            throw new runtime.RequiredError('mempoolTransactionRequest','Required parameter requestParameters.mempoolTransactionRequest was null or undefined when calling mempoolTransactionPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/mempool/transaction`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: MempoolTransactionRequestToJSON(requestParameters.mempoolTransactionRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => MempoolTransactionResponseFromJSON(jsonValue));
    }

    /**
     * Returns the payload of a transaction currently in the mempool
     * Get Mempool Transaction
     */
    async mempoolTransactionPost(requestParameters: MempoolTransactionPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<MempoolTransactionResponse> {
        const response = await this.mempoolTransactionPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

}
