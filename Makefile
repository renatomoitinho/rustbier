DOCKER_REGISTRY ?= registry.naspersclassifieds.com
DOCKER_ORG ?= shared-services/core-services

PROJECT_NAME=rustbier

REVISION ?= $(shell git rev-parse HEAD)
ifeq ($(BUILD_NUMBER),)
	VERSION_TAG ?= $(REVISION)
else
	VERSION_TAG ?= $(REVISION)-$(BUILD_NUMBER)
endif

S3_TEST_DATA=/data/apollo

.PHONY: up test build-base-image build-image

up:
	./environment/bin/setup.sh

test: up
	DOCKER_REGISTRY=$(DOCKER_REGISTRY) DOCKER_ORG=$(DOCKER_ORG) docker-compose -p $(PROJECT_NAME) \
		-f docker-compose.yaml -f docker-compose.tests.yaml \
		up --no-recreate --remove-orphans --exit-code-from cargo

build-base-image:
	docker build -f Dockerfile.base -t "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/base-rust-image:$(VERSION_TAG)" .
	docker tag "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/base-rust-image:$(VERSION_TAG)" "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/base-rust-image:latest"
	docker push "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/base-rust-image:$(VERSION_TAG)"
	docker push "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/base-rust-image:latest"

build-image:
	docker build -t "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/rustbier:$(VERSION_TAG)" .
	docker tag "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/rustbier:$(VERSION_TAG)" "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/rustbier:latest"
	docker push "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/rustbier:$(VERSION_TAG)"
	docker push "$(DOCKER_REGISTRY)/$(DOCKER_ORG)/rustbier/rustbier:latest"
