FROM rust:1.62.1

WORKDIR /app

RUN apt update; apt upgrade -y
RUN apt install -y curl
RUN apt install -y g++-mingw-w64-x86-64
RUN apt install -y g++-x86-64-linux-gnu
RUN apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross
RUN apt install -y build-essential

RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup toolchain install stable-aarch64-unknown-linux-gnu

RUN rustup target add x86_64-pc-windows-gnu
RUN rustup toolchain install stable-x86_64-pc-windows-gnu

RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup toolchain install stable-x86_64-unknown-linux-gnu

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
