# Testnet Node

This is the easiest way to get a Babylon fullnode up and running, for integration development.

This set-up is only intended for local running as a developer - it is not intended for running in production - see [our documentation site for information on running a production node](https://docs-babylon.radixdlt.com/main/node-and-gateway/node-setup-introduction.html).

The node connects to the latest public testnet (as of April/May 2023, this is RCnet-V1, known by its logical name `kisharnet`).

Documentation for integrators is available [here](https://docs.google.com/document/d/1cjc7_alyzIb2QQIGGn1PEpJyjrMRZYHq3VwkOXRP8J0).

## Getting started
1. Install `docker` version 20.10+ with `docker compose` - either by installing docker desktop, or by installing [plain docker and the compose CLI extension](https://docs.docker.com/compose/install/).
2. Ensure `docker compose` runs successfully, printing out the docs for docker compose. If you see an error such as `compose is not a command`, please ensure you are running docker with the compose extension as above. (NOTE: We are using [the future-proof `docker compose` command](https://docs.docker.com/compose/compose-v2/), rather than the legacy `docker-compose`).
3. Ensure your docker daemon is configured with high enough limits, and you have sufficient disk space. We recommend updating the docker limits to at least 2 CPUs, 4GB RAM and 100GB disk size. In Docker Desktop, you can configure this by navigating to `Preferences > Resources`.
4. Run `./run.sh` in this folder, or run `docker compose up --build`.
5. The node will start syncing. You can see how close to synced-up you are by running this query and examining the `ledger_clock.date_time` field:
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

## Debugging

It might happen that you stumble across: 
```
com.sleepycat.je.DiskLimitException: (JE 18.3.12) Disk usage is not within je.maxDisk or je.freeDisk limits and write operations are prohibited: maxDiskLimit=0 freeDiskLimit=5,368,709,120 adjustedMaxDiskLimit=0 maxDiskOverage=0 freeDiskShortage=28,782,592 diskFreeSpace=5,339,926,528 availableLogSize=-28,782,592 totalLogSize=1,915,298 activeLogSize=1,915,298 reservedLogSize=0 protectedLogSize=0 protectedLogSizeMap={}
```

This means you have (almost) reached the virtual disk memory limit of docker. You simply need to increase the limit. 
