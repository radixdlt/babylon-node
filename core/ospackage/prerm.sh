#!/bin/sh

set -x

# vars
RADIXDLT_SERVICE_FILE=radixdlt.service

# kill the process
systemctl stop $RADIXDLT_SERVICE_FILE || :