import fetch from "node-fetch";
import http from 'node:http';

import { CoreApiClient, LtsCommittedTransactionStatus, LtsTransactionIntentStatus, ResponseError } from "../lib";

// These tests are assumed to run against a running local node, with Core API bound to 3333.

async function newCoreApiClient(): Promise<CoreApiClient> {
    return await CoreApiClient.initialize({
        // Note - in nodeJS, you may need to use 127.0.0.1 instead of localhost
        basePath: "http://127.0.0.1:3333/core",
        logicalNetworkName: "localnet",
        // Configuration for node-fetch
        fetch,
        advanced: {
            agent: new http.Agent({ keepAlive: true })
        }
    });
}

interface Constructable<T> {
    new (...args: any[]): T;
}

async function expectError<T>(promise: Promise<unknown>, errorType: Constructable<T>): Promise<T> {
    try {
        const result = await promise;
        throw new Error("Expected error, but got success", { cause: result });
    } catch (e) {
        if (e instanceof errorType) {
            return e;
        }
        throw new Error(`Error was not of type ${errorType.name}, but was: ${e}`, { cause: e });
    }
}

/**
 * A lot of these test error cases, because they're easier and I just want to test that the
 * query syntax and that the query connects to the API.
 */

test('can get construction metadata', async () => {
    const coreApiClient = await newCoreApiClient();
    let response = await coreApiClient.LTS.getConstructionMetadata();
    expect(response.ledger_clock.unix_timestamp_ms).toBeGreaterThanOrEqual(Date.now() - 60 * 1000);
});

test('can get account deposit behaviour', async () => {
    const coreApiClient = await newCoreApiClient();
    const xrdResource = "resource_loc1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxvq32hv";
    let response = await coreApiClient.LTS.getAccountDepositBehaviour(
        "account_loc168yqmxxkzmrzv9knya9865me9qktk43rqfuwdy9fq9rlk7a682gzva", [xrdResource]
    );
    expect(response.default_deposit_rule).toBe("Accept");
    let xrdResponse = response.resource_specific_behaviours![xrdResource];
    expect(xrdResponse.is_xrd).toBe(true);
    expect(xrdResponse.allows_try_deposit).toBe(true);
});

test('submit unparsable transaction errors', async () => {
    const coreApiClient = await newCoreApiClient();
    const invalidTransactionPayload = "00";
    const response = await coreApiClient.LTS.submitTransaction({
        notarized_transaction_hex: invalidTransactionPayload
    });
    if (response.result !== "Rejected") {
        throw new Error(`Expected Error result, got: ${response}`);
    }
    expect(response.details.error_message).toContain("PrepareError");
});

test('get transaction status of an placeholder transaction returns NotSeen', async () => {
    const coreApiClient = await newCoreApiClient();
    let response = await coreApiClient.LTS.getTransactionStatus({
        intent_hash: "0000000000000000000000000000000000000000000000000000000000000000",
    });
    expect(response.intent_status).toBe(LtsTransactionIntentStatus.NotSeen);
});

test('get transaction status of an invalid status returns message', async () => {
    const coreApiClient = await newCoreApiClient();
    const responsePromise = coreApiClient.LTS.getTransactionStatus({
        intent_hash: "INVALID"
    });
    const error = await expectError(responsePromise, ResponseError);
    expect(error.status).toBe(400);
    expect(error.message).toContain("InvalidHash");
});

test('can get genesis transaction outcome', async () => {
    const coreApiClient = await newCoreApiClient();
    const response = await coreApiClient.LTS.getTransactionOutcome({
        state_version: 1
    });
    expect(response!.status).toBe(LtsCommittedTransactionStatus.Success);
});

test('really high transaction is undefined', async () => {
    const coreApiClient = await newCoreApiClient();
    const response = await coreApiClient.LTS.getTransactionOutcome({
        state_version: 10000000000000
    });
    expect(response).toBeUndefined();
});

test('getAccountFungibleResourceBalance with invalid address returns invalid address', async () => {
    const coreApiClient = await newCoreApiClient();
    let responsePromise = coreApiClient.LTS.getAccountFungibleResourceBalance({
        account_address: "account_invalid",
        resource_address: "resource_invalid",
    });
    const error = await expectError(responsePromise, ResponseError);
    expect(error.status).toBe(400);
    expect(error.message).toContain("InvalidAddress");
});

test('getAccountAllFungibleResourceBalances with invalid address returns invalid address', async () => {
    const coreApiClient = await newCoreApiClient();
    let responsePromise = coreApiClient.LTS.getAccountAllFungibleResourceBalances({
        account_address: "account_invalid",
    });
    const error = await expectError(responsePromise, ResponseError);
    expect(error.status).toBe(400);
    expect(error.message).toContain("InvalidAddress");
});

test('getTransactionOutcomes returns early transactions', async () => {
    const coreApiClient = await newCoreApiClient();
    let response = await coreApiClient.LTS.getTransactionOutcomes({
        from_state_version: 1,
        limit: 100,
    });
    expect(response.committed_transaction_outcomes.length).toBe(100);
});

test('getTransactionOutcomes to not return transactions off the end of the ledger', async () => {
    const coreApiClient = await newCoreApiClient();
    let response = await coreApiClient.LTS.getTransactionOutcomes({
        from_state_version: 1000000000000,
        limit: 5,
    });
    expect(response.committed_transaction_outcomes.length).toBe(0);
});

test('getAccountTransactionOutcomes with invalid address to return error', async () => {
    const coreApiClient = await newCoreApiClient();
    let responsePromise = coreApiClient.LTS.getAccountTransactionOutcomes({
        account_address: "account_invalid",
        from_state_version: 1000000000000,
        limit: 5,
    });
    const error = await expectError(responsePromise, ResponseError);
    expect(error.status).toBe(400);
    expect(error.message).toContain("InvalidAddress");
});
