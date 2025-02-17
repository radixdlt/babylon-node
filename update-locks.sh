#!/bin/sh
# Fail on error
set -e

echo "\n>> Writing locks for core dependencies...\n"
./gradlew :core:dependencies --write-locks | sed '/BUILD SUCCESSFUL/d' | tee ./core/dependencies-tree.txt

echo "\n>> Writing locks for cli-tools dependencies...\n"
./gradlew :cli-tools:dependencies --write-locks | sed '/BUILD SUCCESSFUL/d' | tee ./cli-tools/dependencies-tree.txt

echo "\n>> Writing locks for common dependencies...\n"
./gradlew :common:dependencies --write-locks | sed '/BUILD SUCCESSFUL/d' | tee ./common/dependencies-tree.txt

echo "\n>> Writing locks for core-rust-bridge dependencies...\n"
./gradlew :core-rust-bridge:dependencies --write-locks | sed '/BUILD SUCCESSFUL/d' | tee ./core-rust-bridge/dependencies-tree.txt

# Note - core-rust isn't needed as it has no gradle dependencies

echo "\n>> Writing locks for keygen dependencies...\n"
./gradlew :keygen:dependencies --write-locks | sed '/BUILD SUCCESSFUL/d' | tee ./keygen/dependencies-tree.txt

echo "\n>> DONE"