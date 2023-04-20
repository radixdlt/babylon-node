#!/bin/bash

# Fail on error
set -e

# Where we are run from
scriptdir=$(dirname "$0")

# Number of validators
validators_count=${1:-1}

# Check for dockerfile
dockerfile="${scriptdir}/../network-size-${validators_count}.yml"
if [ ! -f "${dockerfile}" ]; then
  echo "Can't find ${dockerfile}, aborting. Ensure you specify the number of validators as a parameter to this script, eg: rundocker.sh 2"
  exit 1
fi

reporoot="${scriptdir}/../.."

# As we're building rust for docker in the rust-builder, we don't need to run the rust build separately.
export SKIP_NATIVE_RUST_BUILD=TRUE

echo "||> Generating environment variables for running with $validators_count validators"
# generateDevUniverse outputs a bunch of "export" lines to stdout.
ENV_TO_LOAD=$(${reporoot}/gradlew -q -P "validators=${validators_count}" ":cli-tools:generateDevUniverse")

echo "||> Loading environment variables for $validators_count validators"
# If the below line errors with a syntax error, see what's going wrong by adding echo "$ENV_TO_LOAD"
eval "$ENV_TO_LOAD"

echo "||> Killing all running docker containers"
docker compose -f "${dockerfile}" down

function cleanup {
  echo "||> Killing all running docker containers"
  docker compose -f "${dockerfile}" down
}

trap cleanup EXIT

echo "||> Starting up new docker containers"
docker compose -f "${dockerfile}" up --build
