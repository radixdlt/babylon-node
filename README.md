[![Docker](https://github.com/radixdlt/babylon-node/actions/workflows/docker.yml/badge.svg?branch=develop)](https://github.com/radixdlt/babylon-node/actions/workflows/docker.yml)
# Radix Babylon Node

This is the repository for the RadixDLT node, for the Babylon release and beyond.

## License

The Babylon node code is released under the [Radix License](LICENSE). Executable components are licensed under the [Radix Node EULA](http://www.radixdlt.com/terms/nodeEULA).

## Integrators

To run a node against our latest test environment, install docker and run `./testnet-node/run.sh`. See further [details on running a testnet node for development here](testnet-node/README.md).

Also check out the [Babylon documentation for exchanges and integrators](https://docs.google.com/document/d/1cjc7_alyzIb2QQIGGn1PEpJyjrMRZYHq3VwkOXRP8J0).

## Subdirectories

Here we have:

- [core](core): The core node, consensus and networking modules - written in Java. It includes a variant implementation of the [HotStuff](https://arxiv.org/abs/1803.05069) BFT-style consensus.
- [core-rust-bridge](core-rust-bridge): A bridge between the Java core and the Rust `core-rust` - written in Java.
  This will likely be merged into core at some point.
- [core-rust](core-rust): Includes the Core API, and the "State Manager" which wraps the Babylon engine - this is written in Rust.
  We pull in the Babylon engine from the [radixdlt-scrypto](https://github.com/radixdlt/radixdlt-scrypto) repository.
- [cli-tools](cli-tools): Various basic command line helpers to assist with spinning up nodes and networks.
- [common](common): Common Java utilities used by various modules. This will likely be merged into core
  once the Olympia Engine has been removed.
- [shell](shell): Examples for how to run Radix shell, which can enable spinning up temporary interactive
   nodes. The Radix Shell code itself is in [cli-tools](cli-tools).
- [docker](docker): An option for running a network of nodes locally. You can also use the "Run Single Validator" IntelliJ option.
- [testnet-node](testnet-node): The easiest way to set up a development environment for integrators (check the [README](testnet-node/README.md))

Until the Babylon engine is feature-compatible with Olympia, we are keeping around the Olympia engine for
some of our tests.

- [olympia-engine](olympia-engine): The Olympia Radix execution layer which provides a UTXO-based state machine

## Contribute

To contribute, you'll need to [setup development environment](docs/development/README.md).

[Contributions](CONTRIBUTING.md) are welcome, we simply ask to:

* Fork the codebase
* Make changes
* Submit a pull request for review

When contributing to this repository, we recommend discussing with the development team the change you wish to make using a [GitHub issue](https://github.com/radixdlt/radixdlt/issues) before making changes.

Please follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all your interactions with the project.

## Links

| Link | Description |
| :----- | :------ |
[radixdlt.com](https://radixdlt.com/) | Radix DLT Homepage
[docs-babylon.radixdlt.com](https://docs-babylon.radixdlt.com/) | Radix Babylon Technical Documentation
[learn.radixdlt.com](https://learn.radixdlt.com/) | Radix Knowledge Base
[discord invite](https://discord.com/invite/WkB2USt) | Radix Discord Server
