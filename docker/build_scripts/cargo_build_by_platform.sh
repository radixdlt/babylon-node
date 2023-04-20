#!/bin/bash

set -x
set -e
set -u
echo "First arg(TARGETPLATFORM): $1"
echo "Second arg(RUST_PROFILE): $2"

TARGETPLATFORM=$1
RUST_PROFILE=$2

if [ $TARGETPLATFORM == 'linux/amd64' ]; then
    TARGET=x86_64-unknown-linux-gnu
    BUILD_ARTIFACT="libcorerust.so"
elif [ $TARGETPLATFORM == 'linux/arm64' ]; then
    TARGET=aarch64-unknown-linux-gnu
    BUILD_ARTIFACT="libcorerust.so"
fi

RUST_BACKTRACE=full $HOME/.cargo/bin/cargo build --target=$TARGET --profile=$RUST_PROFILE
echo "Moving build artifact to root folder /"
cp -R target/$TARGET/release/$BUILD_ARTIFACT / 
rm -rf /app