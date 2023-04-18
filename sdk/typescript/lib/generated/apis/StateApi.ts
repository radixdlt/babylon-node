/* tslint:disable */
/* eslint-disable */
/**
 * Babylon Core API - RCnet V1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.3.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


import * as runtime from '../runtime';
import type {
  BasicErrorResponse,
  StateAccessControllerRequest,
  StateAccessControllerResponse,
  StateClockRequest,
  StateClockResponse,
  StateComponentRequest,
  StateComponentResponse,
  StateEpochRequest,
  StateEpochResponse,
  StateNonFungibleRequest,
  StateNonFungibleResponse,
  StatePackageRequest,
  StatePackageResponse,
  StateResourceRequest,
  StateResourceResponse,
  StateValidatorRequest,
  StateValidatorResponse,
} from '../models';
import {
    BasicErrorResponseFromJSON,
    BasicErrorResponseToJSON,
    StateAccessControllerRequestFromJSON,
    StateAccessControllerRequestToJSON,
    StateAccessControllerResponseFromJSON,
    StateAccessControllerResponseToJSON,
    StateClockRequestFromJSON,
    StateClockRequestToJSON,
    StateClockResponseFromJSON,
    StateClockResponseToJSON,
    StateComponentRequestFromJSON,
    StateComponentRequestToJSON,
    StateComponentResponseFromJSON,
    StateComponentResponseToJSON,
    StateEpochRequestFromJSON,
    StateEpochRequestToJSON,
    StateEpochResponseFromJSON,
    StateEpochResponseToJSON,
    StateNonFungibleRequestFromJSON,
    StateNonFungibleRequestToJSON,
    StateNonFungibleResponseFromJSON,
    StateNonFungibleResponseToJSON,
    StatePackageRequestFromJSON,
    StatePackageRequestToJSON,
    StatePackageResponseFromJSON,
    StatePackageResponseToJSON,
    StateResourceRequestFromJSON,
    StateResourceRequestToJSON,
    StateResourceResponseFromJSON,
    StateResourceResponseToJSON,
    StateValidatorRequestFromJSON,
    StateValidatorRequestToJSON,
    StateValidatorResponseFromJSON,
    StateValidatorResponseToJSON,
} from '../models';

export interface StateAccessControllerPostRequest {
    stateAccessControllerRequest: StateAccessControllerRequest;
}

export interface StateClockPostRequest {
    stateClockRequest: StateClockRequest;
}

export interface StateComponentPostRequest {
    stateComponentRequest: StateComponentRequest;
}

export interface StateEpochPostRequest {
    stateEpochRequest: StateEpochRequest;
}

export interface StateNonFungiblePostRequest {
    stateNonFungibleRequest: StateNonFungibleRequest;
}

export interface StatePackagePostRequest {
    statePackageRequest: StatePackageRequest;
}

export interface StateResourcePostRequest {
    stateResourceRequest: StateResourceRequest;
}

export interface StateValidatorPostRequest {
    stateValidatorRequest: StateValidatorRequest;
}

/**
 * 
 */
export class StateApi extends runtime.BaseAPI {

    /**
     * Reads the access controller\'s substate/s from the top of the current ledger. 
     * Get Access Controller Details
     */
    async stateAccessControllerPostRaw(requestParameters: StateAccessControllerPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StateAccessControllerResponse>> {
        if (requestParameters.stateAccessControllerRequest === null || requestParameters.stateAccessControllerRequest === undefined) {
            throw new runtime.RequiredError('stateAccessControllerRequest','Required parameter requestParameters.stateAccessControllerRequest was null or undefined when calling stateAccessControllerPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/state/access-controller`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StateAccessControllerRequestToJSON(requestParameters.stateAccessControllerRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StateAccessControllerResponseFromJSON(jsonValue));
    }

    /**
     * Reads the access controller\'s substate/s from the top of the current ledger. 
     * Get Access Controller Details
     */
    async stateAccessControllerPost(requestParameters: StateAccessControllerPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StateAccessControllerResponse> {
        const response = await this.stateAccessControllerPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Reads the clock\'s substate/s from the top of the current ledger. 
     * Get Clock Details
     */
    async stateClockPostRaw(requestParameters: StateClockPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StateClockResponse>> {
        if (requestParameters.stateClockRequest === null || requestParameters.stateClockRequest === undefined) {
            throw new runtime.RequiredError('stateClockRequest','Required parameter requestParameters.stateClockRequest was null or undefined when calling stateClockPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/state/clock`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StateClockRequestToJSON(requestParameters.stateClockRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StateClockResponseFromJSON(jsonValue));
    }

    /**
     * Reads the clock\'s substate/s from the top of the current ledger. 
     * Get Clock Details
     */
    async stateClockPost(requestParameters: StateClockPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StateClockResponse> {
        const response = await this.stateClockPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Reads the component\'s substate/s from the top of the current ledger. Also recursively extracts vault balance totals from the component\'s entity subtree. 
     * Get Component Details
     */
    async stateComponentPostRaw(requestParameters: StateComponentPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StateComponentResponse>> {
        if (requestParameters.stateComponentRequest === null || requestParameters.stateComponentRequest === undefined) {
            throw new runtime.RequiredError('stateComponentRequest','Required parameter requestParameters.stateComponentRequest was null or undefined when calling stateComponentPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/state/component`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StateComponentRequestToJSON(requestParameters.stateComponentRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StateComponentResponseFromJSON(jsonValue));
    }

    /**
     * Reads the component\'s substate/s from the top of the current ledger. Also recursively extracts vault balance totals from the component\'s entity subtree. 
     * Get Component Details
     */
    async stateComponentPost(requestParameters: StateComponentPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StateComponentResponse> {
        const response = await this.stateComponentPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Reads the epoch manager\'s substate/s from the top of the current ledger. 
     * Get Epoch Details
     */
    async stateEpochPostRaw(requestParameters: StateEpochPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StateEpochResponse>> {
        if (requestParameters.stateEpochRequest === null || requestParameters.stateEpochRequest === undefined) {
            throw new runtime.RequiredError('stateEpochRequest','Required parameter requestParameters.stateEpochRequest was null or undefined when calling stateEpochPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/state/epoch`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StateEpochRequestToJSON(requestParameters.stateEpochRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StateEpochResponseFromJSON(jsonValue));
    }

    /**
     * Reads the epoch manager\'s substate/s from the top of the current ledger. 
     * Get Epoch Details
     */
    async stateEpochPost(requestParameters: StateEpochPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StateEpochResponse> {
        const response = await this.stateEpochPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Reads the data associated with a single Non-Fungible Unit under a Non-Fungible Resource. 
     * Get Non-Fungible Details
     */
    async stateNonFungiblePostRaw(requestParameters: StateNonFungiblePostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StateNonFungibleResponse>> {
        if (requestParameters.stateNonFungibleRequest === null || requestParameters.stateNonFungibleRequest === undefined) {
            throw new runtime.RequiredError('stateNonFungibleRequest','Required parameter requestParameters.stateNonFungibleRequest was null or undefined when calling stateNonFungiblePost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/state/non-fungible`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StateNonFungibleRequestToJSON(requestParameters.stateNonFungibleRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StateNonFungibleResponseFromJSON(jsonValue));
    }

    /**
     * Reads the data associated with a single Non-Fungible Unit under a Non-Fungible Resource. 
     * Get Non-Fungible Details
     */
    async stateNonFungiblePost(requestParameters: StateNonFungiblePostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StateNonFungibleResponse> {
        const response = await this.stateNonFungiblePostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Reads the package\'s substate/s from the top of the current ledger. 
     * Get Package Details
     */
    async statePackagePostRaw(requestParameters: StatePackagePostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StatePackageResponse>> {
        if (requestParameters.statePackageRequest === null || requestParameters.statePackageRequest === undefined) {
            throw new runtime.RequiredError('statePackageRequest','Required parameter requestParameters.statePackageRequest was null or undefined when calling statePackagePost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/state/package`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StatePackageRequestToJSON(requestParameters.statePackageRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StatePackageResponseFromJSON(jsonValue));
    }

    /**
     * Reads the package\'s substate/s from the top of the current ledger. 
     * Get Package Details
     */
    async statePackagePost(requestParameters: StatePackagePostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StatePackageResponse> {
        const response = await this.statePackagePostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Reads the resource manager\'s substate/s from the top of the current ledger. 
     * Get Resource Details
     */
    async stateResourcePostRaw(requestParameters: StateResourcePostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StateResourceResponse>> {
        if (requestParameters.stateResourceRequest === null || requestParameters.stateResourceRequest === undefined) {
            throw new runtime.RequiredError('stateResourceRequest','Required parameter requestParameters.stateResourceRequest was null or undefined when calling stateResourcePost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/state/resource`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StateResourceRequestToJSON(requestParameters.stateResourceRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StateResourceResponseFromJSON(jsonValue));
    }

    /**
     * Reads the resource manager\'s substate/s from the top of the current ledger. 
     * Get Resource Details
     */
    async stateResourcePost(requestParameters: StateResourcePostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StateResourceResponse> {
        const response = await this.stateResourcePostRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Reads the validator\'s substate/s from the top of the current ledger. 
     * Get Validator Details
     */
    async stateValidatorPostRaw(requestParameters: StateValidatorPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<StateValidatorResponse>> {
        if (requestParameters.stateValidatorRequest === null || requestParameters.stateValidatorRequest === undefined) {
            throw new runtime.RequiredError('stateValidatorRequest','Required parameter requestParameters.stateValidatorRequest was null or undefined when calling stateValidatorPost.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/state/validator`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: StateValidatorRequestToJSON(requestParameters.stateValidatorRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => StateValidatorResponseFromJSON(jsonValue));
    }

    /**
     * Reads the validator\'s substate/s from the top of the current ledger. 
     * Get Validator Details
     */
    async stateValidatorPost(requestParameters: StateValidatorPostRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<StateValidatorResponse> {
        const response = await this.stateValidatorPostRaw(requestParameters, initOverrides);
        return await response.value();
    }

}
