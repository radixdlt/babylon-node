ARG IMAGE_VERSION=v1.3.0

FROM radixdlt/babylon-node-build-layers:${IMAGE_VERSION}-java AS java-build-stage

WORKDIR /radixdlt

ARG VERSION_BRANCH=""
ARG VERSION_COMMIT=""
ARG VERSION_DISPLAY=""
ARG VERSION_BUILD=""
ARG VERSION_TAG=""
ARG VERSION_LAST_TAG=""

ENV VERSION_BRANCH=$VERSION_BRANCH
ENV VERSION_COMMIT=$VERSION_COMMIT
ENV VERSION_DISPLAY=$VERSION_DISPLAY
ENV VERSION_BUILD=$VERSION_BUILD
ENV VERSION_TAG=$VERSION_TAG
ENV VERSION_LAST_TAG=$VERSION_LAST_TAG

# Copy the relevant files at the repo root
COPY \
  build.gradle \
  gradlew \
  gradlew.bat \
  settings.gradle \
  sonar-project.properties \
  gradle.properties \
  licence-header.txt \
  /radixdlt/
COPY ./gradle /radixdlt/gradle
COPY ./common /radixdlt/common
COPY ./core /radixdlt/core
COPY ./core-rust-bridge /radixdlt/core-rust-bridge
COPY ./cli-tools /radixdlt/cli-tools
COPY ./shell /radixdlt/shell
COPY ./keygen /radixdlt/keygen

RUN SKIP_NATIVE_RUST_BUILD=TRUE ./gradlew clean build -x test -Pci=true -PrustBinaryBuildType=release

RUN cd core/build/distributions && \
    unzip -j *.zip && \
    mkdir -p /artifacts && \
    cp *.jar /artifacts && \
    cp core /artifacts

FROM radixdlt/babylon-node-build-layers:${IMAGE_VERSION}-rust AS library-build-stage

WORKDIR /app

COPY core-rust/ /app

RUN mkdir -p /artifacts && \
    /root/.cargo/bin/cargo build --profile=release && \
    cp target/release/libcorerust.so /artifacts/libcorerust.so

# =================================================================================================
# LAYER: library-container
# A layer containing just the built library at the root: /libcorerust.so
# =================================================================================================
FROM scratch AS library-container
COPY --from=library-build-stage /artifacts/libcorerust.so /

# =================================================================================================
# LAYER: java-container
# Exports only the java application artifacts from the java-build-stage
# =================================================================================================
FROM scratch AS java-container
COPY --from=java-build-stage /radixdlt/core/build/distributions /

# =================================================================================================
# LAYER: library-builder-local
# This layer is just used for local building in development via `local-cached-rust-build.yaml`.
# Specifically - the Rust isn't built as part of the image, instead the CMD of the image is to do the build.
# It allows us to use volumes at runtime to cache the build dependencies and artifacts.
# =================================================================================================
FROM radixdlt/babylon-node-build-layers:${IMAGE_VERSION}-rust AS library-builder-local

WORKDIR /app

COPY docker/build_scripts/cargo_local_build.sh /opt/radixdlt/cargo_local_build.sh

COPY core-rust ./

FROM radixdlt/babylon-node-build-layers:${IMAGE_VERSION}-app AS app-container

# Copy in the application artifacts
COPY --from=java-build-stage /artifacts/*.jar /opt/radixdlt/lib/
COPY --from=java-build-stage /artifacts/core /opt/radixdlt/bin/core
COPY --from=library-build-stage /artifacts/libcorerust.so /usr/lib/jni/libcorerust.so

RUN chmod +x /opt/radixdlt/bin/core
