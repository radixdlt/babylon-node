# Fail on error
set -e

# Where we are run from
scriptdir=$(dirname "$0")

cd "$scriptdir"
docker compose up --build