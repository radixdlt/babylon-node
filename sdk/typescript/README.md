# Core API SDK

This SDK is a thin wrapper around the [Babylon Core API](https://docs-babylon.radixdlt.com/main/apis/api-specification.html).

The `CoreApiClient` is the main exported object. It currently includes high-level wrappers around two sub-APIs: `LTS` and `Status`.

For querying other sub-APIs, for now, use methods on `coreApiClient.LowLevel.X` where `X` is each of the different sub-APIs.

## End-to-end examples

See [here for a full end-to-end example working with the TypeScript Radix Engine Toolkit](https://github.com/radixdlt/typescript-radix-engine-toolkit/tree/main/examples/core-e2e-example).

## Client creation

You can instantiate a Core API Client with `CoreApiClient.initialize({ ... })` and various configuration options.
The options you provide will depend on how your node is configured, and how you're connecting to it.

The client checks that it can connect to the Core API at `initialize` time.

If running against RCnet-V1, with a full node running on localhost (such as running `testnet-node/run.sh` in local development), with its Core API bound to 3333, you can use:

```typescript
import { CoreApiClient } from "@radixdlt/babylon-core-api-sdk";

const coreApiClient = await CoreApiClient.initialize({
    basePath: "http://127.0.0.1:3333/core", // Note: 127.0.0.1 works better than localhost on Node.JS
    logicalNetworkName: "kisharnet",
    // Further options - explained below...
});
```

If this doesn't work, please see further options below.

### Providing the Fetch API

Behind the scenes, this library uses the fetch API:
* In Node.JS v18+ or in a browser, `fetch` is provided natively, so you don't need add a `fetch` parameter.
* If earlier Node.JS versions, a stand-in must be used (see eg [node-fetch](https://www.npmjs.com/package/node-fetch)).
  We suggest adding the following dependencies:
  * `node-fetch` version `^2.7.3` (as `3.x` can only be imported as an ESModule).
  * `@types/node-fetch` version `^2.6.3`

Note - if using `node-fetch` with Node.JS 20+, there is an [outstanding bug](https://github.com/node-fetch/node-fetch/issues/1735) which you can fix by explicitly turning on `keepAlive` in a http/https agent.

```typescript
import fetch from "node-fetch"
import http from "node:http";
import https from "node:https";
import { CoreApiClient } from "@radixdlt/babylon-core-api-sdk";

const coreApiClient = await CoreApiClient.initialize({
    // ...
    fetch,
    advanced: {
        // Depending on the scheme of the URL (http/https), you will either need to select a `http.Agent` or a `https.Agent`
        // EITHER:
        agent: new http.Agent({ keepAlive: true }),
        // OR
        agent: new https.Agent({ keepAlive: true }),
    }
});
```

### Connecting to a node set up with the Node CLI / nginx

If you have set up your node using the Radix Node CLI, then the node will likely be configured with its APIs exposed via nginx, and bound to the standard https port.

You will have set up some custom basic authentication for this API, and by default, nginx will use a self-signed certificate.

To work with this set-up you will need to:
* Ensure your `basePath` is set up to connect to the correct host and port, on which nginx is bound.
* Set the `Authorization` header on your request, configured with your basic auth credentials for the Core API.
* In the `agent` / `dispatcher` configuration, include the self-signed certificate by using the `ca` parameter, or if you understand the implications and have precautions against MITM, use `rejectUnauthorized: false` with the https agent.

#### With `node-fetch`

An example, with the `rejectUnauthorized: false` line, assuming you are EG on localhost, or a private network where MITM attacks are mitigated.

```typescript
import https from "node:https";
import fetch from "node-fetch"
import https from "node:https";
import { CoreApiClient } from "@radixdlt/babylon-core-api-sdk";

const basicAuthUsername = "admin";
const basicAuthPassword = "????"; // From your node set-up - provide this securely to your application

const coreApiClient = await CoreApiClient.initialize({
    basePath: "https://127.0.0.1/core",
    logicalNetworkName: "kisharnet",
    advanced: {
        agent: new https.Agent({
            keepAlive: true,
            // NOTE - Only add the below line if you've taken precautions to avoid MITM attacks between you and the node
            rejectUnauthorized: false,
        }),
        headers: {
            "Authorization": `Basic ${Buffer.from(`${basicAuthUsername}:${basicAuthPassword}`).toString("base64")}`
        }
    }
});
```

#### With native Node.JS `fetch`

If wanting to customise the certificate settings on the request, you will need install `undici` (as per [this comment](https://github.com/nodejs/undici/issues/1489#issuecomment-1543856261)):

The below includes the `rejectUnauthorized: false` line, assuming you are EG on localhost, or a private network where MITM attacks are mitigated.

```typescript
import { CoreApiClient } from "@radixdlt/babylon-core-api-sdk";
import { Agent } from 'undici';

const basicAuthUsername = "admin";
const basicAuthPassword = "????"; // From your node set-up - provide this securely to your application

const coreApiClient = await CoreApiClient.initialize({
    basePath: "https://127.0.0.1/core",
    logicalNetworkName: "kisharnet",
    advanced: {
        dispatcher: new Agent({
            connect: {
                // NOTE - Only add the below line if you've taken precautions to avoid MITM attacks between you and the node
                rejectUnauthorized: false,
            },
        }),
        headers: {
            "Authorization": `Basic ${Buffer.from(`${basicAuthUsername}:${basicAuthPassword}`).toString("base64")}`
        }
    }
});
```

## LTS - High Level API

The Core API has a "Long Term Support" sub-API. This is documented in the [API specification](https://docs-babylon.radixdlt.com/main/apis/api-specification.html).

The client has a high-level `LTS` API, which includes the following features:
* A simpler API, for example, the network name is included on the request for you
* `getConstructionMetadata` includes a node synced-up check
* `submitTransaction` handles different submit transaction errors

### LTS Transaction Construction and Submission

Construction and submission looks something like this:

```typescript
import { LtsCommittedTransactionStatus } from "@radixdlt/babylon-core-api-sdk";

const constructionMetadata = await coreApiClient.LTS.getConstructionMetadata();

const currentEpoch = constructionMetadata.current_epoch;
const { notarizedTransactionHex, intentHash } = buildTransaction(currentEpoch, ...);

/**
 * By testing `submitResponse.result`, you can distinguish different possible results
 * Some of these possible results will need to be handled.
 * See the integrators guide for more information.
 */
const submitResponse = await coreApiClient.LTS.submitTransaction({
    notarized_transaction_hex: notarizedTransactionHex
});
console.log(submitResponse);

/**
 * By testing `statusResponse.intent_status`, you can see different intent statuses.
 *
 * You may wish to poll this endpoint until it is one of:
 * - LtsTransactionIntentStatus.CommittedSuccess
 * - LtsTransactionIntentStatus.CommittedFailure
 * - LtsTransactionIntentStatus.PermanentRejection
 * 
 * See the integrators guide for more information.
 */
const statusResponse = await coreApiClient.LTS.getTransactionStatus({
    intent_hash: intentHash
});
console.log(statusResponse);

if (statusResponse.committed_state_version) {
    const committedOutcomeResponse = await coreApiClient.LTS.getTransactionOutcome({
        state_version: statusResponse.committed_state_version
    });

    if (committedOutcomeResponse?.status == LtsCommittedTransactionStatus.Success) {
        console.log("Committed success");
    } else if (committedOutcomeResponse?.status == LtsCommittedTransactionStatus.Failure) {
        console.log("Committed failure");
    } 
}
```

### LTS - Account balances

Account balance queries look like this:

```typescript
const accountAddress = "..";
const resourceAddress = "..";

const balanceResponse = await coreApiClient.LTS.getAccountFungibleResourceBalance({
    account_address: accountAddress,
    resource_address: resourceAddress,
});

const allBalancesResponse = await coreApiClient.LTS.getAccountAllFungibleResourceBalances({
    account_address: accountAddress,
}); 
```

### LTS - Transaction history

You can get transaction outcomes in ledger order with these endpoints.

You can sum the `fungible_entity_balance_changes` from the transaction outcomes to track balance changes in components including accounts.

```typescript
const accountAddress = "..";

const transactionsBatch = await coreApiClient.LTS.getTransactionOutcomes({
    from_state_version: 1,
    limit: 100,
});

const accountTransactionsBatch = await coreApiClient.LTS.getAccountTransactionOutcomes({
    account_address: accountAddress,
    from_state_version: 1,
    limit: 100,
}); 
```
