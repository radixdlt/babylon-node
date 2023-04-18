## Testnet Node

This is the easiest way to get a Babylon fullnode up and running, ready for integration development.
A node in the network is identified by it's public key and there can be multiple addresses associated to a public key.
The protocol only guarantees that a message ment for a public key will arrive to at least one (not all) of the addresses.
This means you need to generate a new key (and keep it to yourself). Luckly you don't have do anything as long as env
variable `RADIXDLT_NODE_KEY_CREATE_IF_MISSING` is set to `true`. The only takeaway is that by default, the private key
is generated and stored inside of docker volume `key-data` (please double check before removing this volume), keystore encrypted
with the password provided via `RADIX_NODE_KEYSTORE_PASSWORD`.
The initial syncing of the whole ledger state can take a while. To avoid having to go through this every time
something is changed in `docker-compose.yml`, we setup `ledger-data` volume for persistance.
Sometimes a ledger wipe is needed, in which case you can run: `docker volume rm testnet-node_ledger-data` to remove (only) `ledger-data` volume.

### Getting started
1. Install `docker` and `docker-compose`
2. Run `docker-compose up` in this folder

### Local builds
If you decide you want to run using locally built images (i.e. you want to add & test your own custom endpoint) change
```YML
        image: radixdlt/babylon-node:<tag>
```
with
```YML
        build:
            context: ..
            dockerfile: Dockerfile
```
And run using `docker-compose up --build`.
