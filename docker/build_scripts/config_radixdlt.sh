#!/bin/bash

set -ex

# Need this to run on Alpine (grsec) kernels without crashing
find /usr -type f -name java -exec setfattr -n user.pax.flags -v em {} \;

# Sets USER_ID to LOCAL_USER_ID if provided, else set it to 999
USER_ID=${LOCAL_USER_ID:-999}
USER_NAME=radixdlt

# Check and delete the user that is created in postinstall action of deb package
getent group $USER_NAME >/dev/null && groupmod -g $USER_ID radixdlt || groupadd -r $USER_NAME -g $USER_ID
getent passwd $USER_NAME >/dev/null && usermod -u $USER_ID radixdlt || useradd -r -d "$RADIXDLT_HOME" -g $USER_NAME $USER_NAME
chown -R radixdlt:radixdlt /home/radixdlt/
chmod u=xr /opt/radixdlt/bin/core

# Check for test network configs
TEST_CONFIGS="${RADIXDLT_HOME:?}"/test.config
if test -f "$TEST_CONFIGS"; then
    echo >> default.config.envsubst
    cat $TEST_CONFIGS >> default.config.envsubst
fi

env | sort

envsubst <"${RADIXDLT_HOME:?}"/default.config.envsubst >"${RADIXDLT_HOME:?}"/default.config

# This command changes the process from executing this script to executing the argument to the script,
# running as the user radixdlt.
#
# More specifically:
# - The exec command replaces the shell with the given command without creating a new process
# - The setuidgid command runs another program under a specified account's uid and gid
# - And the $* command is a special variable that contains all the positional parameters 
exec setuidgid radixdlt $*
