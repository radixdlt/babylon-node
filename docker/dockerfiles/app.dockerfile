# =================================================================================================
# BUILD ARGS
#
# For the standard build:
# - TARGETPLATFORM - provided automatically, specified via `docker build --platform xyz`
#
# =================================================================================================
# LAYER: app-container
# The application container which will actually run the application
# =================================================================================================
FROM ubuntu:22.04 AS app-container

LABEL org.opencontainers.image.source="https://github.com/radixdlt/babylon-node"
LABEL org.opencontainers.image.authors="devops@radixdlt.com"

# Install dependencies needed for building the image or running the application
# - unzip is needed for unpacking the java build artifacts
# - daemontools is needed at application runtime for async tasks
# - software-properties-common is needed for installing debian packages with dpkg
# - gettext-base is needed for envsubst in config_radixdlt.sh
# - curl is needed for the docker-healthcheck
#
# The installed versions are fixed to create an immutable build.
# Availability of fixed version is subject to change.
# The latest available version can be found at these links.
# Update the references versions in case the build fails
# Source Repositories:
# - https://packages.debian.org/bookworm/openjdk-17-jre-headless
# - https://packages.debian.org/bookworm/curl
# - https://packages.debian.org/bookworm/gettext-base
# - https://packages.debian.org/bookworm/daemontools
# - https://packages.debian.org/bookworm/libc6
RUN apt-get update -y \
  && apt-get -y --no-install-recommends install \
    openjdk-17-jre-headless=17.0.12+7-2~deb12u1 \
    # https://security-tracker.debian.org/tracker/CVE-2023-38545
    curl=7.88.1-10+deb12u7 \
    gettext-base=0.21-12 \
    daemontools=1:0.76-8.1 \
    # https://security-tracker.debian.org/tracker/CVE-2023-4911
    # Fixes CVE-2023-4911 can be removed when we update the base OS image to include this fix
    # docker run -it debian:12.1-slim ldd --version
    # This fix can be removed as long as the version printed in the above command is 2.36-9+deb12u3 or above
    libc6=2.36-9+deb12u7 \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

COPY docker/build_scripts/fix-vulnerabilities.sh /tmp
RUN /tmp/fix-vulnerabilities.sh

# Copy configuration templates
WORKDIR /home/radixdlt
COPY docker/config/ /home/radixdlt/

# Add script to install optional network utils - but don't run it
COPY docker/build_scripts/install_network_utils.sh /opt/radixdlt/install_network_utils.sh

# Configure the welcome message when a shell is started.
# Docker defaults to using /bin/sh, so we can't use .bashrc and have to hook into ENV instead
ENV ENV=/etc/environment
COPY docker/build_scripts/configure_motd.sh /tmp
RUN /tmp/configure_motd.sh

# Add healthcheck
COPY docker/build_scripts/docker-healthcheck.sh /home/radixdlt/
RUN chmod +x /home/radixdlt/docker-healthcheck.sh
HEALTHCHECK CMD sh /home/radixdlt/docker-healthcheck.sh

# Set default environment variables
ENV RADIXDLT_HOME=/home/radixdlt \
    RADIXDLT_NETWORK_SEEDS_REMOTE=127.0.0.1 \
    RADIXDLT_DB_LOCATION=./RADIXDB \
    RADIXDLT_DB_ACCOUNT_CHANGE_INDEX_ENABLE=true \
    RADIXDLT_DB_LOCAL_TRANSACTION_EXECUTION_INDEX_ENABLE=true \
    RADIXDLT_VALIDATOR_KEY_LOCATION=/home/radixdlt/node.ks \
    RADIXDLT_NETWORK_USE_PROXY_PROTOCOL=false \
    RADIXDLT_CORE_API_PORT=3333 \
    RADIXDLT_CORE_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_SYSTEM_API_PORT=3334 \
    RADIXDLT_SYSTEM_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_PROMETHEUS_API_PORT=3335 \
    RADIXDLT_PROMETHEUS_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_ENGINE_STATE_API_PORT=3336 \
    RADIXDLT_ENGINE_STATE_API_BIND_ADDRESS=0.0.0.0 \
    RADIXDLT_NETWORK_ID=240 \
    RADIXDLT_NODE_KEY_CREATE_IF_MISSING=false

# Copy in the application artifacts
# The artifacts directory on the host must be populated with the required files.
COPY artifacts/*.jar /opt/radixdlt/lib/
COPY artifacts/core /opt/radixdlt/bin/core
COPY artifacts/libcorerust.so /usr/lib/jni/libcorerust.so

# Create configuration automatically when starting
COPY docker/build_scripts/config_radixdlt.sh /opt/radixdlt/config_radixdlt.sh

# See https://docs.docker.com/engine/reference/builder/#entrypoint
ENTRYPOINT ["/opt/radixdlt/config_radixdlt.sh"]
CMD ["/opt/radixdlt/bin/core"]
