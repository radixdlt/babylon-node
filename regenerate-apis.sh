#!/bin/sh
# Fail on error
set -e

echo "\nRegenerating all Node HTTP APIs models (Rust + Java)...\n"
python3 ./core-rust/node-http-apis/scripts/generate-openapi-servers.py

echo "\nRegenerating Core API models (Typescript)...\n"
python3 ./sdk/typescript/regeneration/generate-typescript-core-api-client.py

echo "\nRegenerating System API models (Java)...\n"
python3 ./core/src/main/java/com/radixdlt/api/regeneration/generate.py