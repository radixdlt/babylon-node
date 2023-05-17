# =================================================================================================
# BUILD ARGS
#
# For the standard build:
# - TARGETPLATFORM - provided automatically, specified via `docker build --platform xyz`
# - RUST_PROFILE - optional - either `debug` or `release` - defaults to release
#
# There are no args for the local rust builder.
# =================================================================================================
# LAYER: java-build-stage
# The base for building the Java application
# =================================================================================================
FROM debian:11-slim AS java-build-stage

LABEL org.opencontainers.image.authors="devops@radixdlt.com"
LABEL org.opencontainers.image.description="Java + Debian 11 (OpenJDK)"

ENV DEBIAN_FRONTEND noninteractive

CMD ["/bin/bash"]

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    docker.io=20.10.5+dfsg1-1+deb11u2 \
    libssl-dev=1.1.1n-0+deb11u4 \
    pkg-config=0.29.2-1 \
    unzip=6.0-26+deb11u1 \
    wget=1.21-1+deb11u1 \
    software-properties-common=0.96.20.2-2.1 \
  && add-apt-repository -y ppa:openjdk-r/ppa \
  && apt-get install -y --no-install-recommends \
    openjdk-17-jdk=17.0.6+10-1~deb11u1 \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  && wget -q https://services.gradle.org/distributions/gradle-7.2-bin.zip \
  && unzip gradle-7.2-bin.zip -d /opt \
  && rm gradle-7.2-bin.zip

ENV GRADLE_HOME=/opt/gradle-7.2
ENV PATH=/opt/gradle-7.2/bin:$PATH
ENV JAVA_TOOL_OPTIONS=-Dfile.encoding=UTF8

RUN mkdir -p /radixdlt
# Copy the relevant files at the repo root
# I wish we could just avoid pulling in the rust here, but https://github.com/moby/moby/issues/15771
COPY \
  build.gradle \
  gradlew \
  gradlew.bat \
  settings.gradle \
  sonar-project.properties \
  gradle.properties \
  licence-header.txt \
  /radixdlt/
COPY ./gradle /radixdlt/gradle
COPY ./common /radixdlt/common
COPY ./core /radixdlt/core
COPY ./core-rust-bridge /radixdlt/core-rust-bridge
COPY ./olympia-engine /radixdlt/olympia-engine
COPY ./cli-tools /radixdlt/cli-tools
COPY ./shell /radixdlt/shell
# Need .git for tag versions - but this can probably be removed soon
COPY ./.git /radixdlt/.git

WORKDIR /radixdlt

USER root
RUN SKIP_NATIVE_RUST_BUILD=TRUE gradle clean build -x test -Pci=true -PrustBinaryBuildType=release
USER nobody

# =================================================================================================
# LAYER: Keygen
# An alternative build target that executes the keygeneration
# =================================================================================================
FROM --platform=linux/amd64 eclipse-temurin:17-jre-alpine AS keygen
LABEL org.opencontainers.image.authors="devops@radixdlt.com"

COPY --from=java-build-stage /radixdlt/cli-tools/build/distributions /tmp/

RUN mkdir -p /keygen

WORKDIR /keygen/

RUN unzip -j /tmp/*.zip && \
    mkdir -p /keygen/bin /keygen/lib && \
    mv /keygen/*.jar /keygen/lib && \
    mv /keygen/keygen /keygen/bin/keygen 
    
ENTRYPOINT ["bin/keygen"]

# =================================================================================================
# LAYER: java-container
# Exports only the java application artifacts from the java-build-stage
# =================================================================================================
FROM scratch AS java-container
COPY --from=java-build-stage /radixdlt/core/build/distributions /

# =================================================================================================
# LAYER: library-build-stage-base
# Creates the base image for building the rust library
# =================================================================================================
FROM debian:11-slim as library-build-stage-base
WORKDIR /app

# Install dependencies needed for building the Rust library
# - NB: ca-certificates is needed for the rustup installation, and is not a fixed version for security reasons
RUN apt-get update \
  && apt-get -y --no-install-recommends install \
    ca-certificates \
    build-essential=12.9 \
    curl=7.74.0-1.3+deb11u7 \
    g++-aarch64-linux-gnu \
    g++-x86-64-linux-gnu \
    libc6-dev-arm64-cross=2.31-9cross4 \
    libclang-dev=1:11.0-51+nmu5 \
    libssl-dev=1.1.1n-0+deb11u4 \
    pkg-config=0.29.2-1 \
  && rm -rf /var/lib/apt/lists/*

# We fix the version of Rust here to ensure that we can update it without having
# issues with the caching layers containing outdated versions which aren't compatible.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh \
  && sh rustup.sh -y --target 1.68.2-aarch64-unknown-linux-gnu 1.68.2-x86_64-unknown-linux-gnu

RUN "$HOME/.cargo/bin/cargo" install sccache

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
ENV RUSTC_WRAPPER=/root/.cargo/bin/sccache

# =================================================================================================
# LAYER: library-builder-local
# This layer is just used for local building in development via `local-cached-rust-build.yaml`.
# Specifically - the Rust isn't built as part of the image, instead the CMD of the image is to do the build.
# It allows us to use volumes at runtime to cache the build dependencies and artifacts.
# =================================================================================================
FROM library-build-stage-base as library-builder-local
WORKDIR /app

COPY docker/build_scripts/cargo_local_build.sh /opt/radixdlt/cargo_local_build.sh

COPY core-rust ./

# See cargo_local_build.sh script for environment variables to provide
CMD ["/opt/radixdlt/cargo_local_build.sh"]

# =================================================================================================
# LAYER: library-build-stage-cache-packages
# This layer allows us to cache the compilation of all our rust dependencies in a Docker layer
# =================================================================================================
FROM library-build-stage-base as library-build-stage-cache-packages

WORKDIR /app

# First - we build a dummy rust file, to cache the compilation of all our dependencies in a Docker layer
RUN USER=root "$HOME/.cargo/bin/cargo" init --lib --name dummy --vcs none . \
  && mkdir -p ./core-api-server/src \
  && mkdir -p ./jni-export/src \
  && mkdir -p ./node-common/src \
  && mkdir -p ./state-manager/src \
  && touch ./core-api-server/src/lib.rs \
  && touch ./jni-export/src/lib.rs \
  && touch ./node-common/src/lib.rs \
  && touch ./state-manager/src/lib.rs
COPY core-rust/Cargo.toml ./
COPY core-rust/Cargo.lock ./
COPY core-rust/core-api-server/Cargo.toml ./core-api-server
COPY core-rust/jni-export/Cargo.toml ./jni-export
COPY core-rust/node-common/Cargo.toml ./node-common
COPY core-rust/state-manager/Cargo.toml ./state-manager

COPY docker/build_scripts/cargo_build_by_platform.sh /opt/radixdlt/cargo_build_by_platform.sh

ARG TARGETPLATFORM
ARG RUST_PROFILE=release
# This caches the sccache (ie rust incremental compilation artifacts) between builds
# See https://docs.docker.com/build/cache/ and https://github.com/mozilla/sccache/issues/547
# In CI we can further improve builds by configuring sccache to use a shared cache
RUN --mount=type=cache,id=radixdlt-babylon-node-rust-cache,target=/root/.cache/sccache \
  /opt/radixdlt/cargo_build_by_platform.sh $TARGETPLATFORM $RUST_PROFILE

# =================================================================================================
# LAYER: library-build-stage
# The actual build of the library
# =================================================================================================
FROM library-build-stage-cache-packages as library-build-stage

# Tidy up from the previous layer
RUN rm -rf core-api-server jni-export node-common state-manager

# Copy across all the code (docker ignore excepted)
COPY core-rust ./

ARG TARGETPLATFORM
ARG RUST_PROFILE=release
# This caches the sccache (ie rust incremental compilation artifacts) between builds
# See https://docs.docker.com/build/cache/ and https://github.com/mozilla/sccache/issues/547
# In CI we can further improve builds by configuring sccache to use a shared cache
# Once we do this, we may be able to remove the library-build-stage-cache-packages workaround
RUN --mount=type=cache,id=radixdlt-babylon-node-rust-cache,target=/root/.cache/sccache \
  /opt/radixdlt/cargo_build_by_platform.sh $TARGETPLATFORM $RUST_PROFILE

# =================================================================================================
# LAYER: library-container
# A layer containing just the built library at the root: /libcorerust.so
# =================================================================================================
FROM scratch as library-container
COPY --from=library-build-stage /libcorerust.so /

# =================================================================================================
# LAYER: app-container
# The application container which will actually run the application
# =================================================================================================
FROM debian:11-slim as app-container
LABEL org.opencontainers.image.authors="devops@radixdlt.com"

# Install dependencies needed for building the image or running the application
# - unzip is needed for unpacking the java build artifacts
# - daemontools is needed at application runtime for async tasks
# - libssl-dev is needed for encryption methods used in the keystore.ks
# - software-properties-common is needed for installing debian packages with dpkg
# - gettext-base is needed for envsubst in config_radixdlt.sh
RUN apt-get update -y \
  && apt-get -y --no-install-recommends install \
    openjdk-17-jre-headless=17.0.6+10-1~deb11u1 \
    unzip=6.0-26+deb11u1 \
    daemontools=1:0.76-7 \
    libssl-dev=1.1.1n-0+deb11u4 \
    software-properties-common=0.96.20.2-2.1 \
    gettext-base=0.21-4 \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

COPY docker/build_scripts/fix-vulnerabilities.sh /tmp
RUN /tmp/fix-vulnerabilities.sh

# Copy configuration templates
WORKDIR /home/radixdlt
COPY docker/config/ /home/radixdlt/

# Add script to install optional network utils - but don't run it
COPY docker/build_scripts/install_network_utils.sh /opt/radixdlt/install_network_utils.sh

# Configure the welcome message when a shell is started.
# Docker defaults to using /bin/sh, so we can't use .bashrc and have to hook into ENV instead
ENV ENV=/etc/environment
COPY docker/build_scripts/configure_motd.sh /tmp
RUN /tmp/configure_motd.sh

# Add healthcheck
COPY docker/build_scripts/docker-healthcheck.sh /home/radixdlt/
RUN chmod +x /home/radixdlt/docker-healthcheck.sh
HEALTHCHECK CMD sh /home/radixdlt/docker-healthcheck.sh

# Set default environment variables
ENV RADIXDLT_HOME=/home/radixdlt \
    RADIXDLT_NETWORK_SEEDS_REMOTE=127.0.0.1 \
    RADIXDLT_DB_LOCATION=./RADIXDB \
    RADIXDLT_DB_ACCOUNT_CHANGE_INDEX_ENABLE=true \
    RADIXDLT_DB_LOCAL_TRANSACTION_EXECUTION_INDEX_ENABLE=true \
    RADIXDLT_VALIDATOR_KEY_LOCATION=/home/radixdlt/node.ks \
    RADIXDLT_NETWORK_USE_PROXY_PROTOCOL=false \
    RADIXDLT_CORE_API_PORT=3333 \
    RADIXDLT_CORE_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_SYSTEM_API_PORT=3334 \
    RADIXDLT_SYSTEM_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_PROMETHEUS_API_PORT=3335 \
    RADIXDLT_PROMETHEUS_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_NETWORK_ID=240 \
    RADIXDLT_NODE_KEY_CREATE_IF_MISSING=false

# Copy in the application artifacts
COPY --from=java-container / /tmp
RUN unzip -j /tmp/*.zip && mkdir -p /opt/radixdlt/bin && \
    mkdir -p /opt/radixdlt/lib && \
    ls -lah && \
    pwd && \
    mv /home/radixdlt/core /opt/radixdlt/bin/core && \
    mv /home/radixdlt/*.jar /opt/radixdlt/lib/ 

COPY --from=library-container /libcorerust.so /usr/lib/jni/libcorerust.so

# Create configuration automatically when starting
COPY docker/build_scripts/config_radixdlt.sh /opt/radixdlt/config_radixdlt.sh

# The entrypoint `config_radixdlt.sh` finishes configuration and then runs its parameters (ie CMD) as the radixdlt user
# See https://docs.docker.com/engine/reference/builder/#entrypoint
ENTRYPOINT ["/opt/radixdlt/config_radixdlt.sh"]
CMD ["/opt/radixdlt/bin/core"]
