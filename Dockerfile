ARG IMAGE_VERSION=v1.3.0

FROM radixdlt/babylon-node-build-layers:${IMAGE_VERSION}-java AS java-container

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

FROM radixdlt/babylon-node-build-layers:${IMAGE_VERSION}-rust AS rust-container

WORKDIR /app

COPY core-rust/ /app

RUN mkdir -p /artifacts && \
    /root/.cargo/bin/cargo build --profile=release && \
    cp target/release/libcorerust.so /artifacts/libcorerust.so

FROM radixdlt/babylon-node-build-layers:${IMAGE_VERSION}-app AS main

# Copy in the application artifacts
COPY --from=java-container /artifacts/*.jar /opt/radixdlt/lib/
COPY --from=java-container /artifacts/core /opt/radixdlt/bin/core
COPY --from=rust-container /artifacts/libcorerust.so /usr/lib/jni/libcorerust.so

RUN chmod +x /opt/radixdlt/bin/core
