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
FROM debian:12.1-slim AS java-build-stage

LABEL org.opencontainers.image.source="https://github.com/radixdlt/babylon-node"
LABEL org.opencontainers.image.authors="devops@radixdlt.com"
LABEL org.opencontainers.image.description="Java + Debian 12 (OpenJDK)"

ENV DEBIAN_FRONTEND=noninteractive

CMD ["/bin/bash"]

ARG WGET_VERSION="1.21.3-1+b2"
ARG VERSION_BRANCH=""
ARG VERSION_COMMIT=""
ARG VERSION_DISPLAY=""
ARG VERSION_BUILD=""
ARG VERSION_TAG=""
ARG VERSION_LAST_TAG=""

ENV VERSION_BRANCH=$VERSION_BRANCH
ENV VERSION_COMMIT=$VERSION_COMMIT
ENV VERSION_DISPLAY=$VERSION_DISPLAY
ENV VERSION_BUILD=$VERSION_BUILD
ENV VERSION_TAG=$VERSION_TAG
ENV VERSION_LAST_TAG=$VERSION_LAST_TAG

# The installed versions are fixed to create an immutable build.
# Availability of fixed version is subject to change.
# The latest available version can be found at these links.
# Update the references versions in case the build fails
# Source Repositories:
# - https://packages.debian.org/bookworm/docker.io
# - https://packages.debian.org/bookworm/libssl-dev
# - https://packages.debian.org/bookworm/pkg-config
# - https://packages.debian.org/bookworm/unzip
# - https://packages.debian.org/bookworm/wget
# - https://packages.debian.org/bookworm/software-properties-common
# - https://packages.debian.org/bookworm/openjdk-17-jdk
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    docker.io=20.10.24+dfsg1-1+b3 \
    libssl-dev=3.0.14-1~deb12u2 \
    pkg-config=1.8.1-1 \
    unzip=6.0-28 \
    wget \
    software-properties-common=0.99.30-4.1~deb12u1 \
  && apt-get install -y --no-install-recommends \
    openjdk-17-jdk=17.0.12+7-2~deb12u1 \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

ENV JAVA_TOOL_OPTIONS="-Dfile.encoding=UTF8"

RUN mkdir -p /radixdlt

WORKDIR /radixdlt

USER root
