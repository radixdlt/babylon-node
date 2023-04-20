# Testnet Node

This is the easiest way to get a Babylon fullnode up and running, ready for integration development.
The node connects to the latest public testnet (as of April/May 2023, this is RCnet V1, known by its logical name `kisharnet`).

Documentation for integrators is available [here](https://docs.google.com/document/d/1cjc7_alyzIb2QQIGGn1PEpJyjrMRZYHq3VwkOXRP8J0).

## Node volumes

The node makes use of some persistent data, which is set up to be stored as docker volumes.

### Ledger Database

The ledger database is stored in a docker volume called `ledger-data`.

Sometimes if updating to a new version of the node, a ledger wipe is needed. To wipe the ledger, run: `docker volume rm radixdlt-testnet-babylon-node_ledger-data` to remove the `ledger-data` volume.

### Key Pair

A node in the network is identified by its public key.

A keystore containing a keypair is automatically generated on startup and stored inside a docker volume called `key-data`, protected by default with password `"password"`.


## Getting started
1. Install `docker` version 20.10+ and `docker-compose` version 1.25+
2. Run `./run.sh` in this folder to `docker compose up --build`

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
