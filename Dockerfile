# All build arguments

# TARGETPLATFORM - automatic assignment via `docker build --platform xyz`
# RUST_PROFILE - defaults to release

# Build babylon-node binary
FROM debian:11-slim AS binary-build-stage
LABEL org.opencontainers.image.authors="devops@radixdlt.com"
LABEL org.opencontainers.image.description="Java + Debian 11 (OpenJDK)"

ENV DEBIAN_FRONTEND noninteractive

CMD ["/bin/bash"]

RUN  \
  apt-get update && apt-get install -y --no-install-recommends \
  docker.io=20.10.5+dfsg1-1+deb11u2 \
  libssl-dev=1.1.1n-0+deb11u4 \
  pkg-config=0.29.2-1 \
  unzip=6.0-26+deb11u1 \
  wget=1.21-1+deb11u1 \
  software-properties-common=0.96.20.2-2.1 
RUN add-apt-repository -y ppa:openjdk-r/ppa && \
  apt-get install -y --no-install-recommends \
  openjdk-17-jdk=17.0.6+10-1~deb11u1

#install Gradle
RUN wget -q https://services.gradle.org/distributions/gradle-7.2-bin.zip \
  && unzip gradle-7.2-bin.zip -d /opt \
  && rm gradle-7.2-bin.zip

# Set Gradle in the environment variables
ENV GRADLE_HOME=/opt/gradle-7.2
ENV PATH=/opt/gradle-7.2/bin:$PATH
ENV JAVA_TOOL_OPTIONS=-Dfile.encoding=UTF8

COPY . /radixdlt
WORKDIR /radixdlt
USER root
RUN SKIP_NATIVE_RUST_BUILD=TRUE gradle clean build -x test -Pci=true -PrustBinaryBuildType=release

USER nobody

# Exporting babylon-node binary
FROM scratch AS binary-container
COPY --from=binary-build-stage /radixdlt/core/build/distributions /

FROM ubuntu:22.04 as base
WORKDIR /app

RUN apt-get update; apt-get upgrade -y && \
  apt install -y \
  build-essential \
  curl \
  g++-aarch64-linux-gnu \
  g++-x86-64-linux-gnu \
  libc6-dev-arm64-cross \
  libclang-dev \
  libssl-dev \
  pkg-config

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh; sh rustup.sh -y --target aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu
RUN $HOME/.cargo/bin/cargo install sccache

# This mapped-volume-build is used in development, via the build-sm.yaml docker compose.
# By using mounts, this allows artifact caching. The whole core-rust directory is mounted, and run is called.
# After the above stage, only this stage of the docker file is built in development.
# This part of the docker file is ignored on CI.
FROM base as mapped-volume-build
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
ENV RUSTC_WRAPPER=/root/.cargo/bin/sccache


ARG TARGETPLATFORM
COPY docker/scripts/cargo_build_by_platform.sh /opt/radixdlt/cargo_build_by_platform.sh
CMD /opt/radixdlt/cargo_build_by_platform.sh build_cache $TARGETPLATFORM release

# On CI we use docker build instead of docker compose.
# The mapped-volume-build stage is skipped, and the following stages are run, with sub-stages cached for future runs.
FROM base as cache-packages
WORKDIR /app

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
ENV RUSTC_WRAPPER=/root/.cargo/bin/sccache

# First - we build a dummy rust file, to cache the compilation of all our dependencies in a Docker layer
RUN USER=root $HOME/.cargo/bin/cargo init --lib --name dummy --vcs none .
RUN USER=root mkdir -p ./state-manager/src
RUN USER=root mkdir -p ./core-rust/src
RUN USER=root mkdir -p ./core-api-server/src
RUN USER=root touch ./state-manager/src/lib.rs
RUN USER=root touch ./core-rust/src/lib.rs
RUN USER=root touch ./core-api-server/src/lib.rs
COPY core-rust/Cargo.toml ./
COPY core-rust/Cargo.lock ./
COPY core-rust/core-rust/Cargo.toml ./core-rust
COPY core-rust/state-manager/Cargo.toml ./state-manager
COPY core-rust/core-api-server/Cargo.toml ./core-api-server

ARG TARGETPLATFORM
ARG RUST_PROFILE
COPY docker/scripts/cargo_build_by_platform.sh /opt/radixdlt/cargo_build_by_platform.sh
RUN /opt/radixdlt/cargo_build_by_platform.sh build_cache $TARGETPLATFORM release

FROM cache-packages as library-builder

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
ENV RUSTC_WRAPPER=/root/.cargo/bin/sccache

WORKDIR /app
# Now - we copy in everything, and do the actual build

RUN rm -rf state-manager core-rust core-api-server
COPY core-rust ./

RUN /opt/radixdlt/cargo_build_by_platform.sh build $TARGETPLATFORM release

FROM scratch as library-container
COPY --from=library-builder /libcorerust.so /

# Building the application container
FROM debian:11-slim as app-container
LABEL org.opencontainers.image.authors="devops@radixdlt.com"

# unzip is needed for unpacking the java build artifacts
# daemontools is needed at application runtime for async tasks
# libssl-dev is needed for encryption methods used in the keystore.ks
# software-properties-common is neeed for installing debian packages with dpkg
# gettext-base is needed for envsubst in config_radixdlt.sh
RUN apt-get update -y && \
    apt-get -y --no-install-recommends install \
    openjdk-17-jre-headless=17.0.6+10-1~deb11u1 \
    unzip=6.0-26+deb11u1 \
    daemontools=1:0.76-7 \
    libssl-dev=1.1.1n-0+deb11u4 \
    software-properties-common=0.96.20.2-2.1 \
    gettext-base=0.21-4 && \  
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY docker/scripts/fix-vulnerabilities.sh /tmp
RUN /tmp/fix-vulnerabilities.sh

# create configuration automatically when starting
COPY docker/scripts/config_radixdlt.sh /opt/radixdlt/config_radixdlt.sh

# copy configuration templates
WORKDIR /home/radixdlt
COPY docker/config/ /home/radixdlt/

# Add script to install optional network utils
COPY docker/scripts/install_network_utils.sh /opt/radixdlt/install_network_utils.sh

RUN echo '[ ! -z "$TERM" -a -r /etc/motd ] && cat /etc/issue && cat /etc/motd' \
    >> /etc/bash.bashrc \
    ; echo "\
===================================================================\n\
= Babylon-node Docker container                                        =\n\
===================================================================\n\
\n\
Welcome to the babylon-node docker container.\n\
You will find the java application in `/opt/radixdlt/bin/core` \n\
and the rust library in `/usr/lib/jni/libcorerust.so` \n\
\n\
All network-utilities have been removed to reduce the image size. \n\
However we left you the instructions to install them in this file: \n\
- /opt/radixdlt/install_network_utils.sh"\n\
    > /etc/motd

# Add healthcheck
COPY docker/scripts/docker-healthcheck.sh /home/radixdlt/
RUN chmod +x /home/radixdlt/docker-healthcheck.sh
HEALTHCHECK CMD sh /home/radixdlt/docker-healthcheck.sh

# set default environment variables
ENV RADIXDLT_HOME=/home/radixdlt \
    RADIXDLT_NETWORK_SEEDS_REMOTE=127.0.0.1 \
    RADIXDLT_DB_LOCATION=./RADIXDB \
    RADIXDLT_VALIDATOR_KEY_LOCATION=/home/radixdlt/node.ks \
    RADIXDLT_NETWORK_USE_PROXY_PROTOCOL=false \
    RADIXDLT_CORE_API_PORT=3333 \
    RADIXDLT_CORE_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_SYSTEM_API_PORT=3334 \
    RADIXDLT_SYSTEM_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_PROMETHEUS_API_PORT=3335 \
    RADIXDLT_PROMETHEUS_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_NETWORK_ID=240 \
    RADIXDLT_DB_ACCOUNT_CHANGE_INDEX_ENABLE=true

COPY --from=binary-container / /tmp
RUN unzip -j /tmp/*.zip && mkdir -p /opt/radixdlt/bin && \
    mkdir -p /opt/radixdlt/lib && \
    ls -lah && \
    pwd && \
    mv /home/radixdlt/core /opt/radixdlt/bin/core && \
    mv /home/radixdlt/*.jar /opt/radixdlt/lib/ 

COPY --from=library-container /libcorerust.so /usr/lib/jni/libcorerust.so

# set entrypoint and default command
ENTRYPOINT ["/opt/radixdlt/config_radixdlt.sh"]
CMD ["/opt/radixdlt/bin/core"]