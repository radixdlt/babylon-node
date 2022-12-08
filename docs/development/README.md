## Development Environment Setup

- Java 17.0.4+ SDK installed and configured. It's very important to have at least 17.0.4, else you will hit Java Compiler bugs.
- latest stable installed and configured (recommended via rustup)
- More or less recent Linux or MacOS (Windows WSL2 may work, but not tested)
- git 2.27+
- docker version 20.10+
- docker-compose version 1.25+

Two last prerequisites are necessary only if you plan to launch a local network through Docker.

Please note that many installations require shell restart to become effective (due to `$PATH` etc.), with the most
notorious being `cargo` - it may happen that only a full system reboot allows for a successful initial build.

### Getting code

* External contributors: please fork the main repository https://github.com/radixdlt/babylon-node into your account and then clone it locally.
* Otherwise, just clone the main repo at https://github.com/radixdlt/babylon-node

### Building code
Use following command to build binaries and run unit tests:

```shell
$ ./gradlew clean build
```

### Running integration tests

Integration tests take quite a while to run (over an hour on most machines).

They are typically run as part of a PR.

```shell
$ ./gradlew runAllIntegrationTests
```

### IntelliJ IDEA Troubleshooting
In some cases IntelliJ IDEA may deny to load project properly. Usually this happens if you have installed more than one Java version.
If you meet this issue, check following configuration options:
 - `Project Structure -> Project Settings -> Project`, make sure `Project SDK` and `Project Language Level` is set to `17 (Preview) - Pattern matching for switch`.
 - `Project Structure -> Project Settings -> Modules`, make sure that every module has `Language Level` set to `17 (Preview) - Pattern matching for switch (Project default)`  
 - `Settings -> Build,Execution, Deployment -> Build Tools -> Gradle`, make sure that `Gradle JVM` is set to `Project JDK`. 

There are a variety of [run configurations](./run-configurations), depending on how you'd like to test your code:

* [Launching a local network in Docker](./run-configurations/launching-a-local-network-in-docker.md)
* [Connecting to a live network via Docker](./run-configurations/connecting-to-a-live-network-in-docker.md)
* Connecting to a live network without Docker
* Running with nginx in front of the node (to replicate a more production-like setup)
