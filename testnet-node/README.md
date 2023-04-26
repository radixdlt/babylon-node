# Testnet Node

This is the easiest way to get a Babylon fullnode up and running, for integration development.

This set-up is only intended for local running as a developer - it is not intended for running in production - see [our documentation site for information on running a production node](https://docs-babylon.radixdlt.com/main/node-and-gateway/node-setup-introduction.html).

The node connects to the latest public testnet (as of April/May 2023, this is RCnet-V1, known by its logical name `kisharnet`).

Documentation for integrators is available [here](https://docs.google.com/document/d/1cjc7_alyzIb2QQIGGn1PEpJyjrMRZYHq3VwkOXRP8J0).

## Getting started
1. Install `docker` version 20.10+
2. Run `./run.sh` in this folder, or run `docker compose up --build`.
   * NOTE: This uses [the recommended `docker compose` command](https://docs.docker.com/compose/compose-v2/), rather than the legacy `docker-compose`. If you see an error such as `compose is not a command`, please ensure you are running docker 20.10 or above.
3. The node will start syncing. You can see how close to synced-up you are by running this query and examining the `ledger_clock.date_time` field:
```sh
curl \
  'http://localhost:3333/core/lts/transaction/construction' \
  -H 'Content-Type: application/json' \
  -d '{ "network": "kisharnet" }'
```

## Node volumes

The node makes use of some persistent data, which is set up to be stored as docker volumes.

### Ledger database

The ledger database is stored in a docker volume called `ledger-data`.

Sometimes if updating to a new version of the node, a ledger wipe is needed. To wipe the ledger, run: `docker volume rm radixdlt-testnet-babylon-node_ledger-data` to remove the `ledger-data` volume.

### Key pair

A node in the network is identified by its public key.

A keystore containing a keypair is automatically generated on startup and stored inside a docker volume called `key-data`, protected by default with password `"password"`.

## Local builds
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

And then `./run.sh` in this folder.
