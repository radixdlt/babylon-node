#!/bin/bash

set -x
set -e
set -u
echo "First arg(COMMANDSTAGE): $1"
echo "Second arg(TARGETPLATFORM): $2"
echo "Third arg(RUST_PROFILE): $3"

COMMANDSTAGE=$1
TARGETPLATFORM=$2

if [ $TARGETPLATFORM == 'linux/amd64' ]; then
    TARGET=x86_64-unknown-linux-gnu
    BUILD_ARTIFACT="libcorerust.so"
elif [ $TARGETPLATFORM == 'linux/arm64' ]; then
    TARGET=aarch64-unknown-linux-gnu
    BUILD_ARTIFACT="libcorerust.so"
fi

if [ $COMMANDSTAGE == 'build-cache' ]; then
    RUST_PROFILE=$3
    RUST_BACKTRACE=full $HOME/.cargo/bin/cargo build --release --target=$TARGET --profile=$RUST_PROFILE
elif [ $COMMANDSTAGE == 'build' ]; then
    RUST_PROFILE=$3
    RUST_BACKTRACE=full $HOME/.cargo/bin/cargo build --release --target=$TARGET --profile=$RUST_PROFILE
    echo "Moving build artifact to root folder /"
    cp -R target/$TARGET/release/$BUILD_ARTIFACT / 
    rm -rf /app
fi