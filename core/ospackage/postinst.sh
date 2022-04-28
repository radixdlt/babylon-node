#!/bin/sh

set -ex

# NB - If updating this script, also update ./docker/scripts/config_radixdlt.sh

# vars
RADIXDLT_USER=radixdlt
RADIXDLT_SERVICE_FILE=radixdlt.service
RADIXDLT_DIRECTORY_NAME=radixdlt

# paths
RADIXDLT_HOME="/opt/$RADIXDLT_DIRECTORY_NAME"
RADIXDLT_LOG_DIR="/var/log/$RADIXDLT_DIRECTORY_NAME"

# create user and group idempotently
getent group $RADIXDLT_USER >/dev/null || groupadd -r $RADIXDLT_USER
getent passwd $RADIXDLT_USER >/dev/null || useradd -r -d "$RADIXDLT_HOME" -g $RADIXDLT_USER $RADIXDLT_USER

# create log dir
mkdir -p "$RADIXDLT_LOG_DIR"

# make sure all files are owned by the radixdlt user/group
chown -Rf "$RADIXDLT_USER:$RADIXDLT_USER" "$RADIXDLT_HOME" "$RADIXDLT_LOG_DIR"

# Make sure that systemd files are owned by root
chown root:root "/etc/systemd/system/$RADIXDLT_SERVICE_FILE"

#systemctl daemon-reload
#systemctl start $RADIXDLT_SERVICE_FILE
