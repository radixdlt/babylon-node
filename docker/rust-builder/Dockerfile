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

CMD $HOME/.cargo/bin/cargo build --target=$TARGET --profile=$RUST_PROFILE

# On CI we use docker build instead of docker compose.
# The mapped-volume-build stage is skipped, and the following stages are run, with sub-stages cached for future runs.
FROM base as cache-packages
WORKDIR /app
ARG TARGET
ARG RUST_PROFILE

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

RUN $HOME/.cargo/bin/cargo build --target=$TARGET --profile=$RUST_PROFILE

FROM cache-packages as prod-build

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
ENV RUSTC_WRAPPER=/root/.cargo/bin/sccache

WORKDIR /app
# Now - we copy in everything, and do the actual build

RUN rm -rf state-manager core-rust core-api-server
COPY core-rust ./

RUN find . -name '*.rs' | xargs touch; $HOME/.cargo/bin/cargo build --target=$TARGET --profile=$RUST_PROFILE
RUN cp -R target/$TARGET/release/libcorerust.so /
RUN rm -rf /app

FROM scratch as artifact
COPY --from=prod-build /libcorerust.so /
