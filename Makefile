CARGO ?= cargo
DOCKER_COMPOSE ?= docker compose
CARGO_BUILD_JOBS ?= 4
PROFILE ?= profiles/B.json
SEED ?= 1
DELAY_MS ?= 90
DURATION_SECONDS ?= 30

.PHONY: all clean docker docker-build docker-run docker-clean

all:
	$(CARGO) build --release --bins
	cp target/release/sender ./sender
	cp target/release/receiver ./receiver

clean:
	$(CARGO) clean
	rm -f sender receiver

docker: docker-run

docker-build:
	CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" $(DOCKER_COMPOSE) build

docker-run: docker-build
	PROFILE="$(PROFILE)" SEED="$(SEED)" DELAY_MS="$(DELAY_MS)" DURATION_SECONDS="$(DURATION_SECONDS)" $(DOCKER_COMPOSE) run --rm flaky-network

docker-clean:
	$(DOCKER_COMPOSE) down --remove-orphans
