REGISTRY ?= docker.io/radixdlt


all:
    $(eval GIT_BRANCH=$(shell git rev-parse --abbrev-ref HEAD | sed 's/\//-/g'))
    $(eval GIT_COMMIT=$(shell git log -1 --format=%h ))
    TAG ?= $(GIT_BRANCH)-$(GIT_COMMIT)
    CORE_REPO ?= $(REGISTRY)/private-babylon-node
	GOOGLE_ARTIFICAT_REGISTRY ?= europe-west2-docker.pkg.dev
    GAR_CACHE_REPO ?= $(GOOGLE_ARTIFICAT_REGISTRY)/dev-container-repo/eu-cache-repo

.PHONY: gcloud-authenticate
gcloud-authenticate:
	 export CLOUDSDK_PYTHON_SITEPACKAGES=1
	 echo "Authenticating service  account "
	 gcloud auth activate-service-account dev-container-uploader@radixdlt.iam.gserviceaccount.com --key-file=$(GC_DEV_REPO_KEY)
	 gcloud auth list

.PHONY: dockerhub-authenticate
dockerhub-authenticate:
	 echo "Authenticating to dockerhub account "
	 echo -n "${DOCKERHUB_TOKEN}" | docker login -u radixdevops --password-stdin

.PHONY: gcloud-token
gcloud-token: gcloud-authenticate
	 echo "Getting access token"
	 $(eval ACCESSTOKEN=$(shell gcloud auth print-access-token))
	 @docker login -u oauth2accesstoken -p $(ACCESSTOKEN) https://eu.gcr.io/
	 @docker login -u oauth2accesstoken -p $(ACCESSTOKEN) https://europe-west2-docker.pkg.dev/

.PHONY: build-sm
build-sm: gcloud-token
	cd .. && SKIP_NATIVE_RUST_BUILD=TRUE  ./gradlew :core-rust:buildRustForDocker -Pci=true -PcacheBranch=$(GIT_BRANCH) -PrustBinaryBuildType=release

.PHONY: build
build-core:
	cd .. && SKIP_NATIVE_RUST_BUILD=TRUE ./gradlew deb4docker -Pci=true -PcacheBranch=$(GIT_BRANCH) -PrustBinaryBuildType=release

.PHONY: package
package: build-core
	docker compose -f ../docker/network-size-1.yml build
	docker tag radixdlt/babylon-node:main $(CORE_REPO):$(TAG)

.PHONY: publish
publish: package
	docker push $(CORE_REPO):$(TAG)

.PHONY: buildx-driver
buildx-driver:
	docker buildx create --use

.PHONY: build-push-using-local-cache
build-push-using-local-cache:  buildx-driver dockerhub-authenticate build-core
	echo "Building and pushing image $(CORE_REPO):$(TAG)"
# Use local cache instead remote cache.
# Remote cache has shown to be too expensive in the long run.
# Might be reintroduced with private DockerHub Images if sufficient image-pulls are available
	$(shell cd ../ && DOCKER_BUILDKIT=1 docker buildx build \
		  --cache-from=type=local,src=/tmp/outputs/cache/docker \
		  --cache-to=type=local,dest=/tmp/outputs/cache/docker,mode=min \
          --tag $(CORE_REPO):$(TAG) -f Dockerfile . --push)

.PHONY: build-push-using-remote-cache
build-push-using-remote-cache:  buildx-driver gcloud-token dockerhub-authenticate build-core
	echo "Building and pushing image $(CORE_REPO):$(TAG)"
	$(shell cd ../ && DOCKER_BUILDKIT=1 docker buildx build \
          --cache-from=type=registry,ref=$(GAR_CACHE_REPO)/build-core-cache \
          --cache-to=type=registry,ref=$(GAR_CACHE_REPO)/build-core-cache,mode=min \
          --tag $(CORE_REPO):$(TAG) -f Dockerfile . --push)

.PHONY: run-docker-node
run-docker-node:
	../docker/scripts/rundocker.sh 2