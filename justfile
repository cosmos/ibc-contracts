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
	just solidity::lint-contracts
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
test-e2e testname:
	just solidity::clean-foundry
	just install-proof-api
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
test-e2e-sp1-ics07 testname:
	just solidity::install-operator
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
