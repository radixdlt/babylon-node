# =================================================================================================
# LAYER: library-build-stage-base
# Creates the base image for building the rust library
# =================================================================================================
FROM debian:12.1-slim AS library-build-stage-base
WORKDIR /app


# The installed versions are fixed to create an immutable build.
# Availability of fixed version is subject to change.
# The latest available version can be found at these links.
# Update the references versions in case the build fails
# Source Repositories:
# - https://packages.debian.org/bookworm/build-essential
# - https://packages.debian.org/bookworm/curl
# - https://packages.debian.org/bookworm/libc6-dev-arm64-cross
# - https://packages.debian.org/bookworm/libclang-dev
# - https://packages.debian.org/bookworm/libssl-dev
# - https://packages.debian.org/bookworm/pkg-config
# Install dependencies needed for building the Rust library
# - NB: ca-certificates is needed for the rustup installation, and is not a fixed version for security reasons
# hadolint ignore=DL3008
RUN apt-get update \
  && apt-get -y --no-install-recommends install \
    ca-certificates \
    build-essential=12.9 \
    # https://security-tracker.debian.org/tracker/CVE-2023-38545
    curl=7.88.1-10+deb12u7 \
    g++-aarch64-linux-gnu \
    g++-x86-64-linux-gnu \
    libc6-dev-arm64-cross=2.36-8cross1 \
    libclang-dev=1:14.0-55.7~deb12u1 \
    libssl-dev=3.0.14-1~deb12u2 \
    pkg-config=1.8.1-1 \
  && rm -rf /var/lib/apt/lists/*

# We fix the version of Rust here to ensure that we can update it without having
# issues with the caching layers containing outdated versions which aren't compatible.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh \
  && sh rustup.sh -y --target 1.71.1-aarch64-unknown-linux-gnu 1.71.1-x86_64-unknown-linux-gnu --default-toolchain 1.77.2

RUN "$HOME/.cargo/bin/cargo" install sccache --version 0.7.4

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
ENV RUSTC_WRAPPER=/root/.cargo/bin/sccache
