PROJECT_NAME ?= silk
BASE_VERSION = 0.0.1
EXTRA_VERSION ?= $(shell git rev-parse --short HEAD)
BUILD_DIR ?= .build

DOCKER_REGISTRY = harbor-k8s.kingdeeresearch.com

IMAGES = front chain consensus network

USERID = $(shell id -u)

PKGNAME = github.com/snlansky/$(PROJECT_NAME)

DRUN = docker run -i --rm --user=$(USERID):$(USERID) \
	-v $(abspath .):/go/src/$(PROJECT_NAME) \
	-e GOCACHE=/tmp/.cache \
	-w /go/src/$(PROJECT_NAME)

.PHONY: help dep protos docker clean start stop test cover
help:
	@echo
	@echo "帮助文档："
	@echo "  - make help              查看可用脚本"
	@echo "  - make dep               安装依赖"
	@echo "  - make protos            编译 Protobuf 协议文件"
	@echo "  - make build             编译可执行文件"
	@echo "  - make docker            编译所有 Docker 镜像"
	@echo "  - make clean             清理所有 Docker 镜像"
	@echo "  - make test              运行单元测试"
	@echo

dep:
	cargo install protobuf-codegen
	cargo install grpcio-compiler

fmt:
	cargo fix; cargo fmt; cargo clippy

build:
	cargo build --release

protos:
	$(shell ./scripts/compile_protos.sh)

test:
#	cargo test --color=always --package silk-consensus --bin silk-consensus consensus::test::tests::raft_test -- --nocapture
	cargo test  --bin silk-consensus consensus::test::tests::raft_test -- --nocapture
