FROM rust:1.62.1-slim as base
WORKDIR /app

RUN apt update; apt upgrade -y && \
    apt install -y curl && \
    apt install -y g++-x86-64-linux-gnu && \
    apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross && \
    apt install -y build-essential

RUN rustup target add aarch64-unknown-linux-gnu && \
    rustup toolchain install stable-aarch64-unknown-linux-gnu

RUN rustup target add x86_64-unknown-linux-gnu && \
    rustup toolchain install stable-x86_64-unknown-linux-gnu

FROM base as mapped-volume-build
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

CMD cargo build --target=${TARGET} --release

FROM base as cache-packages
ARG TARGET

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

# First - we build a dummy rust file, to cache the compilation of all our dependencies in a Docker layer
RUN USER=root cargo new dummy
COPY state-manager/Cargo.toml ./dummy
COPY state-manager/Cargo.lock ./dummy

RUN mv ./dummy/src/main.rs ./dummy/src/lib.rs
WORKDIR /app/dummy
RUN cargo build --target=$TARGET --release
RUN rm -rf /app/dummy/*

FROM cache-packages as prod-build
WORKDIR /app
# Now - we copy in everything, and do the actual built
COPY state-manager ./

RUN cargo build --target=$TARGET --release
RUN cp -R target/$TARGET/release/libstatemanager.so /
RUN rm -rf /app

FROM scratch as artifact
COPY --from=prod-build /libstatemanager.so /