#!/bin/sh
# Fail on error
set -e

echo "\nRegenerating Core API models (Rust + Java)...\n"
python3 ./core-rust/core-api-server/scripts/generate-openapi-server.py

echo "\nRegenerating Engine State API models (Rust + Java)...\n"
python3 ./core-rust/engine-state-api-server/scripts/generate-openapi-server.py

echo "\nRegenerating Core API models (Typescript)...\n"
python3 ./sdk/typescript/regeneration/generate-typescript-core-api-client.py

echo "\nRegenerating System API models (Java)...\n"
python3 ./core/src/main/java/com/radixdlt/api/regeneration/generate.py