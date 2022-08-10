REGISTRY ?= eu.gcr.io/dev-container-repo


all:
    $(eval GIT_BRANCH=$(shell git rev-parse --abbrev-ref HEAD | sed 's/\//-/g'))
    $(eval GIT_COMMIT=$(shell git log -1 --format=%h ))
    TAG ?= $(GIT_BRANCH)-$(GIT_COMMIT)
    CORE_REPO ?= $(REGISTRY)/radixdlt-core
    GOOGLE_ARTIFICAT_REGISTRY ?= europe-west2-docker.pkg.dev
    GAR_CACHE_REPO ?= $(GOOGLE_ARTIFICAT_REGISTRY)/dev-container-repo/eu-cache-repo

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
	 cat $(GC_DEV_REPO_KEY) | docker login -u _json_key --password-stdin https://$(GOOGLE_ARTIFICAT_REGISTRY)
	 cat $(GC_DEV_REPO_KEY) | docker login -u _json_key --password-stdin https://eu.gcr.io

.PHONY: publish
publish: package
	docker push $(CORE_REPO):$(TAG)

.PHONY: build-push-using-remote-cache
build-push-using-remote-cache: build-core gcloud-authenticate
	echo "Building and pushing image $(CORE_REPO):$(TAG)"
	$(shell cd ../docker && DOCKER_BUILDKIT=1 docker buildx build \
          --cache-from=type=registry,ref=$(GAR_CACHE_REPO)/build-core-cache \
          --cache-to=type=registry,ref=$(GAR_CACHE_REPO)/build-core-cache,mode=min \
          --tag $(CORE_REPO):$(TAG) -f Dockerfile.core . --push)

.PHONY: run-docker-node
run-docker-node: package
	../docker/scripts/rundocker.sh 2