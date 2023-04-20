#!/bin/bash

set -x
set -e
set -u

if [[ "$CARGO_CLEAN" = "TRUE" ]]; then
    $HOME/.cargo/bin/cargo clean
fi

RUST_BACKTRACE=full $HOME/.cargo/bin/cargo build --target=$TARGET --profile=$RUST_PROFILE

echo "Moving build artifact to /output"

cp target/$TARGET/$BINARY_BUILD_TYPE/$BUILD_ARTIFACT /output