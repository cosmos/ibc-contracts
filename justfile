# SPDX-License-Identifier: Apache-2.0

set dotenv-load

# Solidity IBC implementation recipes (run from the ibc-solidity directory)
mod solidity 'ibc-solidity/solidity.just'

# Solana IBC implementation recipes (run from the ibc-solana directory)
mod solana 'ibc-solana/solana.just'


# Default task lists all available tasks
default:
  just --list

# Build the proof API using `cargo build`
[group('build')]
build-proof-api:
	cargo build --bin proof-api --release --locked

# Build the solana-ibc CLI tool using `go build`
[group('build')]
build-solana-ibc:
	cd tools/solana-ibc && go build -o ../../bin/solana-ibc .

# Build the proof API docker image
[group('build')]
build-proof-api-image:
    docker build -t proof-api:latest -f programs/proof-api/Dockerfile .

# Install the proof API using `cargo install`
[group('install')]
install-proof-api:
	cargo install --bin proof-api --path programs/proof-api --locked --force

# Run all linters
[group('lint')]
lint:
	@echo "Running all linters..."
	just lint-license
	just solidity::lint-solidity
	just lint-go
	just lint-buf
	just lint-rust

# Check that every source file carries the repository's SPDX license header
[group('lint')]
lint-license:
	@echo "Checking SPDX license headers..."
	./scripts/check-license-headers.sh

# Insert or correct SPDX license headers in place
[group('lint')]
lint-license-fix:
	./scripts/check-license-headers.sh --fix

# Lint the Go code using `golangci-lint`
[group('lint')]
lint-go:
	@echo "Linting the Go code..."
	cd e2e/interchaintestv8 && golangci-lint run
	cd packages/go-abigen && golangci-lint run
	cd packages/go-anchor && golangci-lint run

# Lint the Protobuf files using `buf lint`
[group('lint')]
lint-buf:
	@echo "Linting the Protobuf files..."
	buf lint

# Lint the all the Rust code using `cargo fmt` and `cargo clippy`
[group('lint')]
lint-rust:
	@echo "Linting the Rust code..."
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	just solidity::lint-sp1
	just solidity::lint-cw
	just solana::lint-solana

# Generate the fixtures for the wasm tests using the e2e tests
[group('generate')]
generate-fixtures-wasm: solidity::clean-foundry install-proof-api
	@echo "Generating fixtures... This may take a while."
	@echo "Generating recvPacket and acknowledgePacket groth16 fixtures..."
	cd e2e/interchaintestv8 && ETH_TESTNET_TYPE=pos GENERATE_WASM_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 60m
	@echo "Generating native SdkCoin recvPacket groth16 fixtures..."
	cd e2e/interchaintestv8 && ETH_TESTNET_TYPE=pos GENERATE_WASM_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferNativeCosmosCoinsToEthereumAndBack$' -timeout 60m
	@echo "Generating timeoutPacket groth16 fixtures..."
	cd e2e/interchaintestv8 && ETH_TESTNET_TYPE=pos GENERATE_WASM_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_TimeoutPacketFromCosmos$' -timeout 60m
	@echo "Generating multi-period client update fixtures..."
	cd e2e/interchaintestv8 && ETH_TESTNET_TYPE=pos GENERATE_WASM_FIXTURES=true go test -v -run '^TestWithProofAPITestSuite/Test_MultiPeriodClientUpdateToCosmos$' -timeout 60m

# Generate the fixtures for the Tendermint light client tests using the e2e tests
[group('generate')]
generate-fixtures-tendermint-light-client: install-proof-api
	@echo "Generating Tendermint light client fixtures... This may take a while."
	@echo "Generating basic membership and update client fixtures..."
	cd e2e/interchaintestv8 && GENERATE_TENDERMINT_LIGHT_CLIENT_FIXTURES=true go test -v -run '^TestWithCosmosProofAPITestSuite/Test_UpdateClient$' -timeout 40m

# Generate the Besu QBFT light client fixtures using the focused Besu-to-Besu e2e test
[group('generate')]
generate-fixtures-besu: solidity::clean-foundry install-proof-api
	@echo "Generating Besu QBFT light client fixtures... This may take a while."
	cd e2e/interchaintestv8 && GENERATE_BESU_LIGHT_CLIENT_FIXTURES=true go test -v -run '^TestWithBesuToBesuTestSuite/Test_ICS20TransferERC20FromChainAToChainB$' -timeout 120m
	@echo "Fixtures generated at 'ibc-solidity/test/besu-bft/fixtures'"

# Generate go types for the e2e tests from the ethereum light client code
[group('generate')]
generate-ethereum-types:
	cargo run --bin generate_json_schema --features test-utils
	cd ibc-solidity && bun quicktype --src-lang schema --lang go --just-types-and-package --package ethereum --src ../ethereum_types_schema.json --out ../e2e/interchaintestv8/types/ethereum/types.gen.go --top-level GeneratedTypes
	rm ethereum_types_schema.json
	sed -i.bak 's/int64/uint64/g' e2e/interchaintestv8/types/ethereum/types.gen.go # quicktype generates int64 instead of uint64 :(
	rm -f e2e/interchaintestv8/types/ethereum/types.gen.go.bak # this is to be linux and mac compatible (coming from the sed command)
	cd e2e/interchaintestv8 && golangci-lint run --fix types/ethereum/types.gen.go

# Generate the fixtures for the Solidity tests using the e2e tests
[group('generate')]
generate-fixtures-solidity: solidity::clean-foundry solidity::install-operator install-proof-api
	@echo "Generating fixtures... This may take a while."
	@echo "Generating recvPacket and acknowledgePacket groth16 fixtures..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating recvPacket and acknowledgePacket plonk fixtures..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=plonk go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating recvPacket and acknowledgePacket groth16 fixtures for 25 packets..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_25_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating recvPacket and acknowledgePacket groth16 fixtures for 50 packets..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_50_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating recvPacket and acknowledgePacket plonk fixtures for 50 packets..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=plonk go test -v -run '^TestWithIbcEurekaTestSuite/Test_50_ICS20TransferERC20TokenfromEthereumToCosmosAndBack$' -timeout 40m
	@echo "Generating native SdkCoin recvPacket groth16 fixtures..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferNativeCosmosCoinsToEthereumAndBack$' -timeout 40m
	@echo "Generating native SdkCoin recvPacket plonk fixtures..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=plonk go test -v -run '^TestWithIbcEurekaTestSuite/Test_ICS20TransferNativeCosmosCoinsToEthereumAndBack$' -timeout 40m
	@echo "Generating timeoutPacket groth16 fixtures..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithIbcEurekaTestSuite/Test_TimeoutPacketFromEth$' -timeout 40m
	@echo "Generating timeoutPacket plonk fixtures..."
	cd e2e/interchaintestv8 && GENERATE_SOLIDITY_FIXTURES=true SP1_PROVER=network E2E_PROOF_TYPE=plonk go test -v -run '^TestWithIbcEurekaTestSuite/Test_TimeoutPacketFromEth$' -timeout 40m

private_cluster := if env("E2E_PRIVATE_CLUSTER", "") == "true" { "--private-cluster" } else { "" }

# Generate the fixture files for `TENDERMINT_RPC_URL` using the prover parameter.
[group('generate')]
generate-fixtures-sp1-ics07: solidity::clean-foundry solidity::install-operator install-proof-api
  @echo "Generating fixtures... This may take a while (up to 20 minutes)"
  TENDERMINT_RPC_URL="${TENDERMINT_RPC_URL%/}" && \
  CURRENT_HEIGHT=$(curl "$TENDERMINT_RPC_URL"/block | jq -r ".result.block.header.height") && \
  TRUSTED_HEIGHT=$(($CURRENT_HEIGHT-100)) && \
  TARGET_HEIGHT=$(($CURRENT_HEIGHT-10)) && \
  echo "For tendermint fixtures, trusted block: $TRUSTED_HEIGHT, target block: $TARGET_HEIGHT, from $TENDERMINT_RPC_URL" && \
  parallel --progress --shebang --ungroup -j 6 ::: \
    "RUST_LOG=info SP1_PROVER=network operator fixtures update-client --trusted-block $TRUSTED_HEIGHT --target-block $TARGET_HEIGHT -o 'ibc-solidity/test/sp1-ics07/fixtures/update_client_fixture-plonk.json' {{private_cluster}}" \
    "sleep 20 && RUST_LOG=info SP1_PROVER=network operator fixtures update-client --trusted-block $TRUSTED_HEIGHT --target-block $TARGET_HEIGHT -p groth16 -o 'ibc-solidity/test/sp1-ics07/fixtures/update_client_fixture-groth16.json' {{private_cluster}}" \
    "sleep 40 && RUST_LOG=info SP1_PROVER=network operator fixtures update-client-and-membership --key-paths clients/07-tendermint-0/clientState,clients/07-tendermint-001/clientState --trusted-block $TRUSTED_HEIGHT --target-block $TARGET_HEIGHT -o 'ibc-solidity/test/sp1-ics07/fixtures/uc_and_memberships_fixture-plonk.json' {{private_cluster}}" \
    "sleep 60 && RUST_LOG=info SP1_PROVER=network operator fixtures update-client-and-membership --key-paths clients/07-tendermint-0/clientState,clients/07-tendermint-001/clientState --trusted-block $TRUSTED_HEIGHT --target-block $TARGET_HEIGHT -p groth16 -o 'ibc-solidity/test/sp1-ics07/fixtures/uc_and_memberships_fixture-groth16.json' {{private_cluster}}" \
    "sleep 80 && RUST_LOG=info SP1_PROVER=network operator fixtures membership --key-paths clients/07-tendermint-0/clientState,clients/07-tendermint-001/clientState --trusted-block $TRUSTED_HEIGHT -o 'ibc-solidity/test/sp1-ics07/fixtures/memberships_fixture-plonk.json' {{private_cluster}}" \
    "sleep 100 && RUST_LOG=info SP1_PROVER=network operator fixtures membership --key-paths clients/07-tendermint-0/clientState,clients/07-tendermint-001/clientState --trusted-block $TRUSTED_HEIGHT -p groth16 -o 'ibc-solidity/test/sp1-ics07/fixtures/memberships_fixture-groth16.json' {{private_cluster}}"
  cd e2e/interchaintestv8 && RUST_LOG=info SP1_PROVER=network GENERATE_SOLIDITY_FIXTURES=true E2E_PROOF_TYPE=plonk go test -v -run '^TestWithSP1ICS07TendermintTestSuite/Test_DoubleSignMisbehaviour$' -timeout 40m
  cd e2e/interchaintestv8 && RUST_LOG=info SP1_PROVER=network GENERATE_SOLIDITY_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithSP1ICS07TendermintTestSuite/Test_BreakingTimeMonotonicityMisbehaviour' -timeout 40m
  cd e2e/interchaintestv8 && RUST_LOG=info SP1_PROVER=network GENERATE_SOLIDITY_FIXTURES=true E2E_PROOF_TYPE=groth16 go test -v -run '^TestWithSP1ICS07TendermintTestSuite/Test_100_Membership' -timeout 40m
  cd e2e/interchaintestv8 && RUST_LOG=info SP1_PROVER=network GENERATE_SOLIDITY_FIXTURES=true E2E_PROOF_TYPE=plonk go test -v -run '^TestWithSP1ICS07TendermintTestSuite/Test_25_Membership' -timeout 40m
  @echo "Fixtures generated at 'ibc-solidity/test/sp1-ics07/fixtures'"

# Generate the code from protobuf using `buf generate`
[group('generate')]
generate-buf:
    @echo "Generating Protobuf files"
    buf generate --template buf.gen.yaml

# Run the cargo tests
[group('test')]
test-cargo testname="--all":
	cargo test {{testname}} --locked --no-fail-fast -- --nocapture
	just solidity::test-cargo-cw

# Run the tests in abigen
[group('test')]
test-abigen:
	@echo "Running abigen tests..."
	cd packages/go-abigen && go test -v ./...

# Run any e2e test using the test's full name. For example, `just test-e2e TestWithIbcEurekaTestSuite/Test_Deploy`
[group('test')]
test-e2e testname: solidity::clean-foundry install-proof-api
	@echo "Running {{testname}} test..."
	cd e2e/interchaintestv8 && go test -v -run '^{{testname}}$' -timeout 120m

# Run any e2e test in the IbcEurekaTestSuite. For example, `just test-e2e-eureka Test_Deploy`
[group('test')]
test-e2e-eureka testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithIbcEurekaTestSuite/{{testname}}

# Run any e2e test in the ProofAPITestSuite. For example, `just test-e2e-proof-api Test_ProofAPIInfo`
[group('test')]
test-e2e-proof-api testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithProofAPITestSuite/{{testname}}

# Run any e2e test in the CosmosProofAPITestSuite. For example, `just test-e2e-cosmos-proof-api Test_ProofAPIInfo`
[group('test')]
test-e2e-cosmos-proof-api testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithCosmosProofAPITestSuite/{{testname}}

# Run anu e2e test in the SP1ICS07TendermintTestSuite. For example, `just test-e2e-sp1-ics07 Test_Deploy`
[group('test')]
test-e2e-sp1-ics07 testname: solidity::install-operator
	@echo "Running {{testname}} test..."
	just test-e2e TestWithSP1ICS07TendermintTestSuite/{{testname}}

# Run any e2e test in the MultichainTestSuite. For example, `just test-e2e-multichain Test_Deploy`
[group('test')]
test-e2e-multichain testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithMultichainTestSuite/{{testname}}

# Run any e2e test in the IbcEurekaGmpTestSuite. For example, `just test-e2e-multichain TestDeploy_Groth16`
[group('test')]
test-e2e-gmp testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithIbcEurekaGmpTestSuite/{{testname}}

# Run the e2e tests in the EthToEthAttestedTestSuite. For example, `just test-e2e-eth-to-eth Test_Deploy`
[group('test')]
test-e2e-eth-to-eth testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithEthToEthAttestedTestSuite/{{testname}}

# Run the e2e tests in the MultiAttestorTestSuite. For example, `just test-e2e-multi-attestor Test_MultiAttestorDeploy`
[group('test')]
test-e2e-multi-attestor testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithMultiAttestorTestSuite/{{testname}}

# Run the e2e tests in the IbcEurekaSolanaTestSuite. For example, `just test-e2e-solana Test_Deploy`
[group('test')]
test-e2e-solana testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithIbcEurekaSolanaTestSuite/{{testname}}

# Run the e2e tests in the IbcEurekaSolanaGMPTestSuite. For example, `just test-e2e-solana-gmp Test_GMPSPLTokenTransferFromCosmos`
[group('test')]
test-e2e-solana-gmp testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithIbcEurekaSolanaGMPTestSuite/{{testname}}

# Run the e2e tests in the IbcEurekaSolanaIFTTestSuite. For example, `just test-e2e-solana-ift Test_IFT_CosmosToSolanaRoundtrip`
[group('test')]
test-e2e-solana-ift testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithIbcEurekaSolanaIFTTestSuite/{{testname}}

# Run the e2e tests in the IbcEurekaSolanaUpgradeTestSuite. For example, `just test-e2e-solana-upgrade Test_ProgramUpgrade_Via_AccessManager`
[group('test')]
test-e2e-solana-upgrade testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithIbcEurekaSolanaUpgradeTestSuite/{{testname}}

# Run the e2e tests in the CosmosIFTTestSuite. For example, `just test-e2e-cosmos-ift Test_IFTTransfer`
[group('test')]
test-e2e-cosmos-ift testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithCosmosIFTTestSuite/{{testname}}

# Run the e2e tests in the CosmosEthereumIFTTestSuite. For example, `just test-e2e-cosmos-ethereum-ift Test_Deploy`
[group('test')]
test-e2e-cosmos-ethereum-ift testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithCosmosEthereumIFTTestSuite/{{testname}}

# Run the e2e tests in the EthereumSolanaIFTTestSuite. For example, `just test-e2e-ethereum-solana-ift Test_EthSolana_IFT_Roundtrip`
[group('test')]
test-e2e-ethereum-solana-ift testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithEthereumSolanaIFTTestSuite/{{testname}}

# Run the e2e tests in the IbcSolanaAttestationTestSuite. For example, `just test-e2e-solana-attestation Test_Attestation_Deploy`
[group('test')]
test-e2e-solana-attestation testname:
	@echo "Running {{testname}} test..."
	just test-e2e TestWithIbcSolanaAttestationTestSuite/{{testname}}


# Clean up the cargo artifacts using `cargo clean`
[group('clean')]
clean-cargo:
	@echo "Cleaning up cargo target directory"
	cargo clean
	just solidity::clean-sp1

# Compute IFT contract address and ICA address from deployer private key
# Example: just compute-ift-addresses ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 18 08-wasm-0 wf
[group('tools')]
compute-ift-addresses private-key nonce client-id bech32-prefix salt="":
	@cd tools/compute-ift-addresses && go run . {{private-key}} {{nonce}} {{client-id}} {{bech32-prefix}} {{salt}}
