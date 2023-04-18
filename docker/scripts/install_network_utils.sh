#!/bin/bash
apt-get -y update

apt-get -y --no-install-recommends install \
    apt-utils \
    attr \
    curl \
    daemontools \
    gettext-base \
    iproute2 \
    iptables \
    libssl-dev \
    net-tools \
    pkg-config \
    software-properties-common \
    strace \
    tcpdump 

apt-get clean
rm -rf /var/lib/apt/lists/* /var/tmp/* /tmp/*