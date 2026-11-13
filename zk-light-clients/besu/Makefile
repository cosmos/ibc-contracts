# Build once; run the prover and deployment tools against the mounted outputs.
SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := help
# Keep installation, proving and Docker jobs serial even with make -j.
.NOTPARALLEL:

export PATH := $(HOME)/.sp1/bin:$(HOME)/.foundry/bin:$(HOME)/.cargo/bin:$(PATH)
export SP1_VERSION := 6.1.0
export FOUNDRY_VERSION := v1.8.1
export RUST_VERSION := $(shell sed -n 's/^channel = "\([^"]*\)"/\1/p' rust-toolchain.toml)
SP1_PROVER ?= cpu
export SP1_PROVER
PYTHON ?= $(if $(wildcard .venv/bin/python),$(CURDIR)/.venv/bin/python,python3)
# Export supplied preparation/runtime settings, without introducing empty legacy
# profile variables into the strict deployment validator's environment.
PROFILE_INPUTS := IBC_CONFIG IBC_HOME SOURCE_CHAIN_ID DESTINATION_CHAIN_ID PROFILE CLIENT_ID COUNTERPARTY_CLIENT_ID \
 TRUSTING_PERIOD_SECONDS MAX_CLOCK_DRIFT_SECONDS SP1_VERIFIER_GATEWAY DEPLOYER_KEY_FILE \
 BOOTSTRAP_STATE_BIN CAST_BIN FORGE_BIN HEIGHT BOOTSTRAP_CONFIRMATIONS SP1_GROTH16_SELECTOR \
 DEPLOY_LOCAL_VERIFIER FORCE GAS_ESTIMATE_MULTIPLIER PYTHON LIGHT_CLIENT
$(foreach input,$(PROFILE_INPUTS),$(if $(filter undefined,$(origin $(input))),,$(eval export $(input))))

# ACTION is interpreted by the selected workflow; test/release default to all.
ACTION ?=
PROOF_API_IMAGE ?= besu-proof-api:local
export SUMMARY RELEASE_MANIFEST

.PHONY: help setup build-docker build-proof-api prepare-profile deploy test release e2e

help: ## List workflows and their actions
	@awk 'BEGIN {FS = ":.*## "; print "Usage: make <workflow> [ACTION=action] [VARIABLE=value]\n"} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-20s %s\n", $$1, $$2}' Makefile
	@printf '\nTest actions (default all): all, rust, shell, contracts, check, coverage, coverage-zk, coverage-proof-api, sp1, integration\n'
	@printf 'Release actions (default all): all, reproducible, storage-layout, manifest, metadata\n'
	@printf 'Deploy actions (default deploy): preflight, deploy, register, bootstrap\n'
	@printf '\nFull tests include real proving and external Router integration; live networks use e2e.\n'
	@printf 'Options: PROFILE, IBC_CONFIG, LIGHT_CLIENT, HEIGHT, SUMMARY, RELEASE_MANIFEST, SP1_PROVER, PYTHON, PROOF_API_IMAGE, E2E_ROUTER_DIR, E2E_ROUTER_REV\n'

setup: ## Install pinned native tools, contract dependencies and deployment Python dependencies
	bash scripts/setup.sh

build-docker: ## Build host tools, contracts, circuit ELFs and VKeys in Docker
	bash scripts/docker-build.sh all

build-proof-api: ## Compile the Linux API binary and build its mounted-output runtime image
	bash scripts/docker-build.sh proof-api
	docker build --platform linux/amd64 --tag "$(PROOF_API_IMAGE)" --file deploy/Dockerfile deploy

prepare-profile: ## Prepare light-client deployment inputs from IBC_CONFIG (run build-docker first)
	bash deploy/prepare-profile.sh

deploy: ## Light-client contract operation: PROFILE=name [ACTION=deploy|preflight|register|bootstrap]
	bash scripts/deploy.sh "$(or $(ACTION),deploy)" "$(PROFILE)"

test: ## Run all validation suites, or select ACTION (native tools required)
	bash scripts/test-workflow.sh "$(or $(ACTION),all)"

release: ## Run all release steps, or select ACTION; uses existing build outputs
	bash scripts/release-workflow.sh "$(or $(ACTION),all)"

e2e: ## Start disposable Besu networks, deploy and run real-proof scenarios
	RUN_HEAVY_E2E=1 bash scripts/tests/tools/run-e2e.sh
