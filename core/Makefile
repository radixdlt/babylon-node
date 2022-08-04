REGISTRY ?= eu.gcr.io/dev-container-repo

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

.PHONY: build-external-cache
build-external-cache:
	cd ..
	docker buildx build\
	  --cache-from=type=registry,ref=eu.gcr.io/dev-container-repo/build-core-cache \
	  --cache-to=type=registry,ref=eu.gcr.io/dev-container-repo/build-core-cache,mode=max \
	  --tag $(CORE_REPO):$(TAG) -f docker/Dockerfile.core .

.PHONY: package
package: build-core
	docker-compose -f ../docker/node-1.yml build
	docker tag radixdlt/radixdlt-core:main $(CORE_REPO):$(TAG)

.PHONY: publish
publish: package
	docker push $(CORE_REPO):$(TAG)

.PHONY: run-docker-node
run-docker-node: package
	../docker/scripts/rundocker.sh