# SPDX-License-Identifier: Apache-2.0

# Recipes for the Solidity IBC implementation. Invoked from the repository
# root justfile as the `solidity` module; recipes run with `ibc-solidity/`
# as the working directory.

# Loads the repository root .env (the search walks up parent directories).
set dotenv-load

# Build the contracts using `forge build`
[group('build')]
build-contracts: clean-foundry
	forge build

# Lint the Solidity code using `forge fmt`, `bun:solhint` and `natlint`
[group('lint')]
lint-solidity:
	@echo "Linting the Solidity code..."
	forge fmt --check
	bun solhint -w 0 '{scripts,contracts,test}/**/*.sol'
	natlint run --include 'contracts/**/*.sol'

shadowfork := if env("ETH_RPC_URL", "") == "" { "--no-match-path test/shadowfork/*" } else { "" }

# Run all the foundry tests
[group('test')]
test-foundry testname=".\\*":
	forge test -vvv --show-progress --fuzz-runs 5000 --match-test ^{{testname}}\(.\*\)\$ {{shadowfork}}
	@ {{ if shadowfork == "" { "" } else { 'echo ' + BOLD + YELLOW + 'Ran without shadowfork tests since ETH_RPC_URL was not set' } }}

# Run the benchmark tests and update the committed snapshots and Markdown tables
[group('test')]
benchmark-update:
	bun run benchmarks:update

# Verify that the committed benchmark snapshots and Markdown tables are current
[group('test')]
benchmark-check:
	bun run benchmarks:check

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

# Run Slither static analysis on contracts
[group('security')]
slither:
	@echo "Running Slither static analysis..."
	slither . --config-file .slither.config.json

# Build and package pre-compiled contract artifacts as a release tarball
[group('build')]
package-contracts:
	./scripts/package-contracts.sh
