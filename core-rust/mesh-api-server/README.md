# Mesh API implementation for Radix

- [Mesh API Homepage](https://docs.cdp.coinbase.com/mesh/docs/welcome)
- [Mesh API Specification](https://github.com/coinbase/mesh-specifications)
- [Mesh API CLI test tool](https://github.com/coinbase/mesh-cli)


# Supported features

| Feature                      | Status                                                                |
| ---------------------------- | ----------------------------------------------------------------------|
| Data API                     | Feature-complete with some quirks                                     |
| - `/network/list`            | Done                                                                  |
| - `/network/status`          | Done                                                                  |
| - `/network/options`         | Done                                                                  |
| - `/block`                   | Feature-complete (exposes only balance-changing operations)           |
| - `/block/transaction`       | Feature-complete (exposes only balance-changing operations)           |
| - `/account/balance`         | Done (historical balances available if explicitly enabled)            |
| - `/mempool`                 | Done (however transaction are not hold there for meaningful time)     |
| - `/mempool/transaction`     | Done (no operation estimation)                                        |
| Construction API             | Done                                                                  |
| - `/construction/derive`     | Done                                                                  |
| - `/construction/preprocess` | Done                                                                  |
| - `/construction/metadata`   | Done                                                                  |
| - `/construction/payloads`   | Done (Withdraw and Deposit operations only)                           |
| - `/construction/combine`    | Done                                                                  |
| - `/construction/parse`      | Done                                                                  |
| - `/construction/hash`       | Done                                                                  |
| - `/construction/submit`     | Done                                                                  |

# Config

```
api.mesh.enabled=<false by default>
api.engine_state.port=<3337 by default>
api.engine_state.bind_address=<127.0.01 by default>
db.historical_substate_values.enable=<false by default>
```

Base url:
```
http://<bind_address>:<port>/mesh
```

Example:
- get account balance
```
http://localhost:3337/mesh/account/balance
```

# Testing

TBD

# Abstractions

## NetworkIdentifier
Fields:
- `blockchanin` - "radix"
- `network` - network variant, eg. `mainnet`, `stokenet`, `localnet`
- `sub_network_identifier` - not set

## Block
  Single transaction

## BlockIdentifier
Fields:
- `index` - state version
- `hash` consists of hexstring of 32 bytes of:
  - transaction_tree_hash bytes[0..12]
  - receipt_tree_hash bytes[0..12]
  - state_version bytes[0..8]

## TransactionIdentifier
- if user transaction<br>
bech32-encoded transaction_intent_hash (txid),
eg. `txid_tdx_2_1nvm90npmkjcltvpy38nr373pt38ctptgg9y0g3wemhtjxyjmau7s32hd0n`
- if non-user transaction<br>
bech32-encoded ledger_transaction_hash,
eg. `ledgertransaction_tdx_2_1s45u3f6xrh4tf4040aqt9fql3wqlhvwwwfpaz4rsru3pr88f3anstnds7s`

## AccountIdentifier
Fields:
- `address` - bech32-encoded Global Entity Address
- `sub_account` - not set
- `metadata` - not set

## Currency
Fields:
- `symbol` - bech32-encoded Resource Address (symbol in resouce address metadata in Radix is mutable, thus cannot be used)
- `decimals` - the resource's divisibility
- `metadata` - not set

## Amount
Fields:
- `value` - amount od the currency
- `currency` - resource information
- `metadata` - not set
## Operation
Fields:
- `operation_identifier` index of the operation within a transaction
- `related_operations` - not set
- `type`<br>
Supported operation types:
  - `Withdraw`<br>
Withdraw some assets from the account. Might be success or failure.
  - `Deposit`<br>
Deposit some assets to the account. Might be success or failure.
  - `FeePayment`<br>
Withdraw some assets from the account to cover transaction fee. Always success, even if transaction failed.
It is not supported at parsing time.
- `status` - status of the operation
- `account` - the amount which transfers the resources
- `amount` - amount of currency transferred
- `coin_change` - not set
- `metadata` - not set
