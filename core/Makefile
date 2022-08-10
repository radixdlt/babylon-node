REGISTRY ?= eu.gcr.io/dev-container-repo
GOOGLE_ARTIFICAT_REGISTRY ?= europe-west2-docker.pkg.dev/dev-container-repo/eu-cache-repo
all:
    $(eval GIT_BRANCH=$(shell git rev-parse --abbrev-ref HEAD | sed 's/\//-/g'))
    $(eval GIT_COMMIT=$(shell git log -1 --format=%h ))
    TAG ?= $(GIT_BRANCH)-$(GIT_COMMIT)
    CORE_REPO ?= $(REGISTRY)/radixdlt-core

.PHONY: build-sm
build-sm:
	cd .. && ./gradlew :state-manager:buildRustForDocker -Pci=true

.PHONY: build
build-core:
	cd .. && ./gradlew deb4docker -Pci=true


.PHONY: package
package: build-core
	docker-compose -f ../docker/node-1.yml build
	docker tag radixdlt/radixdlt-core:main $(CORE_REPO):$(TAG)

.PHONY: gcloud-authenticate
gcloud-authenticate:
	gcloud auth activate-service-account  docker-upload@lunar-arc-236318.iam.gserviceaccount.com --key-file=$(SERVICE_ACCOUNT_FILE)

.PHONY: publish
publish: package
	docker push $(CORE_REPO):$(TAG)

.PHONY: build-push-using-remote-cache
build-push-using-remote-cache: build-core
	echo "Building and pushing image $(CORE_REPO):$(TAG)"
	$(shell cd ../docker && DOCKER_BUILDKIT=1 docker buildx build \
          --cache-from=type=registry,ref=$(GOOGLE_ARTIFICAT_REGISTRY)/build-core-cache \
          --cache-to=type=registry,ref=$(GOOGLE_ARTIFICAT_REGISTRY)/build-core-cache,mode=min \
          --tag $(CORE_REPO):$(TAG) -f Dockerfile.core . --push)

.PHONY: run-docker-node
run-docker-node: package
	../docker/scripts/rundocker.sh 2