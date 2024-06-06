# Radix Babylon Node

This is the repository for the RadixDLT node, for the Babylon release and beyond.

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

## Contribute

Please follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all your interactions with the project.
See the [Contributing Guide](CONTRIBUTING.md) for more details on how to get involved.

## Links

| Link                                                            | Description                           |
|-----------------------------------------------------------------|---------------------------------------|
| [radixdlt.com](https://radixdlt.com/)                           | Radix DLT Homepage                    |                   
| [docs-babylon.radixdlt.com](https://docs-babylon.radixdlt.com/) | Radix Babylon Technical Documentation |
| [learn.radixdlt.com](https://learn.radixdlt.com/)               | Radix Knowledge Base                  |
| [discord invite](https://discord.com/invite/WkB2USt)            | Radix Discord Server                  |

## License

The executable components of the Babylon Node code are licensed under the [Radix Node EULA](http://www.radixdlt.com/terms/nodeEULA).

The Babylon Node code is released under the [Radix License 1.0 (modified Apache 2.0)](LICENSE):

```
Copyright 2023 Radix Publishing Ltd incorporated in Jersey, Channel Islands.

Licensed under the Radix License, Version 1.0 (the "License"); you may not use
this file except in compliance with the License.

You may obtain a copy of the License at:
https://www.radixfoundation.org/licenses/license-v1

The Licensor hereby grants permission for the Canonical version of the Work to
be published, distributed and used under or by reference to the Licensor’s
trademark Radix® and use of any unregistered trade names, logos or get-up.

The Licensor provides the Work (and each Contributor provides its Contributions)
on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
express or implied, including, without limitation, any warranties or conditions
of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A PARTICULAR
PURPOSE.

Whilst the Work is capable of being deployed, used and adopted (instantiated) to
create a distributed ledger it is your responsibility to test and validate the
code, together with all logic and performance of that code under all foreseeable
scenarios.

The Licensor does not make or purport to make and hereby excludes liability for
all and any representation, warranty or undertaking in any form whatsoever,
whether express or implied, to any entity or person, including any
representation, warranty or undertaking, as to the functionality security use,
value or other characteristics of any distributed ledger nor in respect the
functioning or value of any tokens which may be created stored or transferred
using the Work.

The Licensor does not warrant that the Work or any use of the Work complies with
any law or regulation in any territory where it may be implemented or used or
that it will be appropriate for any specific purpose.

Neither the licensor nor any current or former employees, officers, directors,
partners, trustees, representatives, agents, advisors, contractors, or
volunteers of the Licensor shall be liable for any direct or indirect, special,
incidental, consequential or other losses of any kind, in tort, contract or
otherwise (including but not limited to loss of revenue, income or profits, or
loss of use or data, or loss of reputation, or loss of any economic or other
opportunity of whatsoever nature or howsoever arising), arising out of or in
connection with (without limitation of any use, misuse, of any ledger system or
use made or its functionality or any performance or operation of any code or
protocol caused by bugs or programming or logic errors or otherwise);

A. any offer, purchase, holding, use, sale, exchange or transmission of any
cryptographic keys, tokens or assets created, exchanged, stored or arising from
any interaction with the Work;

B. any failure in a transmission or loss of any token or assets keys or other
digital artifacts due to errors in transmission;

C. bugs, hacks, logic errors or faults in the Work or any communication;

D. system software or apparatus including but not limited to losses caused by
errors in holding or transmitting tokens by any third-party;

E. breaches or failure of security including hacker attacks, loss or disclosure
of password, loss of private key, unauthorised use or misuse of such passwords
or keys;

F. any losses including loss of anticipated savings or other benefits resulting
from use of the Work or any changes to the Work (however implemented).

You are solely responsible for; testing, validating and evaluation of all
operation logic, functionality, security and appropriateness of using the Work
for any commercial or non-commercial purpose and for any reproduction or
redistribution by You of the Work. You assume all risks associated with Your use
of the Work and the exercise of permissions under this Licence.
```
