REGISTRY ?= eu.gcr.io/dev-container-repo


all:
    $(eval GIT_BRANCH=$(shell git rev-parse --abbrev-ref HEAD | sed 's/\//-/g'))
    $(eval GIT_COMMIT=$(shell git log -1 --format=%h ))
    TAG ?= $(GIT_BRANCH)-$(GIT_COMMIT)
    CORE_REPO ?= $(REGISTRY)/babylon-node
    GOOGLE_ARTIFICAT_REGISTRY ?= europe-west2-docker.pkg.dev
    GAR_CACHE_REPO ?= $(GOOGLE_ARTIFICAT_REGISTRY)/dev-container-repo/eu-cache-repo


.PHONY: gcloud-authenticate
gcloud-authenticate:
	 export CLOUDSDK_PYTHON_SITEPACKAGES=1
	 echo "Authenticating service  account "
	 gcloud auth activate-service-account dev-container-uploader@dev-container-repo.iam.gserviceaccount.com --key-file=$(GC_DEV_REPO_KEY)
	 gcloud auth list


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
	docker compose -f ../docker/node-1.yml build
	docker tag radixdlt/radixdlt-core:main $(CORE_REPO):$(TAG)


.PHONY: publish
publish: package
	docker push $(CORE_REPO):$(TAG)

.PHONY: buildx-driver
buildx-driver:
	docker buildx create --use



.PHONY: build-push-using-remote-cache
build-push-using-remote-cache:  buildx-driver gcloud-token build-core
	echo "Building and pushing image $(CORE_REPO):$(TAG)"
	$(shell cd ../docker && DOCKER_BUILDKIT=1 docker buildx build \
          --cache-from=type=registry,ref=$(GAR_CACHE_REPO)/build-core-cache \
          --cache-to=type=registry,ref=$(GAR_CACHE_REPO)/build-core-cache,mode=min \
          --tag $(CORE_REPO):$(TAG) -f Dockerfile.core . --push)

.PHONY: run-docker-node
run-docker-node:
	../docker/scripts/rundocker.sh 2