## Testnet Node

This is the easiest way to get a Babylon fullnode up and running, ready for integration development.
The node connects to the latest public testnet (as of April/May 2023, this is RCnet V1, known by its logical name kisharnet).
A node in the network is identified by its public key. A keystore containing a keypair is automatically generated on 
startup and stored inside a docker volume called `key-data`, protected by default with password `"password"`.
The initial syncing of the whole ledger state can take a while. To avoid having to go through this every time
something is changed in `docker-compose.yml`, we set up `ledger-data` volume for persistence.
Sometimes a ledger wipe is needed, in which case you can run: `docker volume rm testnet-node_ledger-data` to remove 
(only) `ledger-data` volume.

Documentation for integrators is available [here](https://docs.google.com/document/d/1cjc7_alyzIb2QQIGGn1PEpJyjrMRZYHq3VwkOXRP8J0).

### Getting started
1. Install `docker` and `docker-compose`
2. Run `docker-compose up` in this folder

Tested with:
- `Docker` version 20.10+
- `docker-compose` version 1.25+

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
