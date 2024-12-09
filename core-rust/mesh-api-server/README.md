# Mesh API Implementation for Radix

## Mesh API information

- [Mesh API Homepage](https://docs.cdp.coinbase.com/mesh/docs/welcome)
- [Mesh API Specification](https://github.com/coinbase/mesh-specifications)
- [Mesh API CLI Test Tool](https://github.com/coinbase/mesh-cli)

## Supported Features

| Feature                      | Status                                                                |
| ---------------------------- | ----------------------------------------------------------------------|
| Data API                     | Feature-complete, with some quirks                                    |
| - `/network/list`            | Complete                                                              |
| - `/network/status`          | Complete                                                              |
| - `/network/options`         | Complete                                                              |
| - `/block`                   | Feature-complete (exposes only balance-changing operations)           |
| - `/block/transaction`       | Feature-complete (exposes only balance-changing operations)           |
| - `/account/balance`         | Complete (historical balances available if explicitly enabled)        |
| - `/mempool`                 | Complete (transactions are not held for a meaningful duration)        |
| - `/mempool/transaction`     | Complete (basic operations supported)                                 |
| Construction API             | Complete                                                              |
| - `/construction/derive`     | Complete                                                              |
| - `/construction/preprocess` | Complete                                                              |
| - `/construction/metadata`   | Complete                                                              |
| - `/construction/payloads`   | Complete (supports Withdraw and Deposit operations only)              |
| - `/construction/combine`    | Complete                                                              |
| - `/construction/parse`      | Complete (basic operations supported)                                 |
| - `/construction/hash`       | Complete                                                              |
| - `/construction/submit`     | Complete                                                              |

## Additional Considerations

### Accounts

The current implementation has the following constraints:
- **Supports only account components**: Returns block operations or balances exclusively for accounts. Other components (e.g., smart contracts, lockers) are ignored.
- **Supports only Withdraw, Deposit, and FeePayment operations**: Minting and burning are skipped.

These constraints simplify the implementation without causing Mesh CLI tests to fail. If non-account components must be supported, the following may be required:
- Adding support for Minting and Burning operations.
- Providing explicit support for non-account components in balance fetching (or using `dump_component_state()`, which is resource-intensive).
- Exempting some addresses.

### Operations

Currently, a very specific parser is used to extract operations from given instructions (endpoints: `/mempool/transaction` and `/construction/parse`).
It works only with the instructions constructed by Mesh.
`Withdraw` and `Deposit` are the direct result of the instructions being used, while a `FeePayment` is added at commit time.

Technically, it would be possible to use transaction previews, receipts, and balance change summaries to extract operations.
But we don't do it for following reasons:
- both endpoint methods should wotrk offline
- both endpoint methods should be static (not affected by current state of the network)
- this approach is deemed too resource-heavy

## Configuration

### Server settings
There are 3 settings to configure Mesh API server, which allow to:
- enable/disable Mesh API server launch (disabled by default),
- override the default port (3337),
- override the default bind address (127.0.0.1).

#### node running bare-metal
```plaintext
api.mesh.enabled=<true/false>
api.mesh.port=<port number>
api.mesh.bind_address=<ip address>
```
#### node-running in a Docker
Set below environmental variables

```plaintext
RADIXDLT_MESH_API_ENABLED=<true/false>
RADIXDLT_MESH_API_PORT=<port number>
RADIXDLT_MESH_API_BIND_ADDRESS=<ip address>
```

### Enable historical balances for reconciliation tests
In order to proceed with reconciliation tests historical balances shall be enabled.
There are 2 useful settings:
- enable/disable historical substate values (disabled by default),
- adjust the state version history length to keep (60000 by default).

#### node running bare-metal
`state_version_history_length` controls how much history is kept in historical_substate_balues
```plaintext
db.historical_substate_values.enable=<true/false>
state_hash_tree.state_version_history_length=<history_length_to_keep>
```

#### node-running in a Docker
```
RADIXDLT_DB_HISTORICAL_SUBSTATE_VALUES_ENABLE=<true/false>
RADIXDLT_STATE_HASH_TREE_STATE_VERSION_HISTORY_LENGTH=<history_length_to_keep>
```

### Base URL

```plaintext
http://<bind_address>:<port>/mesh
```

**Example**: Fetching account balance
```plaintext
http://localhost:3337/mesh/account/balance
```

## Testing

### Mesh CLI

#### Steps:
1. [Terminal 1] Download the `rosetta-cli` prebuilt binary:
    ```bash
    curl -sSfL https://raw.githubusercontent.com/coinbase/mesh-cli/master/scripts/install.sh | sh -s

    alias mesh-cli='./bin/rosetta-cli --configuration-file <root-of-babylon-node-repo>/core-rust/mesh-api-server/mesh-cli-configs/localnet.json'
    ```
    **Note:** As of November 2024, there are issues with the prebuilt MacOS binary. Use the workaround below:
    ```bash
    git clone git@github.com:coinbase/mesh-cli
    cd mesh-cli
    git checkout bbbd759
    alias mesh-cli='go run main.go --configuration-file <root-of-babylon-node-repo>/core-rust/mesh-api-server/mesh-cli-configs/localnet.json'
    ```

2. [Terminal 2] Launch the node:
    ```bash
    cd <root-of-babylon-node-repo>
    RADIXDLT_NODE_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY= ./gradlew :core:run --info
    ```

3. [Terminal 1] Run Mesh API tests:
    ```bash
    mesh-cli check:data
    mesh-cli check:construction
    mesh-cli check:spec
    ```

#### Reconciliation Tests

- **Historical Balances:** Enable historical balances before launching the node:
    ```plaintext
    db.historical_substate_values.enable=true
    ```
    Or, use the following environment variable:
    ```bash
    RADIXDLT_DB_HISTORICAL_SUBSTATE_VALUES_ENABLE=1
    ```

- **Whole Ledger Reconciliation:** If reconciling the entire ledger for a network (e.g., `stokenet`):
  - Set a future `state_version` in the `data.end_conditions.index` field of the `mesh-cli` config file.
  - Launch the node with an empty database.
  - Start `mesh-cli` as soon as possible to avoid pruning historical balances.

### Unit Tests

- **Java:**
    ```bash
    ./gradlew :core:test --tests '*MeshApiMempoolEndpointsTest*'
    ```
- **Rust:** (To Be Determined)

## Abstractions

### NetworkIdentifier

Fields:
- `blockchain`: "radix"
- `network`: Network variant (e.g., `mainnet`, `stokenet`, `localnet`).
- `sub_network_identifier`: Not set.

### Block

Represents a single transaction.

### BlockIdentifier

Fields:
- `index`: State version.
- `hash`: Hex-encoded string of 32 bytes composed of:
  - `transaction_tree_hash` (bytes[0..12]).
  - `receipt_tree_hash` (bytes[0..12]).
  - `state_version` (bytes[0..8]).

### TransactionIdentifier

- **User transaction:** Bech32-encoded `transaction_intent_hash` (e.g., `txid_tdx_2_1nvm90npmkjcltvpy38nr373pt38ctptgg9y0g3wemhtjxyjmau7s32hd0n`).
- **Non-user transaction:** Bech32-encoded `ledger_transaction_hash` (e.g., `ledgertransaction_tdx_2_1s45u3f6xrh4tf4040aqt9fql3wqlhvwwwfpaz4rsru3pr88f3anstnds7s`).

### AccountIdentifier

Fields:
- `address`: Bech32-encoded Global Entity Address.
- `sub_account`: Not set.
- `metadata`: Not set.

### Currency

Fields:
- `symbol`: Bech32-encoded Resource Address.
- `decimals`: Resource divisibility.
- `metadata`: Not set.

### Amount

Fields:
- `value`: Currency amount.
- `currency`: Resource information.
- `metadata`: Not set.

### Operation

Fields:
- `operation_identifier`: Index of the operation within a transaction.
- `related_operations`: Not set.
- `type`:
  - `Withdraw`: Withdraw assets from an account (always success, failed operations are filtered out).
  - `Deposit`: Deposit assets to an account (always success, failed operations are filtered out).
  - `FeePayment`: Withdraw assets to cover transaction fees (always success, even if the transaction fails).
- `status`: Operation status.
- `account`: Account transferring the resources.
- `amount`: Amount of currency transferred.
- `coin_change`: Not set.
- `metadata`: Not set.

