# Core API SDK

This SDK is a thin wrapper around the [Babylon Core API](https://docs-babylon.radixdlt.com/main/apis/api-specification.html).

The `CoreApiClient` is the main exported object. It currently includes high-level wrappers around two sub-APIs: `LTS` and `Status`.

For querying other sub-APIs, for now, use methods on `coreApiClient.LowLevel.X` where `X` is each of the different sub-APIs.

## Client creation

Behind the scenes, this library uses the fetch API:
* If in an environment where `fetch` is not available, a polyfill must be used (see eg [node-fetch](https://www.npmjs.com/package/node-fetch)).
* If in a browser, pass `window.fetch` into the `fetch` parameter.

The client checks that it can connect to the Core API at `initialize` time. If you'd rather, you can use `initializeUnchecked` which skips this check.

```typescript
import fetch from "node-fetch" // Optional polyfill for fetch required if running in nodeJS
import { CoreApiClient} from "@radixdlt/babylon-core-api-sdk";

const coreApiClient = await CoreApiClient.initialize({
    basePath: "http://localhost:3333/core",
    fetch,
    logicalNetworkName: "kisharnet",
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