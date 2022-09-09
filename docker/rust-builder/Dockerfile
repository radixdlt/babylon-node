FROM rust:1.62.1-slim as base
WORKDIR /app

RUN apt update; apt upgrade -y && \
    apt install -y curl && \
    apt install -y g++-x86-64-linux-gnu && \
    apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross && \
    apt install -y build-essential && \
    apt install -y libclang-dev

RUN rustup target add aarch64-unknown-linux-gnu && \
    rustup toolchain install stable-aarch64-unknown-linux-gnu

RUN rustup target add x86_64-unknown-linux-gnu && \
    rustup toolchain install stable-x86_64-unknown-linux-gnu

# This mapped-volume-build is used in development, via the build-sm.yaml docker compose.
# By using mounts, this allows artifact caching. The whole core-rust directory is mounted, and run is called.
# After the above stage, only this stage of the docker file is built in development.
# This part of the docker file is ignored on CI.
FROM base as mapped-volume-build
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

CMD cargo build --target=$TARGET --profile=$RUST_PROFILE

# On CI we use docker build instead of docker compose.
# The mapped-volume-build stage is skipped, and the following stages are run, with sub-stages cached for future runs.
FROM base as cache-packages
ARG TARGET
ARG RUST_PROFILE

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

# First - we build a dummy rust file, to cache the compilation of all our dependencies in a Docker layer
RUN USER=root cargo new dummy
RUN USER=root mkdir -p ./dummy/state-manager/src
RUN USER=root mkdir -p ./dummy/core-rust/src
RUN USER=root mkdir -p ./dummy/core-api-server/src
RUN USER=root touch ./dummy/state-manager/src/lib.rs
RUN USER=root touch ./dummy/core-rust/src/lib.rs
RUN USER=root touch ./dummy/core-api-server/src/lib.rs
COPY core-rust/Cargo.toml ./dummy
COPY core-rust/Cargo.lock ./dummy
COPY core-rust/core-rust/Cargo.toml ./dummy/core-rust
COPY core-rust/state-manager/Cargo.toml ./dummy/state-manager
COPY core-rust/core-api-server/Cargo.toml ./dummy/core-api-server

RUN mv ./dummy/src/main.rs ./dummy/src/lib.rs
WORKDIR /app/dummy
RUN cargo build --target=$TARGET --profile=$RUST_PROFILE
RUN rm -rf /app/dummy/*

FROM cache-packages as prod-build
WORKDIR /app
# Now - we copy in everything, and do the actual build
COPY core-rust ./

RUN cargo build --target=$TARGET --profile=$RUST_PROFILE
RUN cp -R target/$TARGET/release/libcorerust.so /
RUN rm -rf /app

FROM scratch as artifact
COPY --from=prod-build /libcorerust.so /