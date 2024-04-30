/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon (Anemone)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.1.3
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


import * as runtime from '../runtime';
import type {
  BasicErrorResponse,
  NetworkConfigurationResponse,
  NetworkStatusRequest,
  NetworkStatusResponse,
  ScenariosRequest,
  ScenariosResponse,
} from '../models';
import {
    BasicErrorResponseFromJSON,
    BasicErrorResponseToJSON,
    NetworkConfigurationResponseFromJSON,
    NetworkConfigurationResponseToJSON,
    NetworkStatusRequestFromJSON,
    NetworkStatusRequestToJSON,
    NetworkStatusResponseFromJSON,
    NetworkStatusResponseToJSON,
    ScenariosRequestFromJSON,
    ScenariosRequestToJSON,
    ScenariosResponseFromJSON,
    ScenariosResponseToJSON,
} from '../models';

export interface StatusNetworkStatusPostRequest {
    networkStatusRequest: NetworkStatusRequest;
}

export interface StatusScenariosPostRequest {
    scenariosRequest: ScenariosRequest;
}

/**
 * 
 */
export class StatusApi extends runtime.BaseAPI {

    /**
     * Returns the network configuration of the network the node is connected to.
     * Get Network Configuration
     */
    async statusNetworkConfigurationPostRaw(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<NetworkConfigurationResponse>> {
        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/status/network-configuration`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => NetworkConfigurationResponseFromJSON(jsonValue));
    }

    /**
     * Returns the network configuration of the network the node is connected to.
     * Get Network Configuration
     */
    async statusNetworkConfigurationPost(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<NetworkConfigurationResponse> {
        const response = await this.statusNetworkConfigurationPostRaw(initOverrides);
        return await response.value();
    }

    /**
     * Returns the current state and status of the node\'s copy of the ledger.
     * Get Network Status
     */
    async statusNetworkStatusPostRaw(requestParameters: StatusNetworkStatusPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<NetworkStatusResponse>> {
        if (requestParameters.networkStatusRequest === null || requestParameters.networkStatusRequest === undefined) {
            throw new runtime.RequiredError('networkStatusRequest','Required parameter requestParameters.networkStatusRequest was null or undefined when calling statusNetworkStatusPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/status/network-status`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: NetworkStatusRequestToJSON(requestParameters.networkStatusRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => NetworkStatusResponseFromJSON(jsonValue));
    }

    /**
     * Returns the current state and status of the node\'s copy of the ledger.
     * Get Network Status
     */
    async statusNetworkStatusPost(requestParameters: StatusNetworkStatusPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<NetworkStatusResponse> {
        const response = await this.statusNetworkStatusPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Get results of test-oriented \"Genesis Scenarios\" executed on this Network.
     * Get Scenarios\' results.
     */
    async statusScenariosPostRaw(requestParameters: StatusScenariosPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<ScenariosResponse>> {
        if (requestParameters.scenariosRequest === null || requestParameters.scenariosRequest === undefined) {
            throw new runtime.RequiredError('scenariosRequest','Required parameter requestParameters.scenariosRequest was null or undefined when calling statusScenariosPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/status/scenarios`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: ScenariosRequestToJSON(requestParameters.scenariosRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => ScenariosResponseFromJSON(jsonValue));
    }

    /**
     * Get results of test-oriented \"Genesis Scenarios\" executed on this Network.
     * Get Scenarios\' results.
     */
    async statusScenariosPost(requestParameters: StatusScenariosPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<ScenariosResponse> {
        const response = await this.statusScenariosPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

}
