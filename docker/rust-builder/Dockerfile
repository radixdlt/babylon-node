FROM rust:1.62.1 as build
ARG TARGET
WORKDIR /app

RUN apt update; apt upgrade -y
RUN apt install -y curl
RUN apt install -y g++-mingw-w64-x86-64
RUN apt install -y g++-x86-64-linux-gnu
RUN apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross
RUN apt install -y build-essential

RUN case "$TARGET" in \
      "x86_64-unknown-linux-gnu") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
      "x86_64-pc-windows-gnu") echo x86_64-pc-windows-gnu > /rust_target.txt ;; \
      "arch64-unknown-linux-gnu") echo arch64-unknown-linux-gnu > /rust_target.txt ;; \
      *) exit 1 ;; \
    esac
RUN rustup target add $(cat /rust_target.txt)
RUN rustup toolchain install stable-$(cat /rust_target.txt)

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
COPY state-manager ./

RUN cargo build --target=$TARGET
RUN mkdir -p /app/binary
RUN cp -R target /app/output


FROM scratch as artifact
COPY --from=build /app/output /