DOCKER_REGISTRY ?= registry.naspersclassifieds.com
DOCKER_ORG ?= shared-services/core-services

REVISION ?= $(shell git rev-parse --short HEAD)
ifeq ($(BUILD_NUMBER),)
	VERSION_TAG ?= $(REVISION)
else
	VERSION_TAG ?= $(REVISION)-$(BUILD_NUMBER)
endif

S3_TEST_DATA=/data/apollo

.PHONY: up setup-test test build-base-image build-image

up:
	docker-compose up -d

setup-test: up
	docker-compose exec s3 mkdir "$(S3_TEST_DATA)" || true
	docker cp tests/resources/watermark "rustbier_s3_1:$(S3_TEST_DATA)/watermark"
	docker cp tests/resources/highres "rustbier_s3_1:$(S3_TEST_DATA)/highres"
	docker cp tests/resources/img-test "rustbier_s3_1:$(S3_TEST_DATA)/img-test"

test: setup-test
	cargo test

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
