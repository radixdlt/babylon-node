#!/bin/bash
set -ex

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
if [[ -z "${LOCAL_USER_ID}" ]]; then
    echo "Running with default uid and gid..."
else
    usermod -u $LOCAL_USER_ID radixdlt    
    groupmod -g $LOCAL_USER_ID radixdlt  
fi
exec setuidgid radixdlt $*
