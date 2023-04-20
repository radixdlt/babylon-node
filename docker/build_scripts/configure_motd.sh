#!/bin/bash

# Configure the message-of-the-day to appear
echo '[ ! -z "$TERM" -a -r /etc/motd ] && cat /etc/motd' >> /etc/environment;

# Set up the message-of-the-day
cat << 'EOF' > /etc/motd
===================================================================
=     Welcome to the RadixDLT babylon-node docker container       =
===================================================================

You will find the java application in `/opt/radixdlt/bin/core`
and the rust library in `/usr/lib/jni/libcorerust.so`

To reduce image size and attack surface, various utilities are no
longer pre-installed. You can install them by running the script at
`/opt/radixdlt/install_network_utils.sh`

EOF
