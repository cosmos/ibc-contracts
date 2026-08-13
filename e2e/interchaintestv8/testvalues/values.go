// SPDX-License-Identifier: Apache-2.0

package testvalues

import (
	"encoding/json"
	"math/big"
	"os"
	"time"

	"github.com/holiman/uint256"

	"github.com/ethereum/go-ethereum/crypto"

	"github.com/gagliardetto/solana-go"

	"cosmossdk.io/math"

	ibctm "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"

	"github.com/cosmos/interchaintest/v11/chain/ethereum"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"
)

const (
	// InitialBalance is the amount of tokens to give to each user at the start of the test.
	InitialBalance int64 = 1_000_000_000_000

	// TransferAmount is the default transfer amount
	TransferAmount int64 = 1_000_000_000

	// NumAttestors is the number of attestor instances to start in tests
	NumAttestors int = 1

	// InitialSolBalance is the default amount of SOL to give a new user
	InitialSolBalance uint64 = solana.LAMPORTS_PER_SOL * 1

	// EnvKeyTendermintRPC Tendermint RPC URL.
	EnvKeyTendermintRPC = "TENDERMINT_RPC_URL"
	// EnvKeyEthRPC Ethereum RPC URL.
	EnvKeyEthRPC = "RPC_URL"
	// EnvKeyOperatorPrivateKey Private key used to submit transactions by the operator.
	EnvKeyOperatorPrivateKey = "PRIVATE_KEY"
	// Optional address of the sp1 verifier contract to use
	// if not set, the contract will be deployed
	// Can be set to "mock" to use the mock verifier
	EnvKeyVerifier = "VERIFIER"
	// EnvKeySp1Prover The prover type (local|network|mock).
	EnvKeySp1Prover = "SP1_PROVER"
	// EnvKeyNetworkPrivateKey Private key for the sp1 prover network.
	EnvKeyNetworkPrivateKey = "NETWORK_PRIVATE_KEY"
	// EnvKeyNetworkPrivateCluster Run the network prover in a private cluster.
	EnvKeyNetworkPrivateCluster = "E2E_PRIVATE_CLUSTER"
	// EnvKeyGenerateSolidityFixtures Generate fixtures for the solidity tests if set to true.
	EnvKeyGenerateSolidityFixtures = "GENERATE_SOLIDITY_FIXTURES"
	// EnvKeyGenerateSolidityFixtures Generate fixtures for the solidity tests if set to true.
	EnvKeyGenerateWasmFixtures = "GENERATE_WASM_FIXTURES"
	// EnvKeyGenerateTendermintLightClientFixtures Generate fixtures for the tendermint light client tests if set to true.
	EnvKeyGenerateTendermintLightClientFixtures = "GENERATE_TENDERMINT_LIGHT_CLIENT_FIXTURES"
	// The log level for the Rust logger.
	EnvKeyRustLog = "RUST_LOG"

	// Enable local observability (Grafana stack under scripts/local-grafana-stack) for e2e runs.
	// When set to "true", the relayer will emit OTLP traces to http://127.0.0.1:4317 and keep metrics enabled.
	EnvKeyEnableLocalObservability = "ENABLE_LOCAL_OBSERVABILITY"

	// Log level for the Rust logger.
	EnvValueRustLog_Info = "info"
	// Enable local observability flag value
	EnvValueEnableLocalObservability_True = "true"
	// EnvValueSp1Prover_Network is the prover type for the network prover.
	EnvValueSp1Prover_Network = "network"
	// EnvValueSp1Prover_PrivateCluster is the for running the network prover in a private cluster.
	EnvValueSp1Prover_PrivateCluster = "true"
	// EnvValueSp1Prover_Mock is the prover type for the mock prover.
	EnvValueSp1Prover_Mock = "mock"
	// EnvValueVerifier_Mock is the verifier type for the mock verifier.
	EnvValueVerifier_Mock = "mock"
	// EnvValueGenerateFixtures_True is the value to set to generate fixtures for the solidity tests.
	EnvValueGenerateFixtures_True = "true"
	// EnvValueEthereumPosPreset_Minimal is the default preset for Ethereum PoS testnet.
	EnvValueEthereumPosPreset_Minimal = "minimal"
	// EnvValueProofType_Groth16 is the proof type for Groth16.
	EnvValueProofType_Groth16 = "groth16"
	// EnvValueProofType_Plonk is the proof type for Plonk.
	EnvValueProofType_Plonk = "plonk"
	// EnvValueWasmLightClientTag_Local is the value to set to use the local Wasm light client binary.
	EnvValueWasmLightClientTag_Local = "local"

	// EthTestnetTypeAnvil uses local Anvil chain (supports dummy or attestor light client)
	EthTestnetTypeAnvil = "anvil"

	// EthTestnetTypePoS uses Kurtosis for full PoS infrastructure with beacon chain
	EthTestnetTypePoS = "pos"

	// EthTestnetType_None disables Ethereum chain setup
	EthTestnetType_None = "none"

	// Dummy light client (for Eth verification on Cosmos)
	EthLCOnCosmosTypeDummyWasm = "dummy"
	// Full light client (for Eth verification on Cosmos)
	EthLCOnCosmosTypeFullWasm = "full"
	// Native ibc-go attestor light client (for Eth verification on Cosmos) - uses attestations module
	EthLCOnCosmosTypeAttestorNative = "attestor-native"

	// SP1 light client (for Cosmos verification on Ethereum)
	CosmosLcTypeSp1 = "sp1"
	// Attestor light client (for Cosmos verification on Ethereum)
	CosmosLcTypeAttestor = "attestor"

	// EnvKeyEthTestnetType The Ethereum testnet type (anvil|pos).
	EnvKeyEthTestnetType = "ETH_TESTNET_TYPE"
	// EnvKeyEthAnvilCount Number of Anvil chains to create (only when ETH_TESTNET_TYPE=anvil). Defaults to 1.
	EnvKeyEthAnvilCount = "ETH_ANVIL_COUNT"
	// EnvE2EFacuetAddress The address of the faucet
	EnvKeyE2EFacuetAddress = "E2E_FAUCET_ADDRESS"
	// EnvKeyEthereumPosNetworkPreset The environment variable name to configure the Kurtosis network preset
	EnvKeyEthereumPosNetworkPreset = "ETHEREUM_POS_NETWORK_PRESET"
	// EnvKeyE2EProofType is the environment variable name to configure the proof type. (groth16|plonk)
	// A randomly selected proof type is used if not set.
	EnvKeyE2EProofType = "E2E_PROOF_TYPE"
	// EnvKeyE2EGmpEncoding is the environment variable name to configure the GMP encoding type.
	// Accepted values: "application/x-protobuf" | "application/x-solidity-abi". A randomly selected encoding is used if not set.
	EnvKeyE2EGmpEncoding = "E2E_GMP_ENCODING"
	// EnvKeyE2EWasmLightClientTag is the environment variable name to configure the eth light client version.
	// Either an empty string, or 'local', means it will use the local binary in the repo, unless running in mock mode
	// otherwise, it will download the version from the github release with the given tag
	EnvKeyE2EWasmLightClientTag = "E2E_WASM_LIGHT_CLIENT_TAG"
	// EnvKeyEthLcOnCosmos is the environment variable name to configure the Ethereum light client
	// deployed on Cosmos (dummy|full|attestor-native)
	EnvKeyEthLcOnCosmos = "ETH_LC_ON_COSMOS"
	// EnvKeyCosmosLcOnEth is the environment variable name to configure the Cosmos light client
	// deployed on Ethereum (sp1|attestor). Defaults to sp1.
	EnvKeyCosmosLcOnEth = "COSMOS_LC_ON_ETH"

	// EnvKeyMultiAttestorCount is the total number of attestor keys to generate and register
	// in light clients as authorized signers.
	EnvKeyMultiAttestorCount = "MULTI_ATTESTOR_COUNT"
	// EnvKeyMultiAttestorQuorum is the minimum number of signatures required for valid attestation.
	EnvKeyMultiAttestorQuorum = "MULTI_ATTESTOR_QUORUM"
	// EnvKeyMultiAttestorActive is the number of attestor processes to actually run.
	EnvKeyMultiAttestorActive = "MULTI_ATTESTOR_ACTIVE"

	// EnvKeySolanaTestnetType is the environment variable name to configure the Solana testnet type.
	EnvKeySolanaTestnetType = "SOLANA_TESTNET_TYPE"
	// SolanaTestnetType_Localnet is the Solana testnet type for using a local testnet.
	SolanaTestnetType_Localnet = "localnet"
	// SolanaTestnetType_None is the Solana testnet type for using no chain.
	SolanaTestnetType_None = "none"
	// SolanaChainID is the chain identifier for Solana localnet used in proof API config.
	SolanaChainID = "solana-localnet"
	// SolanaLocalnetRPC is the default RPC URL for Solana localnet.
	SolanaLocalnetRPC = "http://localhost:8899"
	// SolanaGMPPortID is the port identifier for GMP (General Message Passing) application.
	SolanaGMPPortID = "gmpport"

	// Ics27Version is the ICS27 GMP protocol version.
	Ics27Version = "ics27-2"
	// Ics27AbiEncoding is the solidity abi encoding type for the ICS27 packets.
	Ics27AbiEncoding = "application/x-solidity-abi"
	// Ics27ProtobufEncoding is the protobuf encoding type for ICS27 packets (used for Solana).
	Ics27ProtobufEncoding = "application/x-protobuf"

	// SolidityProjectDir is the root of the Foundry project for the Solidity IBC implementation,
	// relative to the repository root. Forge commands are run from this directory.
	SolidityProjectDir = "ibc-solidity"
	// Sp1GenesisFilePath is the path to the genesis file for the SP1 chain.
	// This file is generated and then deleted by the test.
	Sp1GenesisFilePath = "ibc-solidity/scripts/genesis.json"
	// SolidityFixturesDir is the directory where the Solidity fixtures are stored.
	SolidityFixturesDir = "ibc-solidity/test/solidity-ibc/fixtures/"
	// SP1ICS07FixturesDir is the directory where the SP1ICS07 fixtures are stored.
	SP1ICS07FixturesDir = "ibc-solidity/test/sp1-ics07/fixtures"
	// WasmFixturesDir is the directory where the Rust fixtures are stored.
	WasmFixturesDir = "packages/ethereum/light-client/src/test_utils/fixtures"
	// ProofAPIConfigFilePath is the path to generate the proof API config file.
	ProofAPIConfigFilePath = "programs/proof-api/config.json"
	// E2EDeployScriptPath is the path to the E2E deploy script.
	E2EDeployScriptPath = "scripts/E2ETestDeploy.s.sol:E2ETestDeploy"

	// E2EDeployerPrivateKeyHex is Anvil's default account 0 private key.
	// Used as both the E2E deployer (for deterministic contract addresses) and the Anvil faucet.
	// Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
	E2EDeployerPrivateKeyHex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
	// DeterministicIFTAddress is the IFT contract address deployed by E2EDeployerPrivateKeyHex at nonce 18.
	// Nonce 18 is the deployment order of the IFT proxy in E2ETestDeploy.s.sol.
	// If the deploy script changes, re-run the test with logging to find the new nonce.
	// Recompute with: just compute-ift-addresses
	DeterministicIFTAddress = "0x68B1D87F95878fE05B998F19b66F4baba5De1aed"
	// DeterministicICAAddress is the Cosmos ICA address controlled by DeterministicIFTAddress.
	// Recompute with: just compute-ift-addresses
	DeterministicICAAddress = "wf1c98z43jf73erjp94y2nde4hpyyugtgvlcc9jwus8s25tpxc703pqfp60zh"
	// EnvKeyIFTICAAddress is the env var for passing ICA address to the deploy script.
	EnvKeyIFTICAAddress = "IFT_ICA_ADDRESS"
	// SolanaLedgerDir is the path to the Solana ledger directory.
	SolanaLedgerDir = "test-ledger"
	// TendermintLightClientFixturesDir is the directory where the Tendermint light client fixtures are stored.
	TendermintLightClientFixturesDir = "packages/tendermint-light-client/fixtures/"

	// IbcCommitmentSlotHex is the storage slot in the IBC solidity contract for the IBC commitments.
	IbcCommitmentSlotHex = ics26router.IbcStoreStorageSlot

	// FirstWasmClientID is the first wasm client ID. Used for testing.
	FirstWasmClientID = "08-wasm-0"
	// FirstAttestationsClientID is the first native ibc-go attestations client ID. Used for testing.
	FirstAttestationsClientID = "attestations-0"
	// FirstUniversalClientID is the first universal client ID. Used for testing.
	FirstUniversalClientID = "client-0"
	// SecondUniversalClientID is the second universal client ID. Used for testing.
	SecondUniversalClientID = "client-1"
	// CustomClientID is the custom client ID used for testing.
	// BUG: https://github.com/cosmos/ibc-go/issues/8145
	// We must use a client ID of the form `type-n` due to the issue above.
	CustomClientID = "cosmoshub-1"

	// IFTTestDenom is the default IFT test denom used in tests.
	IFTTestDenom = "testift"
	// IFTSendCallConstructorCosmos is the context for IFT send call constructor on Cosmos.
	IFTSendCallConstructorCosmos = "cosmos"
	// IFTSendCallConstructorEVM is the context for IFT send call constructor on EVM.
	IFTSendCallConstructorEVM = "evm"
	// IFTSendCallConstructorSolana is the context for IFT send call constructor on Solana.
	IFTSendCallConstructorSolana = "solana"
	// IFTModuleName is the IFT module name.
	IFTModuleName = "ift"

	// Sp1 verifier address parameter key for the proof API's sp1 light client creation.
	ParameterKey_Sp1Verifier = "sp1_verifier"
	// Zk algorithm parameter key for the proof API's sp1 light client creation.
	ParameterKey_ZkAlgorithm = "zk_algorithm"
	// The role manager address parameter key for the proof API's sp1 light client creation.
	ParameterKey_RoleManager = "role_manager"
	// Checksum hex parameter key for the proof API's ethereum light client creation.
	ParameterKey_ChecksumHex = "checksum_hex"

	// Min sigs parameter for the attestor light client creation.
	ParameterKey_MinRequiredSigs = "min_required_sigs"
	// Default minimum required signatures for attestor light client in tests.
	DefaultMinRequiredSigs = 1
	// Addresses parameter for the attestor light client creation.
	ParameterKey_AttestorAddresses = "attestor_addresses"
	// Height parameter for the attestor light client creation.
	ParameterKey_height = "height"
	// Timestamp parameter for the attestor light client creation.
	ParameterKey_timestamp = "timestamp"
	// The tmp path used for programatically generated attestor configs
	AttestorConfigPath = "/tmp/attestor.toml"
	// The tmp path template for multiple Ethereum attestor configs
	EthAttestorConfigPathTemplate = "/tmp/eth_attestor_%d.toml"
	// The tmp path template for multiple Cosmos attestor configs
	CosmosAttestorConfigPathTemplate = "/tmp/cosmos_attestor_%d.toml"
	// The tmp path template for multiple Solana attestor configs
	SolanaAttestorConfigPathTemplate = "/tmp/solana_attestor_%d.toml"
	// The tmp path template for attestor keystores
	AttestorKeystorePathTemplate = "/tmp/attestor_keystore_%d"
	// The tmp path used for programatically generated aggregator configs
	AggregatorConfigPath = "/tmp/aggregator.toml"
	// The RPC endpoint for the aggregator service
	AggregatorRpcPath = "http://localhost:8080"

	// Attestor chain type values
	Attestor_ChainType_EVM    = "evm"
	Attestor_ChainType_Cosmos = "cosmos"
	Attestor_ChainType_Solana = "solana"
)

var (
	// MaxDepositPeriod Maximum period to deposit on a proposal.
	// This value overrides the default value in the gov module using the `modifyGovV1AppState` function.
	MaxDepositPeriod = time.Second * 10
	// VotingPeriod Duration of the voting period.
	// This value overrides the default value in the gov module using the `modifyGovV1AppState` function.
	VotingPeriod = time.Second * 30

	// StartingEthBalance is the amount of ETH to give to each user at the start of the test.
	StartingEthBalance = math.NewInt(2 * ethereum.ETHER.Int64())

	// DefaultTrustLevel is the trust level used by the SP1ICS07Tendermint contract.
	DefaultTrustLevel = ibctm.Fraction{Numerator: 1, Denominator: 3}.ToTendermint()

	// DefaultTrustPeriod is the trust period used by the SP1ICS07Tendermint contract.
	// 129600 seconds = 14 days
	DefaultTrustPeriod = 1209600

	// DefaultMaxClockDrift is the default maximum clock drift used by ICS07Tendermint (in seconds).
	DefaultMaxClockDrift = 15

	// MaxUint256 is the maximum value for a uint256.
	MaxUint256 = uint256.MustFromHex("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")

	// StartingERC20Balance is the starting balance for the ERC20 contract.
	StartingERC20Balance = new(big.Int).Div(MaxUint256.ToBig(), big.NewInt(2))

	// DefaultAdminRole is the default admin role for AccessControl contract.
	DefaultAdminRole = [32]byte{0x00}

	// PortCustomizerRole is the role required to customize the port.
	PortCustomizerRole = func() (role [32]byte) {
		copy(role[:], crypto.Keccak256([]byte("PORT_CUSTOMIZER_ROLE")))
		return role
	}()
)

func EnvGet(key, defaultValue string) string {
	value := os.Getenv(key)
	if value == "" {
		return defaultValue
	}

	return value
}

func EnvEnsure(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}

	os.Setenv(key, defaultValue)

	return defaultValue
}

type SolanaOptions struct {
	GMPProgramID string `json:"gmp_program_id"`
	MintPubkey   string `json:"mint_pubkey"`
}

// BuildSolanaIFTConstructor returns "{\"solana\":{\"gmp_program_id\":\"...\",\"mint_pubkey\":\"...\"}}"
// counterpartyClientId is the client ID on Solana that tracks the Cosmos chain (needed for gmp_account_pda derivation)
func BuildSolanaIFTConstructor(gmpProgramID, mintPubkey string) string {
	cfg := SolanaOptions{
		GMPProgramID: gmpProgramID,
		MintPubkey:   mintPubkey,
	}
	wrapper := map[string]SolanaOptions{IFTSendCallConstructorSolana: cfg}
	jsonBytes, _ := json.Marshal(wrapper)
	return string(jsonBytes)
}
