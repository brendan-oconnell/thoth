THOTH_GRAPHQL_API ?= http://localhost:8000
THOTH_EXPORT_API ?= http://localhost:8181

.PHONY: \
	build-graphql-api \
	build-export-api \
	build-app \
	run-app \
	run-graphql-api \
	run-export-api \
	docker-dev \
	docker-dev-build \
	docker-dev-run \
	docker-dev-db \
	test \
	clippy \
	format \
	check-format \
	check \
	check-all \

all: build-graphql-api build-export-api build-app
check-all: test check clippy check-format

run-app: build-app
	RUST_BACKTRACE=1 cargo run start app

run-graphql-api: build-graphql-api
	RUST_BACKTRACE=1 cargo run init

run-export-api: build-export-api
	RUST_BACKTRACE=1 cargo run start export-api

docker-dev: docker-dev-build docker-dev-run

docker-dev-build:
	docker-compose -f docker-compose.dev.yml build

docker-dev-run:
	docker-compose -f docker-compose.dev.yml up

docker-dev-db:
	docker-compose -f docker-compose.dev.yml up db

cargo-build:
	cargo build

build-graphql-api: cargo-build

build-export-api: cargo-build

build-app: cargo-build

test:
	THOTH_GRAPHQL_API=$(THOTH_GRAPHQL_API) \
	THOTH_EXPORT_API=$(THOTH_EXPORT_API) \
	cargo test --workspace

clippy:
	THOTH_GRAPHQL_API=$(THOTH_GRAPHQL_API) \
	THOTH_EXPORT_API=$(THOTH_EXPORT_API) \
	cargo clippy --all --all-targets --all-features -- -D warnings

format:
	cargo fmt --all --

check-format:
	cargo fmt --all -- --check

check:
	THOTH_GRAPHQL_API=$(THOTH_GRAPHQL_API) \
	THOTH_EXPORT_API=$(THOTH_EXPORT_API) \
	cargo check --workspace
