# SPDX-License-Identifier: Apache-2.0

# Recipes for the Solidity IBC implementation. Invoked from the repository
# root justfile as the `solidity` module; recipes run with `ibc-solidity/`
# as the working directory.

# Loads the repository root .env (the search walks up parent directories).
set dotenv-load

# Detect which cargo-prove command is available for building SP1 programs
prove_cmd := `command -v cargo-prove >/dev/null 2>&1 && echo "cargo-prove" || echo "~/.sp1/bin/cargo-prove"`
private_cluster := if env("E2E_PRIVATE_CLUSTER", "") == "true" { "--private-cluster" } else { "" }

# List the recipes exposed by this module
default:
	just --list solidity

# Build the contracts using `forge build`
[group('build')]
build-contracts: clean-foundry
	forge build

# Build the operator using `cargo build`
[group('build')]
build-operator:
	cargo build --bin operator --release --locked

# Build RISC-V ELF files using `cargo prove`
[group('build')]
build-sp1-programs:
	@echo "Building SP1 programs in 'programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/'"
	cd programs/sp1-programs && {{prove_cmd}} prove build -p sp1-ics07-tendermint-update-client --locked
	cd programs/sp1-programs && {{prove_cmd}} prove build -p sp1-ics07-tendermint-membership --locked
	cd programs/sp1-programs && {{prove_cmd}} prove build -p sp1-ics07-tendermint-uc-and-membership --locked
	cd programs/sp1-programs && {{prove_cmd}} prove build -p sp1-ics07-tendermint-misbehaviour --locked

# Build and optimize the Ethereum WASM light client using Docker and gzip
[group('build')]
build-cw-ics08-wasm-eth:
	-@docker image rm cosmwasm-builder:latest
	cd programs/cw-ics08-wasm-eth && docker buildx build --platform linux/amd64 -t cosmwasm-builder:latest .
	docker run --rm --platform=linux/amd64 -t \
		-v "$PWD/..":/code \
		cosmwasm-builder:latest
	cp ../artifacts/cw_ics08_wasm_eth.wasm ../e2e/interchaintestv8/wasm
	gzip -n ../e2e/interchaintestv8/wasm/cw_ics08_wasm_eth.wasm -f

# Install the SP1 Tendermint operator for use in e2e tests
[group('install')]
install-operator:
	cargo install --bin operator --path programs/operator --locked --force

# Install the root-workspace proof API used by fixture generators
_install-proof-api:
	just install-proof-api

# Lint the Solidity code using `forge fmt`, `bun:solhint` and `natlint`
[group('lint')]
lint-contracts:
	@echo "Linting the Solidity code..."
	forge fmt --check
	bun solhint -w 0 '{scripts,contracts,test}/**/*.sol'
	natlint run --root "$PWD" --include 'contracts/**/*.sol' --exclude 'contracts/utils/TrieProof.sol'

# Lint the SP1 programs using `cargo fmt` and `cargo clippy`
[group('lint')]
lint-sp1:
	@echo "Linting the SP1 programs..."
	cd programs/sp1-programs && cargo fmt --all -- --check
	cd programs/sp1-programs && cargo clippy --all-targets --all-features -- -D warnings

# Lint the CosmWasm Ethereum light client workspace
[group('lint')]
lint-cw:
	@echo "Linting the cw-ics08-wasm-eth contract..."
	cd programs/cw-ics08-wasm-eth && cargo fmt --all -- --check
	cd programs/cw-ics08-wasm-eth && cargo clippy --all-targets --locked -- -D warnings

shadowfork := if env("ETH_RPC_URL", "") == "" { "--no-match-path test/shadowfork/*" } else { "" }

# Run all the foundry tests
[group('test')]
test-foundry testname=".\\*":
	forge test -vvv --show-progress --fuzz-runs 5000 --match-test ^{{testname}}\(.\*\)\$ {{shadowfork}}
	@ {{ if shadowfork == "" { "" } else { 'echo ' + BOLD + YELLOW + 'Ran without shadowfork tests since ETH_RPC_URL was not set' } }}

# Run the CosmWasm Ethereum light client tests
[group('test')]
test-cargo-cw:
	cd programs/cw-ics08-wasm-eth && cargo test --locked --no-fail-fast -- --nocapture

# Run the benchmark tests and update the committed snapshots and Markdown tables
[group('test')]
benchmark-update:
	bun run benchmarks:update

# Verify that the committed benchmark snapshots and Markdown tables are current
[group('test')]
benchmark-check:
	bun run benchmarks:check

# Generate the WASM light-client fixtures using the e2e tests
[group('generate')]
generate-fixtures-wasm: clean-foundry _install-proof-api
	@echo "Generating fixtures... This may take a while."
	@echo "Generating recvPacket and acknowledgePacket groth16 fixtures..."
	cd ../e2e/interchaintestv8 && ETH_TESTNET_TYPE=pos GENERATE_WASM_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 60m
	@echo "Generating native SdkCoin recvPacket groth16 fixtures..."
	cd ../e2e/interchaintestv8 && ETH_TESTNET_TYPE=pos GENERATE_WASM_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferNativeCosmosCoinsToEthereumAndBack$' -timeout 60m
	@echo "Generating timeoutPacket groth16 fixtures..."
	cd ../e2e/interchaintestv8 && ETH_TESTNET_TYPE=pos GENERATE_WASM_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_TimeoutPacketFromCosmos$' -timeout 60m
	@echo "Generating multi-period client update fixtures..."
	cd ../e2e/interchaintestv8 && ETH_TESTNET_TYPE=pos GENERATE_WASM_FIXTURES=true go test -v -run '^TestWithProofAPITestSuite/Test_MultiPeriodClientUpdateToCosmos$' -timeout 60m

# Generate the Tendermint light-client fixtures using the e2e tests
[group('generate')]
generate-fixtures-tendermint-light-client: _install-proof-api
	@echo "Generating Tendermint light client fixtures... This may take a while."
	@echo "Generating basic membership and update client fixtures..."
	cd ../e2e/interchaintestv8 && GENERATE_TENDERMINT_LIGHT_CLIENT_FIXTURES=true go test -v -run '^TestWithCosmosProofAPITestSuite/Test_UpdateClient$' -timeout 40m

# Generate the Besu QBFT light-client fixtures using the focused e2e test
[group('generate')]
generate-fixtures-besu: clean-foundry _install-proof-api
	@echo "Generating Besu QBFT light client fixtures... This may take a while."
	cd ../e2e/interchaintestv8 && GENERATE_BESU_LIGHT_CLIENT_FIXTURES=true go test -v -run '^TestWithBesuToBesuTestSuite/Test_ICS20TransferERC20FromChainAToChainB$' -timeout 120m
	@echo "Fixtures generated at 'ibc-solidity/test/besu-bft/fixtures'"

# Generate Go types for the e2e tests from the Ethereum light-client schema
[group('generate')]
generate-ethereum-types:
	cargo run --bin generate_json_schema --features test-utils
	bun quicktype --src-lang schema --lang go --just-types-and-package --package ethereum --src ethereum_types_schema.json --out ../e2e/interchaintestv8/types/ethereum/types.gen.go --top-level GeneratedTypes
	rm ethereum_types_schema.json
	sed -i.bak 's/int64/uint64/g' ../e2e/interchaintestv8/types/ethereum/types.gen.go # quicktype generates int64 instead of uint64 :(
	rm -f ../e2e/interchaintestv8/types/ethereum/types.gen.go.bak # this is to be linux and mac compatible (coming from the sed command)
	cd ../e2e/interchaintestv8 && golangci-lint run --fix types/ethereum/types.gen.go

# Generate the Solidity fixtures using the e2e tests
[group('generate')]
generate-fixtures-solidity: clean-foundry install-operator _install-proof-api
	@echo "Generating fixtures... This may take a while."
	@echo "Generating recvPacket and acknowledgePacket groth16 fixtures..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating recvPacket and acknowledgePacket plonk fixtures..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=plonk go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating recvPacket and acknowledgePacket groth16 fixtures for 25 packets..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_25_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating recvPacket and acknowledgePacket groth16 fixtures for 50 packets..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_50_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating recvPacket and acknowledgePacket plonk fixtures for 50 packets..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=plonk go test -v -run '^TestWithIbcEurekaTestSuite/Test_50_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating native SdkCoin recvPacket groth16 fixtures..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferNativeCosmosCoinsToEthereumAndBack$' -timeout 40m
	@echo "Generating native SdkCoin recvPacket plonk fixtures..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=plonk go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferNativeCosmosCoinsToEthereumAndBack$' -timeout 40m
	@echo "Generating timeoutPacket groth16 fixtures..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_TimeoutPacketFromEth$' -timeout 40m
	@echo "Generating timeoutPacket plonk fixtures..."
	cd ../e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=plonk go test -v -run '^TestWithIbcEurekaTestSuite/Test_TimeoutPacketFromEth$' -timeout 40m

# Generate SP1 fixture files for TENDERMINT_RPC_URL
[group('generate')]
generate-fixtures-sp1-ics07: clean-foundry install-operator _install-proof-api
	@echo "Generating fixtures... This may take a while (up to 20 minutes)"
	TENDERMINT_RPC_URL="${TENDERMINT_RPC_URL%/}" && \
	CURRENT_HEIGHT=$(curl "$TENDERMINT_RPC_URL"/block | jq -r ".result.block.header.height") && \
	TRUSTED_HEIGHT=$(($CURRENT_HEIGHT-100)) && \
	TARGET_HEIGHT=$(($CURRENT_HEIGHT-10)) && \
	echo "For tendermint fixtures, trusted block: $TRUSTED_HEIGHT, target block: $TARGET_HEIGHT, from $TENDERMINT_RPC_URL" && \
	parallel --progress --shebang --ungroup -j 6 ::: \
		"RUST_LOG=info SP1_PROVER=network operator fixtures update-client --trusted-block $TRUSTED_HEIGHT --target-block $TARGET_HEIGHT -o 'test/sp1-ics07/fixtures/update_client_fixture-plonk.json' {{private_cluster}}" \
		"sleep 20 && RUST_LOG=info SP1_PROVER=network operator fixtures update-client --trusted-block $TRUSTED_HEIGHT --target-block $TARGET_HEIGHT -p groth16 -o 'test/sp1-ics07/fixtures/update_client_fixture-groth16.json' {{private_cluster}}" \
		"sleep 40 && RUST_LOG=info SP1_PROVER=network operator fixtures update-client-and-membership --key-paths clients/07-tendermint-0/clientState,clients/07-tendermint-001/clientState --trusted-block $TRUSTED_HEIGHT --target-block $TARGET_HEIGHT -o 'test/sp1-ics07/fixtures/uc_and_memberships_fixture-plonk.json' {{private_cluster}}" \
		"sleep 60 && RUST_LOG=info SP1_PROVER=network operator fixtures update-client-and-membership --key-paths clients/07-tendermint-0/clientState,clients/07-tendermint-001/clientState --trusted-block $TRUSTED_HEIGHT --target-block $TARGET_HEIGHT -p groth16 -o 'test/sp1-ics07/fixtures/uc_and_memberships_fixture-groth16.json' {{private_cluster}}" \
		"sleep 80 && RUST_LOG=info SP1_PROVER=network operator fixtures membership --key-paths clients/07-tendermint-0/clientState,clients/07-tendermint-001/clientState --trusted-block $TRUSTED_HEIGHT -o 'test/sp1-ics07/fixtures/memberships_fixture-plonk.json' {{private_cluster}}" \
		"sleep 100 && RUST_LOG=info SP1_PROVER=network operator fixtures membership --key-paths clients/07-tendermint-0/clientState,clients/07-tendermint-001/clientState --trusted-block $TRUSTED_HEIGHT -p groth16 -o 'test/sp1-ics07/fixtures/memberships_fixture-groth16.json' {{private_cluster}}"
	cd ../e2e/interchaintestv8 && RUST_LOG=info SP1_PROVER=network GENERATE_SOLIDITY_FIXTURES=true E2E_PROOF_TYPE=plonk go test -v -run '^TestWithSP1ICS07TendermintTestSuite/Test_DoubleSignMisbehaviour$' -timeout 40m
	cd ../e2e/interchaintestv8 && RUST_LOG=info SP1_PROVER=network GENERATE_SOLIDITY_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithSP1ICS07TendermintTestSuite/Test_BreakingTimeMonotonicityMisbehaviour' -timeout 40m
	cd ../e2e/interchaintestv8 && RUST_LOG=info SP1_PROVER=network GENERATE_SOLIDITY_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithSP1ICS07TendermintTestSuite/Test_100_Membership' -timeout 40m
	cd ../e2e/interchaintestv8 && RUST_LOG=info SP1_PROVER=network GENERATE_SOLIDITY_FIXTURES=true E2E_PROOF_TYPE=plonk go test -v -run '^TestWithSP1ICS07TendermintTestSuite/Test_25_Membership' -timeout 40m
	@echo "Fixtures generated at 'ibc-solidity/test/sp1-ics07/fixtures'"

# Generate all tracked contract artifacts and Go abigen bindings
[group('generate')]
generate-abi: build-contracts
	mkdir -p tmp/abigen
	jq '.abi' out/ICS26Router.sol/ICS26Router.json > abi/ICS26Router.json
	jq -r '.bytecode.object' out/ICS26Router.sol/ICS26Router.json > tmp/abigen/ICS26Router.bin
	jq '.abi' out/ICS20Transfer.sol/ICS20Transfer.json > abi/ICS20Transfer.json
	jq -r '.bytecode.object' out/ICS20Transfer.sol/ICS20Transfer.json > tmp/abigen/ICS20Transfer.bin
	jq '.abi' out/SP1ICS07Tendermint.sol/SP1ICS07Tendermint.json > abi/SP1ICS07Tendermint.json
	jq -r '.bytecode.object' out/SP1ICS07Tendermint.sol/SP1ICS07Tendermint.json > tmp/abigen/SP1ICS07Tendermint.bin
	jq '.abi' out/ERC20.sol/ERC20.json > abi/ERC20.json
	jq -r '.bytecode.object' out/ERC20.sol/ERC20.json > tmp/abigen/ERC20.bin
	jq '.abi' out/IBCERC20.sol/IBCERC20.json > abi/IBCERC20.json
	jq -r '.bytecode.object' out/IBCERC20.sol/IBCERC20.json > tmp/abigen/IBCERC20.bin
	jq '.abi' out/ICS27Account.sol/ICS27Account.json > abi/ICS27Account.json
	jq -r '.bytecode.object' out/ICS27Account.sol/ICS27Account.json > tmp/abigen/ICS27Account.bin
	jq '.abi' out/ICS27GMP.sol/ICS27GMP.json > abi/ICS27GMP.json
	jq -r '.bytecode.object' out/ICS27GMP.sol/ICS27GMP.json > tmp/abigen/ICS27GMP.bin
	jq '.abi' out/RelayerHelper.sol/RelayerHelper.json > abi/RelayerHelper.json
	jq -r '.bytecode.object' out/RelayerHelper.sol/RelayerHelper.json > tmp/abigen/RelayerHelper.bin
	jq '.abi' out/AttestationLightClient.sol/AttestationLightClient.json > abi/AttestationLightClient.json
	jq -r '.bytecode.object' out/AttestationLightClient.sol/AttestationLightClient.json > tmp/abigen/AttestationLightClient.bin
	jq '.abi' out/BesuIBFT2LightClient.sol/BesuIBFT2LightClient.json > abi/BesuIBFT2LightClient.json
	jq '.abi' out/BesuQBFTLightClient.sol/BesuQBFTLightClient.json > abi/BesuQBFTLightClient.json
	jq '.abi' out/IFTOwnable.sol/IFTOwnable.json > abi/IFTOwnable.json
	jq -r '.bytecode.object' out/IFTOwnable.sol/IFTOwnable.json > tmp/abigen/IFTOwnable.bin
	jq '.abi' out/ERC1967Proxy.sol/ERC1967Proxy.json > abi/ERC1967Proxy.json
	jq -r '.bytecode.object' out/ERC1967Proxy.sol/ERC1967Proxy.json > tmp/abigen/ERC1967Proxy.bin
	jq '.abi' out/EVMIFTSendCallConstructor.sol/EVMIFTSendCallConstructor.json > abi/EVMIFTSendCallConstructor.json
	jq -r '.bytecode.object' out/EVMIFTSendCallConstructor.sol/EVMIFTSendCallConstructor.json > tmp/abigen/EVMIFTSendCallConstructor.bin
	abigen --abi abi/ERC20.json --bin tmp/abigen/ERC20.bin --pkg erc20 --type Contract --out ../e2e/interchaintestv8/types/erc20/contract.go
	abigen --abi abi/IFTOwnable.json --bin tmp/abigen/IFTOwnable.bin --pkg ift --type Contract --out ../packages/go-abigen/ift/contract.go
	abigen --abi abi/SP1ICS07Tendermint.json --bin tmp/abigen/SP1ICS07Tendermint.bin --pkg sp1ics07tendermint --type Contract --out ../packages/go-abigen/sp1ics07tendermint/contract.go
	abigen --abi abi/ICS20Transfer.json --bin tmp/abigen/ICS20Transfer.bin --pkg ics20transfer --type Contract --out ../packages/go-abigen/ics20transfer/contract.go
	abigen --abi abi/ICS26Router.json --bin tmp/abigen/ICS26Router.bin --pkg ics26router --type Contract --out ../packages/go-abigen/ics26router/contract.go
	abigen --abi abi/IBCERC20.json --bin tmp/abigen/IBCERC20.bin --pkg ibcerc20 --type Contract --out ../packages/go-abigen/ibcerc20/contract.go
	abigen --abi abi/ICS27Account.json --bin tmp/abigen/ICS27Account.bin --pkg ics27account --type Contract --out ../packages/go-abigen/ics27account/contract.go
	abigen --abi abi/ICS27GMP.json --bin tmp/abigen/ICS27GMP.bin --pkg ics27gmp --type Contract --out ../packages/go-abigen/ics27gmp/contract.go
	abigen --abi abi/RelayerHelper.json --bin tmp/abigen/RelayerHelper.bin --pkg relayerhelper --type Contract --out ../packages/go-abigen/relayerhelper/contract.go
	abigen --abi abi/AttestationLightClient.json --bin tmp/abigen/AttestationLightClient.bin --pkg attestation --type Contract --out ../packages/go-abigen/attestation/contract.go
	abigen --abi abi/ERC1967Proxy.json --bin tmp/abigen/ERC1967Proxy.bin --pkg erc1967proxy --type Contract --out ../packages/go-abigen/erc1967proxy/contract.go
	abigen --abi abi/EVMIFTSendCallConstructor.json --bin tmp/abigen/EVMIFTSendCallConstructor.bin --pkg evmiftsendcall --type Contract --out ../packages/go-abigen/evmiftsendcall/contract.go
	rm -rf abi/bytecode
	mkdir -p abi/bytecode
	jq -c '{abi, bytecode: {object: .bytecode.object}}' out/SP1ICS07Tendermint.sol/SP1ICS07Tendermint.json > abi/bytecode/SP1ICS07Tendermint.json
	jq -c '{abi, bytecode: {object: .bytecode.object}}' out/AttestationLightClient.sol/AttestationLightClient.json > abi/bytecode/AttestationLightClient.json
	jq -c '{abi, bytecode: {object: .bytecode.object}}' out/BesuIBFT2LightClient.sol/BesuIBFT2LightClient.json > abi/bytecode/BesuIBFT2LightClient.json
	jq -c '{abi, bytecode: {object: .bytecode.object}}' out/BesuQBFTLightClient.sol/BesuQBFTLightClient.json > abi/bytecode/BesuQBFTLightClient.json
	rm -rf tmp/abigen

# Clean up the foundry cache and out directories
[group('clean')]
clean-foundry:
	@echo "Cleaning up cache and out directories"
	-rm -rf cache out broadcast # ignore errors

# Clean the SP1 programs workspace
[group('clean')]
clean-sp1:
	cd programs/sp1-programs && cargo clean

# Run Slither static analysis on contracts
[group('security')]
slither:
	@echo "Running Slither static analysis..."
	slither . --config-file .slither.config.json

# Build and package pre-compiled contract artifacts as a release tarball
[group('build')]
package-contracts:
	./scripts/package-contracts.sh
