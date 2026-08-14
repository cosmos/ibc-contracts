// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"crypto/ecdsa"
	"encoding/hex"
	"fmt"
	"math/big"
	"os"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	"github.com/ethereum/go-ethereum/accounts/abi"
	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"
	ibchostv2 "github.com/cosmos/ibc-go/v11/modules/core/24-host/v2"
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"

	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/attestation"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ibcerc20"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics20transfer"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/sp1ics07tendermint"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/attestor"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	ibctesting "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ibctesting"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/erc20"
	ethereumtypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/ethereum"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

// IbcEurekaTestSuite is a suite of tests that wraps TestSuite
// and can provide additional functionality
type IbcEurekaTestSuite struct {
	e2esuite.TestSuite

	// Whether to generate fixtures for tests or not
	solidityFixtureGenerator *types.SolidityFixtureGenerator
	wasmFixtureGenerator     *types.WasmFixtureGenerator

	// The private key of a test account
	key *ecdsa.PrivateKey
	// The private key of the faucet account of interchaintest
	deployer *ecdsa.PrivateKey

	contractAddresses ethereum.DeployedContracts
	sp1Ics07Address   ethcommon.Address

	sp1Ics07Contract       *sp1ics07tendermint.Contract
	attestationLightClient *attestation.Contract
	ics26Contract          *ics26router.Contract
	ics20Contract          *ics20transfer.Contract
	erc20Contract          *erc20.Contract

	ProofApiClient proofapitypes.ProofApiServiceClient

	SimdRelayerSubmitter ibc.Wallet
	EthRelayerSubmitter  *ecdsa.PrivateKey

	// Attestor-specific fields
	ethAttestorEndpoints    []string // Used when ETH_LC_TYPE=attestor
	ethAttestorAddresses    []string // Attestor addresses for light client registration
	cosmosAttestorEndpoints []string // Used when COSMOS_LC_TYPE=attestor
	cosmosAttestorAddresses []string // Attestor addresses for light client registration
}

// TestWithIbcEurekaTestSuite is the boilerplate code that allows the test suite to be run
func TestWithIbcEurekaTestSuite(t *testing.T) {
	suite.Run(t, new(IbcEurekaTestSuite))
}

// isEthLcOnCosmosAttestor returns true if the Ethereum light client on Cosmos uses any attestor type (attestor-native)
func (s *IbcEurekaTestSuite) isEthLcOnCosmosAttestor() bool {
	return s.GetEthLightClientType() == testvalues.EthLCOnCosmosTypeAttestorNative
}

// isEthLcOnCosmosNativeAttestor returns true if the Ethereum light client on Cosmos is a native ibc-go attestor (attestor-native)
func (s *IbcEurekaTestSuite) isEthLcOnCosmosNativeAttestor() bool {
	return s.GetEthLightClientType() == testvalues.EthLCOnCosmosTypeAttestorNative
}

// isCosmosLcOnEthAttestor returns true if the Cosmos light client on Ethereum uses attestor (COSMOS_LC_ON_ETH=attestor)
func (s *IbcEurekaTestSuite) isCosmosLcOnEthAttestor() bool {
	cosmosLcType := os.Getenv(testvalues.EnvKeyCosmosLcOnEth)
	return cosmosLcType == testvalues.CosmosLcTypeAttestor
}

// getEthLcClientIDOnCosmos returns the expected client ID for the Ethereum light client deployed on Cosmos.
// Returns FirstAttestationsClientID for native attestor, FirstWasmClientID otherwise.
func (s *IbcEurekaTestSuite) getEthLcClientIDOnCosmos() string {
	if s.isEthLcOnCosmosNativeAttestor() {
		return testvalues.FirstAttestationsClientID
	}
	return testvalues.FirstWasmClientID
}

// SetupSuite calls the underlying IbcEurekaTestSuite's SetupSuite method
// and deploys the IbcEureka contract
func (s *IbcEurekaTestSuite) SetupSuite(ctx context.Context, proofType types.SupportedProofType) {
	s.TestSuite.SetupSuite(ctx)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	s.T().Logf("Setting up the test suite with proof type: %s", proofType.String())

	var prover string
	s.Require().True(s.Run("Set up environment", func() {
		err := os.Chdir("../..")
		s.Require().NoError(err)

		s.key, err = eth.CreateAndFundUser()
		s.Require().NoError(err)

		s.EthRelayerSubmitter, err = eth.CreateAndFundUser()
		s.Require().NoError(err)

		operatorKey, err := eth.CreateAndFundUser()
		s.Require().NoError(err)

		s.deployer, err = eth.CreateAndFundUser()
		s.Require().NoError(err)

		s.SimdRelayerSubmitter = s.CreateAndFundCosmosUser(ctx, simd)

		prover = os.Getenv(testvalues.EnvKeySp1Prover)
		switch prover {
		case "", testvalues.EnvValueSp1Prover_Mock:
			s.T().Logf("Using mock prover")
			prover = testvalues.EnvValueSp1Prover_Mock
			os.Setenv(testvalues.EnvKeySp1Prover, testvalues.EnvValueSp1Prover_Mock)
			os.Setenv(testvalues.EnvKeyVerifier, testvalues.EnvValueVerifier_Mock)

			s.Require().Empty(
				os.Getenv(testvalues.EnvKeyGenerateSolidityFixtures),
				"Fixtures are not supported for mock prover",
			)
		case testvalues.EnvValueSp1Prover_Network:
			s.Require().Empty(
				os.Getenv(testvalues.EnvKeyVerifier),
				fmt.Sprintf("%s should not be set when using the network prover in e2e tests.", testvalues.EnvKeyVerifier),
			)
			// make sure that the NETWORK_PRIVATE_KEY is set.
			s.Require().NotEmpty(os.Getenv(testvalues.EnvKeyNetworkPrivateKey))
		default:
			s.Require().Fail("invalid prover type: %s", prover)
		}

		if os.Getenv(testvalues.EnvKeyRustLog) == "" {
			os.Setenv(testvalues.EnvKeyRustLog, testvalues.EnvValueRustLog_Info)
		}
		os.Setenv(testvalues.EnvKeyEthRPC, eth.RPC)
		os.Setenv(testvalues.EnvKeyTendermintRPC, simd.GetHostRPCAddress())
		os.Setenv(testvalues.EnvKeySp1Prover, prover)
		os.Setenv(testvalues.EnvKeyOperatorPrivateKey, hex.EncodeToString(crypto.FromECDSA(operatorKey)))
	}))

	// Needs to be added here so the cleanup is called after the test suite is done
	s.wasmFixtureGenerator = types.NewWasmFixtureGenerator(&s.Suite)
	s.solidityFixtureGenerator = types.NewSolidityFixtureGenerator()

	s.Require().True(s.Run("Deploy IBC contracts", func() {
		stdout, err := eth.ForgeScript(s.deployer, testvalues.E2EDeployScriptPath)
		s.Require().NoError(err)

		s.contractAddresses, err = ethereum.GetEthContractsFromDeployOutput(string(stdout))
		s.Require().NoError(err)
		s.ics26Contract, err = ics26router.NewContract(ethcommon.HexToAddress(s.contractAddresses.Ics26Router), eth.RPCClient)
		s.Require().NoError(err)
		s.ics20Contract, err = ics20transfer.NewContract(ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer), eth.RPCClient)
		s.Require().NoError(err)
		s.erc20Contract, err = erc20.NewContract(ethcommon.HexToAddress(s.contractAddresses.Erc20), eth.RPCClient)
		s.Require().NoError(err)
	}))

	// Start Ethereum attestors if ETH_LC_TYPE=attestor (for eth→cosmos direction)
	// NOTE: Not wrapped in s.Run() subtest to ensure t.Cleanup() registers on parent test,
	// preventing premature container cleanup when subtest completes
	if s.isEthLcOnCosmosAttestor() {
		s.T().Log("Setting up Ethereum attestors...")
		chainType := attestor.ChainTypeEvm
		if os.Getenv(testvalues.EnvKeyEthTestnetType) == testvalues.EthTestnetTypePoS {
			chainType = attestor.ChainTypeCosmos
		}
		result := attestor.SetupEthAttestors(ctx, s.T(), s.GetDockerClient(), s.GetNetworkID(), eth.DockerRPC, s.contractAddresses.Ics26Router, chainType)
		s.ethAttestorEndpoints = result.Endpoints
		s.ethAttestorAddresses = result.Addresses
	}

	// Start Cosmos attestors if COSMOS_LC_TYPE=attestor (for cosmos→eth direction)
	// NOTE: Not wrapped in s.Run() subtest to ensure t.Cleanup() registers on parent test,
	// preventing premature container cleanup when subtest completes
	if s.isCosmosLcOnEthAttestor() {
		s.T().Log("Setting up Cosmos attestors...")
		result := attestor.SetupCosmosAttestors(ctx, s.T(), s.GetDockerClient(), s.GetNetworkID(), simd.GetRPCAddress())
		s.cosmosAttestorEndpoints = result.Endpoints
		s.cosmosAttestorAddresses = result.Addresses
	}

	// Note: Docker containers cleanup automatically via t.Cleanup() registered in SetupAttestors

	var proofApiProcess *os.Process
	s.Require().True(s.Run("Start Proof API", func() {
		builder := proofapi.NewConfigBuilder()

		// Eth → Cosmos direction
		if s.isEthLcOnCosmosAttestor() {
			builder.EthToCosmosAttested(proofapi.EthToCosmosAttestedParams{
				EthChainID:        eth.ChainID.String(),
				CosmosChainID:     simd.Config().ChainID,
				TmRPC:             simd.GetHostRPCAddress(),
				ICS26Address:      s.contractAddresses.Ics26Router,
				EthRPC:            eth.RPC,
				SignerAddress:     s.SimdRelayerSubmitter.FormattedAddress(),
				AttestorEndpoints: s.ethAttestorEndpoints,
			})
		} else {
			beaconAPI := ""
			if eth.BeaconAPIClient != nil {
				beaconAPI = eth.BeaconAPIClient.GetBeaconAPIURL()
			}
			builder.EthToCosmos(proofapi.EthToCosmosParams{
				EthChainID:    eth.ChainID.String(),
				CosmosChainID: simd.Config().ChainID,
				TmRPC:         simd.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				BeaconAPI:     beaconAPI,
				SignerAddress: s.SimdRelayerSubmitter.FormattedAddress(),
				MockClient:    os.Getenv(testvalues.EnvKeyEthTestnetType) == testvalues.EthTestnetTypeAnvil,
			})
		}

		// Cosmos → Eth direction
		if s.isCosmosLcOnEthAttestor() {
			builder.CosmosToEthAttested(proofapi.CosmosToEthAttestedParams{
				CosmosChainID:     simd.Config().ChainID,
				EthChainID:        eth.ChainID.String(),
				TmRPC:             simd.GetHostRPCAddress(),
				ICS26Address:      s.contractAddresses.Ics26Router,
				EthRPC:            eth.RPC,
				AttestorEndpoints: s.cosmosAttestorEndpoints,
			})
		} else {
			builder.CosmosToEthSP1(proofapi.CosmosToEthSP1Params{
				CosmosChainID: simd.Config().ChainID,
				EthChainID:    eth.ChainID.String(),
				TmRPC:         simd.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				Prover: proofapi.SP1ProverConfig{
					Type:           prover,
					PrivateCluster: os.Getenv(testvalues.EnvKeyNetworkPrivateCluster) == testvalues.EnvValueSp1Prover_PrivateCluster,
				},
			})
		}

		config := builder.Build()
		err := config.GenerateConfigFile(testvalues.ProofAPIConfigFilePath)
		s.Require().NoError(err)

		proofApiProcess, err = proofapi.StartProofAPI(testvalues.ProofAPIConfigFilePath)
		s.Require().NoError(err)

		s.T().Cleanup(func() {
			os.Remove(testvalues.ProofAPIConfigFilePath)
		})
	}))

	s.T().Cleanup(func() {
		if proofApiProcess != nil {
			err := proofApiProcess.Kill()
			if err != nil {
				s.T().Logf("Failed to kill the proof API process: %v", err)
			}
		}
	})

	s.Require().True(s.Run("Create Proof API Client", func() {
		var err error
		s.ProofApiClient, err = proofapi.GetGRPCClient(proofapi.DefaultProofAPIGRPCAddress())
		s.Require().NoError(err)
	}))

	// Deploy light client on Ethereum for Cosmos→Eth direction
	if s.isCosmosLcOnEthAttestor() {
		s.Require().True(s.Run("Deploy Cosmos attestor light client on EVM", func() {
			latestCosmosHeader, err := simd.GetFullNode().Client.Header(ctx, nil)
			s.Require().NoError(err)

			s.Require().NotEmpty(s.cosmosAttestorAddresses, "cosmos attestor addresses should be set")

			cosmosLCParams := map[string]string{
				testvalues.ParameterKey_AttestorAddresses: ethcommon.HexToAddress(s.cosmosAttestorAddresses[0]).Hex(),
				testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(testvalues.DefaultMinRequiredSigs),
				testvalues.ParameterKey_height:            strconv.FormatInt(latestCosmosHeader.Header.Height, 10),
				testvalues.ParameterKey_timestamp:         strconv.FormatInt(latestCosmosHeader.Header.Time.Unix(), 10),
				testvalues.ParameterKey_RoleManager:       ethcommon.HexToAddress(s.contractAddresses.Ics26Router).Hex(),
			}

			var createClientTxBz []byte
			s.Require().True(s.Run("Retrieve create client tx", func() {
				resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
					SrcChain:   simd.Config().ChainID,
					DstChain:   eth.ChainID.String(),
					Parameters: cosmosLCParams,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				s.Require().Empty(resp.Address)

				createClientTxBz = resp.Tx
			}))

			s.Require().True(s.Run("Broadcast relay tx", func() {
				receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, nil, createClientTxBz)
				s.Require().NoError(err)
				s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

				s.sp1Ics07Address = receipt.ContractAddress
				s.attestationLightClient, err = attestation.NewContract(s.sp1Ics07Address, eth.RPCClient)
				s.Require().NoError(err)
			}))
		}))
	} else {
		s.Require().True(s.Run("Deploy SP1 ICS07 contract", func() {
			verifierAddress, err := s.contractAddresses.GetVerifierAddress(prover, proofType.String())
			s.Require().NoError(err)

			var createClientTxBz []byte
			s.Require().True(s.Run("Retrieve create client tx", func() {
				resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
					SrcChain: simd.Config().ChainID,
					DstChain: eth.ChainID.String(),
					Parameters: map[string]string{
						testvalues.ParameterKey_Sp1Verifier: verifierAddress,
						testvalues.ParameterKey_ZkAlgorithm: proofType.String(),
					},
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				s.Require().Empty(resp.Address)

				createClientTxBz = resp.Tx
			}))

			s.Require().True(s.Run("Broadcast relay tx", func() {
				receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, nil, createClientTxBz)
				s.Require().NoError(err)
				s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

				s.sp1Ics07Address = receipt.ContractAddress
				s.sp1Ics07Contract, err = sp1ics07tendermint.NewContract(s.sp1Ics07Address, eth.RPCClient)
				s.Require().NoError(err)
			}))
		}))
	}

	s.Require().True(s.Run("Fund address with ERC20", func() {
		tx, err := s.erc20Contract.Transfer(s.GetTransactOpts(eth.Faucet, eth), crypto.PubkeyToAddress(s.key.PublicKey), testvalues.StartingERC20Balance)
		s.Require().NoError(err)

		_, err = eth.GetTxReciept(ctx, tx.Hash()) // wait for the tx to be mined
		s.Require().NoError(err)
	}))

	// Create Ethereum light client on Cosmos for Eth→Cosmos direction
	s.Require().True(s.Run("Create ethereum light client on Cosmos chain", func() {
		checksumHex := s.StoreLightClient(ctx, simd, s.SimdRelayerSubmitter)
		// checksumHex is empty for native attestor (no wasm needed)
		if !s.isEthLcOnCosmosNativeAttestor() {
			s.Require().NotEmpty(checksumHex)
		}

		var createClientTxBodyBz []byte

		// Get attestor address if Eth→Cosmos uses attestor mode
		var attestorAddress string
		if s.isEthLcOnCosmosAttestor() {
			s.Require().NotEmpty(s.ethAttestorAddresses, "eth attestor addresses should be set")
			attestorAddress = s.ethAttestorAddresses[0]
		}

		s.Require().True(s.Run("Retrieve create client tx", func() {
			parameters := map[string]string{
				testvalues.ParameterKey_ChecksumHex: checksumHex,
			}

			// Add attestor-specific parameters if Eth→Cosmos uses attestor mode
			if s.isEthLcOnCosmosAttestor() {
				currentBlockHeader, err := eth.RPCClient.HeaderByNumber(ctx, nil)
				s.Require().NoError(err)

				clientHeight := currentBlockHeader.Number.Int64()
				if clientHeight < 1 {
					clientHeight = 1
				}

				blockHeader, err := eth.RPCClient.HeaderByNumber(ctx, big.NewInt(clientHeight))
				s.Require().NoError(err)

				parameters[testvalues.ParameterKey_AttestorAddresses] = ethcommon.HexToAddress(attestorAddress).Hex()
				parameters[testvalues.ParameterKey_MinRequiredSigs] = strconv.Itoa(testvalues.DefaultMinRequiredSigs)
				parameters[testvalues.ParameterKey_height] = strconv.FormatInt(clientHeight, 10)
				parameters[testvalues.ParameterKey_timestamp] = strconv.FormatUint(blockHeader.Time, 10)
			}

			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain:   eth.ChainID.String(),
				DstChain:   simd.Config().ChainID,
				Parameters: parameters,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			createClientTxBodyBz = resp.Tx
		}))

		err := s.wasmFixtureGenerator.AddInitialStateStep(createClientTxBodyBz)
		s.Require().NoError(err)

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, createClientTxBodyBz)
			clientId, err := cosmos.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(s.getEthLcClientIDOnCosmos(), clientId)
		}))
	}))

	s.Require().True(s.Run("Add client and counterparty on EVM", func() {
		// Merkle prefix depends on how Cosmos state is verified on Ethereum
		// Attestor: uses keccak256(commitment_path) directly, so prefix is [""]
		// SP1: uses ICS23 merkle proofs against IAVL, so prefix is ["ibc", ""]
		var merklePrefix [][]byte
		if s.isCosmosLcOnEthAttestor() {
			merklePrefix = [][]byte{[]byte("")}
		} else {
			merklePrefix = [][]byte{[]byte(ibcexported.StoreKey), []byte("")}
		}

		counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
			ClientId:     s.getEthLcClientIDOnCosmos(),
			MerklePrefix: merklePrefix,
		}
		tx, err := s.ics26Contract.AddClient(s.GetTransactOpts(s.deployer, eth), testvalues.CustomClientID, counterpartyInfo, s.sp1Ics07Address)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)

		event, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseICS02ClientAdded)
		s.Require().NoError(err)
		s.Require().Equal(testvalues.CustomClientID, event.ClientId)
		s.Require().Equal(s.getEthLcClientIDOnCosmos(), event.CounterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Register counterparty on Cosmos chain", func() {
		merklePathPrefix := [][]byte{[]byte("")}

		_, err := s.BroadcastMessages(ctx, simd, s.SimdRelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 s.getEthLcClientIDOnCosmos(),
			CounterpartyMerklePrefix: merklePathPrefix,
			CounterpartyClientId:     testvalues.CustomClientID,
			Signer:                   s.SimdRelayerSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Generate the genesis fixtures", func() {
		if !s.solidityFixtureGenerator.Enabled {
			s.T().Skip("Skipping solidity fixture generation")
		}

		clientState, err := s.sp1Ics07Contract.ClientState(nil)
		s.Require().NoError(err)
		clientStateBz, err := s.sp1Ics07Contract.GetClientState(nil)
		s.Require().NoError(err)
		consensusStateHash, err := s.sp1Ics07Contract.GetConsensusStateHash(nil, clientState.LatestHeight.RevisionHeight)
		s.Require().NoError(err)
		updateClientVkey, err := s.sp1Ics07Contract.UPDATECLIENTPROGRAMVKEY(nil)
		s.Require().NoError(err)
		membershipVkey, err := s.sp1Ics07Contract.MEMBERSHIPPROGRAMVKEY(nil)
		s.Require().NoError(err)
		ucAndMembershipVkey, err := s.sp1Ics07Contract.UPDATECLIENTANDMEMBERSHIPPROGRAMVKEY(nil)
		s.Require().NoError(err)
		misbehaviourVkey, err := s.sp1Ics07Contract.MISBEHAVIOURPROGRAMVKEY(nil)
		s.Require().NoError(err)

		s.solidityFixtureGenerator.SetGenesisFixture(
			clientStateBz, consensusStateHash, updateClientVkey,
			membershipVkey, ucAndMembershipVkey, misbehaviourVkey,
		)
	}))
}

func (s *IbcEurekaTestSuite) Test_Deploy() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.DeployTest(ctx, proofType)
}

// DeployTest tests the deployment of the IbcEureka contracts
func (s *IbcEurekaTestSuite) DeployTest(ctx context.Context, proofType types.SupportedProofType) {
	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	if !s.isCosmosLcOnEthAttestor() {
		s.Require().True(s.Run("Verify SP1 Client", func() {
			clientState, err := s.sp1Ics07Contract.ClientState(nil)
			s.Require().NoError(err)

			stakingParams, err := simd.StakingQueryParams(ctx)
			s.Require().NoError(err)

			s.Require().Equal(simd.Config().ChainID, clientState.ChainId)
			s.Require().Equal(uint8(testvalues.DefaultTrustLevel.Numerator), clientState.TrustLevel.Numerator)
			s.Require().Equal(uint8(testvalues.DefaultTrustLevel.Denominator), clientState.TrustLevel.Denominator)
			s.Require().Equal(uint32(testvalues.DefaultTrustPeriod), clientState.TrustingPeriod)
			s.Require().Equal(uint32(stakingParams.UnbondingTime.Seconds()), clientState.UnbondingPeriod)
			s.Require().False(clientState.IsFrozen)
			s.Require().Equal(uint64(1), clientState.LatestHeight.RevisionNumber)
			s.Require().Greater(clientState.LatestHeight.RevisionHeight, uint64(0))
		}))
	}

	s.Require().True(s.Run("Verify ICS02 Client", func() {
		clientAddress, err := s.ics26Contract.GetClient(nil, testvalues.CustomClientID)
		s.Require().NoError(err)
		s.Require().NotEqual(ethcommon.Address{}, clientAddress)
		if !s.isCosmosLcOnEthAttestor() {
			s.Require().Equal(s.sp1Ics07Address, clientAddress)
		}

		counterpartyInfo, err := s.ics26Contract.GetCounterparty(nil, testvalues.CustomClientID)
		s.Require().NoError(err)
		s.Require().Equal(s.getEthLcClientIDOnCosmos(), counterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Verify ICS26 Router", func() {
		transferAddress, err := s.ics26Contract.GetIBCApp(nil, transfertypes.PortID)
		s.Require().NoError(err)
		s.Require().Equal(s.contractAddresses.Ics20Transfer, strings.ToLower(transferAddress.Hex()))
	}))

	s.Require().True(s.Run("Verify ERC20 Genesis", func() {
		userBalance, err := s.erc20Contract.BalanceOf(nil, crypto.PubkeyToAddress(s.key.PublicKey))
		s.Require().NoError(err)
		s.Require().Equal(testvalues.StartingERC20Balance, userBalance)
	}))

	s.Require().True(s.Run("Verify ethereum light client", func() {
		_, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simd, &clienttypes.QueryClientStateRequest{
			ClientId: s.getEthLcClientIDOnCosmos(),
		})
		s.Require().NoError(err)

		counterpartyInfoResp, err := e2esuite.GRPCQuery[clienttypesv2.QueryCounterpartyInfoResponse](ctx, simd, &clienttypesv2.QueryCounterpartyInfoRequest{
			ClientId: s.getEthLcClientIDOnCosmos(),
		})
		s.Require().NoError(err)
		s.Require().Equal(testvalues.CustomClientID, counterpartyInfoResp.CounterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Verify Cosmos to Eth Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: simd.Config().ChainID,
			DstChain: eth.ChainID.String(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(simd.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(eth.ChainID.String(), info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Eth to Cosmos Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: eth.ChainID.String(),
			DstChain: simd.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(eth.ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(simd.Config().ChainID, info.TargetChain.ChainId)
	}))
}

func (s *IbcEurekaTestSuite) Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenfromEthereumToCosmosAndBackTest(ctx, proofType, 1, big.NewInt(testvalues.TransferAmount))
}

func (s *IbcEurekaTestSuite) Test_25_ICS20TransferERC20TokenfromEthereumToCosmosAndBack() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenfromEthereumToCosmosAndBackTest(ctx, proofType, 25, big.NewInt(testvalues.TransferAmount))
}

func (s *IbcEurekaTestSuite) Test_50_ICS20TransferERC20TokenfromEthereumToCosmosAndBack() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenfromEthereumToCosmosAndBackTest(ctx, proofType, 50, big.NewInt(testvalues.TransferAmount))
}

func (s *IbcEurekaTestSuite) Test_ICS20TransferUint256TokenfromEthereumToCosmosAndBack() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	transferAmount := new(big.Int).Div(testvalues.StartingERC20Balance, big.NewInt(2))
	s.ICS20TransferERC20TokenfromEthereumToCosmosAndBackTest(ctx, proofType, 1, transferAmount)
}

// ICS20TransferERC20TokenfromEthereumToCosmosAndBackTest tests the ICS20 transfer functionality by transferring
// ERC20 tokens with n packets from Ethereum to Cosmos chain and then back from Cosmos chain to Ethereum
func (s *IbcEurekaTestSuite) ICS20TransferERC20TokenfromEthereumToCosmosAndBackTest(
	ctx context.Context, proofType types.SupportedProofType, numOfTransfers int, transferAmount *big.Int,
) {
	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	totalTransferAmount := new(big.Int).Mul(transferAmount, big.NewInt(int64(numOfTransfers)))
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	ics20transferAbi, err := abi.JSON(strings.NewReader(ics20transfer.ContractABI))
	s.Require().NoError(err)

	s.Require().True(s.Run("Approve the ICS20Transfer.sol contract to spend the erc20 tokens", func() {
		tx, err := s.erc20Contract.Approve(s.GetTransactOpts(s.key, eth), ics20Address, totalTransferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		allowance, err := s.erc20Contract.Allowance(nil, ethereumUserAddress, ics20Address)
		s.Require().NoError(err)
		s.Require().Equal(totalTransferAmount, allowance)
	}))

	var (
		sendPacket    ics26router.IICS26RouterMsgsPacket
		ethSendTxHash []byte
		escrowAddress ethcommon.Address
	)
	s.Require().True(s.Run(fmt.Sprintf("Send %d transfers on Ethereum", numOfTransfers), func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		transferMulticall := make([][]byte, numOfTransfers)

		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			SourceClient:     testvalues.CustomClientID,
			Memo:             "",
		}

		encodedMsg, err := ics20transferAbi.Pack("sendTransfer", msgSendPacket)
		s.Require().NoError(err)
		for i := range numOfTransfers {
			transferMulticall[i] = encodedMsg
		}

		tx, err := s.ics20Contract.Multicall(s.GetTransactOpts(s.key, eth), transferMulticall)
		s.Require().NoError(err)
		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		s.T().Logf("Multicall send %d transfers gas used: %d", numOfTransfers, receipt.GasUsed)
		ethSendTxHash = tx.Hash().Bytes()

		sendPacketEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseSendPacket)
		s.Require().NoError(err)
		sendPacket = sendPacketEvent.Packet
		s.Require().Equal(uint64(1), sendPacket.Sequence)
		s.Require().Equal(timeout, sendPacket.TimeoutTimestamp)
		s.Require().Len(sendPacket.Payloads, 1)
		s.Require().Equal(transfertypes.PortID, sendPacket.Payloads[0].SourcePort)
		s.Require().Equal(testvalues.CustomClientID, sendPacket.SourceClient)
		s.Require().Equal(transfertypes.PortID, sendPacket.Payloads[0].DestPort)
		s.Require().Equal(s.getEthLcClientIDOnCosmos(), sendPacket.DestClient)
		s.Require().Equal(transfertypes.V1, sendPacket.Payloads[0].Version)
		s.Require().Equal(transfertypes.EncodingABI, sendPacket.Payloads[0].Encoding)

		s.True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(new(big.Int).Sub(testvalues.StartingERC20Balance, totalTransferAmount), userBalance)

			// Get the escrow address
			escrowAddress, err = s.ics20Contract.GetEscrow(nil, testvalues.CustomClientID)
			s.Require().NoError(err)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Equal(totalTransferAmount, escrowBalance)
		}))
	}))

	var (
		denomOnCosmos transfertypes.Denom
		ackTxHash     []byte
	)
	s.Require().True(s.Run("Receive packets on Cosmos chain", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx

			s.wasmFixtureGenerator.AddFixtureStep("receive_packets", ethereumtypes.RelayerMessages{
				RelayerTxBody: hex.EncodeToString(relayTxBodyBz),
			})
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, relayTxBodyBz)

			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			denomOnCosmos = transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, s.getEthLcClientIDOnCosmos()))

			// User balance on Cosmos chain
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(totalTransferAmount, resp.Balance.Amount.BigInt())
			s.Require().Equal(denomOnCosmos.IBCDenom(), resp.Balance.Denom)
		}))
	}))

	s.Require().True(s.Run("Acknowledge packets on Ethereum", func() {
		var ackRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{ackTxHash},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			ackRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &ics26Address, ackRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))
			s.T().Logf("Multicall ack %d packets gas used: %d", numOfTransfers, receipt.GasUsed)

			// Verify the ack packet event exists
			_, err = e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseAckPacket)
			s.Require().NoError(err)
		}))

		s.Require().NoError(s.solidityFixtureGenerator.GenerateAndSaveSolidityFixture(
			fmt.Sprintf("acknowledgeMultiPacket_%d-%s.json", numOfTransfers, proofType.String()),
			s.contractAddresses.Erc20, ackRelayTx, sendPacket,
		))

		s.Require().True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(new(big.Int).Sub(testvalues.StartingERC20Balance, totalTransferAmount), userBalance)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Equal(totalTransferAmount, escrowBalance)
		}))
	}))

	var returnSendTxHash []byte
	s.Require().True(s.Run("Transfer tokens back from Cosmos chain", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		ibcCoin := sdk.NewCoin(denomOnCosmos.Path(), sdkmath.NewIntFromBigInt(transferAmount))

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    ibcCoin.Denom,
			Amount:   ibcCoin.Amount.String(),
			Sender:   cosmosUserWallet.FormattedAddress(),
			Receiver: strings.ToLower(ethereumUserAddress.Hex()),
			Memo:     "",
		}
		encodedPayload, err := transfertypes.EncodeABIFungibleTokenPacketData(&transferPayload)
		s.Require().NoError(err)

		payload := channeltypesv2.Payload{
			SourcePort:      transfertypes.PortID,
			DestinationPort: transfertypes.PortID,
			Version:         transfertypes.V1,
			Encoding:        transfertypes.EncodingABI,
			Value:           encodedPayload,
		}

		transferMsgs := make([]sdk.Msg, numOfTransfers)
		for i := range numOfTransfers {
			transferMsgs[i] = &channeltypesv2.MsgSendPacket{
				SourceClient:     s.getEthLcClientIDOnCosmos(),
				TimeoutTimestamp: timeout,
				Payloads: []channeltypesv2.Payload{
					payload,
				},
				Signer: cosmosUserWallet.FormattedAddress(),
			}
		}

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, 20_000_000, transferMsgs...)
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		returnSendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			// User balance on Cosmos chain
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(sdkmath.ZeroInt(), resp.Balance.Amount)
			s.Require().Equal(denomOnCosmos.IBCDenom(), resp.Balance.Denom)
		}))
	}))

	var returnAckTxHash []byte
	s.Require().True(s.Run(fmt.Sprintf("Receive %d packets on Ethereum", numOfTransfers), func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{returnSendTxHash},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			recvRelayTx = resp.Tx
		}))

		var returnPacket ics26router.IICS26RouterMsgsPacket
		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &ics26Address, recvRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))
			s.T().Logf("Multicall receive %d packets gas used: %d", numOfTransfers, receipt.GasUsed)

			returnWriteAckEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseWriteAcknowledgement)
			s.Require().NoError(err)

			returnPacket = returnWriteAckEvent.Packet
			returnAckTxHash = receipt.TxHash.Bytes()
		}))

		s.Require().NoError(s.solidityFixtureGenerator.GenerateAndSaveSolidityFixture(
			fmt.Sprintf("receiveMultiPacket_%d-%s.json", numOfTransfers, proofType.String()),
			s.contractAddresses.Erc20, recvRelayTx, returnPacket,
		))

		s.True(s.Run("Verify balances on Ethereum", func() {
			// User balance should be back to the starting point
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(testvalues.StartingERC20Balance, userBalance)

			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Zero(escrowBalance.Int64())
		}))
	}))

	s.Require().True(s.Run("Acknowledge packets on Cosmos chain", func() {
		s.Require().True(s.Run("Verify commitments exists", func() {
			for i := range numOfTransfers {
				resp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
					ClientId: s.getEthLcClientIDOnCosmos(),
					Sequence: uint64(i) + 1,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Commitment)
			}
		}))

		var ackRelayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{returnAckTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			ackRelayTxBodyBz = resp.Tx

			s.wasmFixtureGenerator.AddFixtureStep("ack_packets", ethereumtypes.RelayerMessages{
				RelayerTxBody: hex.EncodeToString(ackRelayTxBodyBz),
			})
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, ackRelayTxBodyBz)

			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)
		}))

		s.Require().True(s.Run("Verify commitments removed", func() {
			for i := range numOfTransfers {
				_, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
					ClientId: s.getEthLcClientIDOnCosmos(),
					Sequence: uint64(i) + 1,
				})
				s.Require().ErrorContains(err, "packet commitment hash not found")
			}
		}))
	}))
}

func (s *IbcEurekaTestSuite) Test_ICS20TransferERC20TokenFromEthereumToCosmosAndBackFails() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenFromEthereumToCosmosAndBackFailsTest(ctx, proofType, 1, big.NewInt(testvalues.TransferAmount))
}

func (s *IbcEurekaTestSuite) ICS20TransferERC20TokenFromEthereumToCosmosAndBackFailsTest(
	ctx context.Context, proofType types.SupportedProofType, numOfTransfers int, transferAmount *big.Int,
) {
	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	totalTransferAmount := new(big.Int).Mul(transferAmount, big.NewInt(int64(numOfTransfers)))
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	ics20transferAbi, err := abi.JSON(strings.NewReader(ics20transfer.ContractABI))
	s.Require().NoError(err)

	s.Require().True(s.Run("Approve the ICS20Transfer.sol contract to spend the erc20 tokens", func() {
		tx, err := s.erc20Contract.Approve(s.GetTransactOpts(s.key, eth), ics20Address, totalTransferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		allowance, err := s.erc20Contract.Allowance(nil, ethereumUserAddress, ics20Address)
		s.Require().NoError(err)
		s.Require().Equal(totalTransferAmount, allowance)
	}))

	var ethSendTxHash []byte
	s.Require().True(s.Run(fmt.Sprintf("Send %d transfers on Ethereum", numOfTransfers), func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		transferMulticall := make([][]byte, numOfTransfers)

		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			SourceClient:     testvalues.CustomClientID,
			Memo:             "",
		}

		encodedMsg, err := ics20transferAbi.Pack("sendTransfer", msgSendPacket)
		s.Require().NoError(err)
		for i := range numOfTransfers {
			transferMulticall[i] = encodedMsg
		}

		tx, err := s.ics20Contract.Multicall(s.GetTransactOpts(s.key, eth), transferMulticall)
		s.Require().NoError(err)
		_, err = eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		ethSendTxHash = tx.Hash().Bytes()
	}))

	var ackTxHash []byte
	s.Require().True(s.Run("Receive packets on Cosmos chain", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)

			relayTxBodyBz = resp.Tx
			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp, err := s.BroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, relayTxBodyBz)
			s.Require().NoError(err)

			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
		}))
	}))

	s.Require().True(s.Run("Transfer tokens back from Cosmos chain with invalid receiver should fail", func() {
		denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, s.getEthLcClientIDOnCosmos()))
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		ibcCoin := sdk.NewCoin(denomOnCosmos.Path(), sdkmath.NewIntFromBigInt(transferAmount))

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    ibcCoin.Denom,
			Amount:   ibcCoin.Amount.String(),
			Sender:   cosmosUserWallet.FormattedAddress(),
			Receiver: "invalid_receiver_address",
			Memo:     "",
		}
		encodedPayload, err := transfertypes.EncodeABIFungibleTokenPacketData(&transferPayload)
		s.Require().NoError(err)

		payload := channeltypesv2.Payload{
			SourcePort:      transfertypes.PortID,
			DestinationPort: transfertypes.PortID,
			Version:         transfertypes.V1,
			Encoding:        transfertypes.EncodingABI,
			Value:           encodedPayload,
		}

		transferMsg := &channeltypesv2.MsgSendPacket{
			SourceClient:     s.getEthLcClientIDOnCosmos(),
			TimeoutTimestamp: timeout,
			Payloads: []channeltypesv2.Payload{
				payload,
			},
			Signer: cosmosUserWallet.FormattedAddress(),
		}

		var relayTxHash []byte
		s.Require().True(s.Run("Broadcast transfer msgs", func() {
			resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, 20_000_000, transferMsg)
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.TxHash)

			relayTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
		}))

		var transferRelayTxHashBz []byte
		s.Require().True(s.Run("Relay transfer to eth", func() {
			var relayTxBodyBz []byte
			s.Require().True(s.Run("Retrieve relay tx", func() {
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    simd.Config().ChainID,
					DstChain:    eth.ChainID.String(),
					SourceTxIds: [][]byte{relayTxHash},
					SrcClientId: s.getEthLcClientIDOnCosmos(),
					DstClientId: testvalues.CustomClientID,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				s.Require().NotEmpty(resp.Address)

				relayTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run("Submit relay tx to eth", func() {
				receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &ics26Address, relayTxBodyBz)
				s.Require().NoError(err)

				transferRelayTxHashBz = receipt.TxHash.Bytes()
			}))
		}))

		var ackRelayTxBodyBz []byte
		s.Require().True(s.Run("Acknowledge packet to Cosmos chain", func() {
			s.Require().True(s.Run("Retrieve relay tx", func() {
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    eth.ChainID.String(),
					DstChain:    simd.Config().ChainID,
					SourceTxIds: [][]byte{transferRelayTxHashBz},
					SrcClientId: testvalues.CustomClientID,
					DstClientId: s.getEthLcClientIDOnCosmos(),
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)

				ackRelayTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run("Submit acknowledgement relay tx to Cosmos", func() {
				resp, err := s.BroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, ackRelayTxBodyBz)
				s.Require().NoError(err)

				ackTxHash, err = hex.DecodeString(resp.TxHash)
				s.Require().NoError(err)
				s.Require().NotEmpty(ackTxHash)
			}))
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			// Vouchers should be restored to the user's balance
			s.Require().Equal(totalTransferAmount, resp.Balance.Amount.BigInt())
			s.Require().Equal(denomOnCosmos.IBCDenom(), resp.Balance.Denom)
		}))
	}))
}

func (s *IbcEurekaTestSuite) Test_ICS20TransferNativeCosmosCoinsToEthereumAndBack() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferNativeCosmosCoinsToEthereumAndBackTest(ctx, proofType, big.NewInt(testvalues.TransferAmount))
}

// ICS20TransferNativeCosmosCoinsToEthereumAndBackTest tests the ICS20 transfer functionality
// by transferring native coins from a Cosmos chain to Ethereum and back
func (s *IbcEurekaTestSuite) ICS20TransferNativeCosmosCoinsToEthereumAndBackTest(ctx context.Context, pt types.SupportedProofType, transferAmount *big.Int) {
	s.SetupSuite(ctx, pt)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	transferCoin := sdk.NewCoin(simd.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()
	sendMemo := "nativesend"

	var (
		cosmosSendTxHash []byte
		ibcERC20         *ibcerc20.Contract
		ibcERC20Address  ethcommon.Address
	)
	s.Require().True(s.Run("Send transfer on Cosmos chain", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    transferCoin.Denom,
			Amount:   transferCoin.Amount.String(),
			Sender:   cosmosUserAddress,
			Receiver: strings.ToLower(ethereumUserAddress.Hex()),
			Memo:     sendMemo,
		}
		encodedPayload, err := transfertypes.EncodeABIFungibleTokenPacketData(&transferPayload)
		s.Require().NoError(err)

		payload := channeltypesv2.Payload{
			SourcePort:      transfertypes.PortID,
			DestinationPort: transfertypes.PortID,
			Version:         transfertypes.V1,
			Encoding:        transfertypes.EncodingABI,
			Value:           encodedPayload,
		}
		msgSendPacket := channeltypesv2.MsgSendPacket{
			SourceClient:     s.getEthLcClientIDOnCosmos(),
			TimeoutTimestamp: timeout,
			Payloads: []channeltypesv2.Payload{
				payload,
			},
			Signer: cosmosUserWallet.FormattedAddress(),
		}

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, 200_000, &msgSendPacket)
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		cosmosSendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			// Check the balance of UserB
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   transferCoin.Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance-testvalues.TransferAmount, resp.Balance.Amount.Int64())
		}))
	}))

	var ackTxHash []byte
	s.Require().True(s.Run("Receive packet on Ethereum", func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{cosmosSendTxHash},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			recvRelayTx = resp.Tx
		}))

		var packet ics26router.IICS26RouterMsgsPacket
		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, recvRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			ethReceiveAckEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseWriteAcknowledgement)
			s.Require().NoError(err)

			packet = ethReceiveAckEvent.Packet
			ackTxHash = receipt.TxHash.Bytes()
		}))

		s.Require().NoError(s.solidityFixtureGenerator.GenerateAndSaveSolidityFixture(
			fmt.Sprintf("receiveNativePacket-%s.json", pt.String()),
			s.contractAddresses.Erc20, recvRelayTx, packet,
		))

		// Recreate the full denom path
		denomOnEthereum := transfertypes.NewDenom(transferCoin.Denom, transfertypes.NewHop(packet.Payloads[0].DestPort, packet.DestClient))

		var err error
		ibcERC20Address, err = s.ics20Contract.IbcERC20Contract(nil, denomOnEthereum.Path())
		s.Require().NoError(err)

		ibcERC20, err = ibcerc20.NewContract(ibcERC20Address, eth.RPCClient)
		s.Require().NoError(err)

		actualDenom, err := ibcERC20.Name(nil)
		s.Require().NoError(err)
		s.Require().Equal(denomOnEthereum.Path(), actualDenom)

		actualSymbol, err := ibcERC20.Symbol(nil)
		s.Require().NoError(err)
		s.Require().Equal(denomOnEthereum.Path(), actualSymbol)

		actualFullDenom, err := ibcERC20.FullDenomPath(nil)
		s.Require().NoError(err)
		s.Require().Equal(denomOnEthereum.Path(), actualFullDenom)

		s.True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(transferAmount, userBalance)

			// ICS20 contract balance on Ethereum
			ics20TransferBalance, err := ibcERC20.BalanceOf(nil, ics20Address)
			s.Require().NoError(err)
			s.Require().Zero(ics20TransferBalance.Int64())
		}))
	}))

	s.Require().True(s.Run("Acknowledge packet on Cosmos chain", func() {
		s.Require().True(s.Run("Verify commitments exists", func() {
			resp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
				ClientId: s.getEthLcClientIDOnCosmos(),
				Sequence: 1,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Commitment)
		}))

		var ackRelayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{ackTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			ackRelayTxBodyBz = resp.Tx

			s.wasmFixtureGenerator.AddFixtureStep("ack_packets", ethereumtypes.RelayerMessages{
				RelayerTxBody: hex.EncodeToString(ackRelayTxBodyBz),
			})
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, ackRelayTxBodyBz)

			var err error
			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)
		}))

		s.Require().True(s.Run("Verify commitments removed", func() {
			_, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
				ClientId: s.getEthLcClientIDOnCosmos(),
				Sequence: 1,
			})
			s.Require().ErrorContains(err, "packet commitment hash not found")
		}))
	}))

	s.Require().True(s.Run("Approve the ICS20Transfer.sol contract to spend the erc20 tokens", func() {
		tx, err := ibcERC20.Approve(s.GetTransactOpts(s.key, eth), ics20Address, transferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		allowance, err := ibcERC20.Allowance(nil, ethereumUserAddress, ics20Address)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount, allowance)
	}))

	var ethSendTxHash []byte
	s.Require().True(s.Run("Transfer tokens back from Ethereum", func() {
		returnMemo := "testreturnmemo"
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            ibcERC20Address,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			SourceClient:     testvalues.CustomClientID,
			Memo:             returnMemo,
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		ethSendTxHash = tx.Hash().Bytes()

		sendPacketEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseSendPacket)
		s.Require().NoError(err)
		s.Require().Equal(uint64(1), sendPacketEvent.Packet.Sequence)
		s.Require().Equal(timeout, sendPacketEvent.Packet.TimeoutTimestamp)
		s.Require().Equal(transfertypes.PortID, sendPacketEvent.Packet.Payloads[0].SourcePort)
		s.Require().Equal(testvalues.CustomClientID, sendPacketEvent.Packet.SourceClient)
		s.Require().Equal(transfertypes.PortID, sendPacketEvent.Packet.Payloads[0].DestPort)
		s.Require().Equal(s.getEthLcClientIDOnCosmos(), sendPacketEvent.Packet.DestClient)
		s.Require().Equal(transfertypes.V1, sendPacketEvent.Packet.Payloads[0].Version)
		s.Require().Equal(transfertypes.EncodingABI, sendPacketEvent.Packet.Payloads[0].Encoding)

		s.True(s.Run("Verify balances on Ethereum", func() {
			userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Zero(userBalance.Int64())

			// the whole balance should have been burned
			ics20TransferBalance, err := ibcERC20.BalanceOf(nil, ics20Address)
			s.Require().NoError(err)
			s.Require().Zero(ics20TransferBalance.Int64())
		}))
	}))

	var returnAckTxHash []byte
	s.Require().True(s.Run("Receive packet on Cosmos chain", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx

			s.wasmFixtureGenerator.AddFixtureStep("receive_packets", ethereumtypes.RelayerMessages{
				RelayerTxBody: hex.EncodeToString(relayTxBodyBz),
			})
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, relayTxBodyBz)

			var err error
			returnAckTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(returnAckTxHash)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			// Check the balance of UserB
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   transferCoin.Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance, resp.Balance.Amount.Int64())
		}))
	}))

	s.Require().True(s.Run("Acknowledge packet on Ethereum", func() {
		s.Require().True(s.Run("Verify commitment exists", func() {
			packetCommitmentPath := ibchostv2.PacketCommitmentKey(testvalues.CustomClientID, 1)
			var ethPath [32]byte
			copy(ethPath[:], crypto.Keccak256(packetCommitmentPath))

			resp, err := s.ics26Contract.GetCommitment(nil, ethPath)
			s.Require().NoError(err)
			s.Require().NotZero(resp)
		}))

		var ackRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{returnAckTxHash},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			ackRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, ackRelayTx)
			s.Require().NoError(err)

			// Verify the ack packet event exists
			_, err = e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseAckPacket)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Verify commitment removed", func() {
			packetCommitmentPath := ibchostv2.PacketCommitmentKey(testvalues.CustomClientID, 1)
			var ethPath [32]byte
			copy(ethPath[:], crypto.Keccak256(packetCommitmentPath))

			resp, err := s.ics26Contract.GetCommitment(nil, ethPath)
			s.Require().NoError(err)
			s.Require().Zero(resp)
		}))

		s.Require().True(s.Run("Verify balances on Ethereum after ack", func() {
			// User balance should still be zero
			userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Zero(userBalance.Int64())

			// ICS20 contract balance should still be zero
			ics20TransferBalance, err := ibcERC20.BalanceOf(nil, ics20Address)
			s.Require().NoError(err)
			s.Require().Zero(ics20TransferBalance.Int64())
		}))
	}))
}

func (s *IbcEurekaTestSuite) Test_TimeoutPacketFromEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredICS20TimeoutPacketFromEthereumTest(ctx, proofType, 1, nil)
}

func (s *IbcEurekaTestSuite) Test_10_TimeoutPacketFromEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredICS20TimeoutPacketFromEthereumTest(ctx, proofType, 10, nil)
}

func (s *IbcEurekaTestSuite) Test_5_TimeoutPacketFromEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredICS20TimeoutPacketFromEthereumTest(ctx, proofType, 5, []uint64{1, 2, 3, 4, 5})
}

func (s *IbcEurekaTestSuite) Test_5_FilteredTimeoutPacketFromEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredICS20TimeoutPacketFromEthereumTest(ctx, proofType, 5, []uint64{2, 3})
}

func (s *IbcEurekaTestSuite) FilteredICS20TimeoutPacketFromEthereumTest(
	ctx context.Context, pt types.SupportedProofType, numOfTransfers int, timeoutFilter []uint64,
) {
	s.Require().GreaterOrEqual(numOfTransfers, len(timeoutFilter))
	s.Require().Greater(numOfTransfers, 0)

	s.SetupSuite(ctx, pt)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	transferAmount := big.NewInt(testvalues.TransferAmount)
	totalTransferAmount := new(big.Int).Mul(transferAmount, big.NewInt(int64(numOfTransfers)))
	var refundedAmount *big.Int
	if len(timeoutFilter) == 0 {
		refundedAmount = totalTransferAmount
	} else {
		refundedAmount = new(big.Int).Mul(transferAmount, big.NewInt(int64(len(timeoutFilter))))
	}
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	var originalBalance *sdk.Coin
	s.Require().True(s.Run("Retrieve original balance", func() {
		denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, s.getEthLcClientIDOnCosmos()))

		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: cosmosUserAddress,
			Denom:   denomOnCosmos.IBCDenom(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.Balance)
		originalBalance = resp.Balance
	}))

	s.Require().True(s.Run("Approve the ICS20Transfer.sol contract to spend the erc20 tokens", func() {
		ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
		tx, err := s.erc20Contract.Approve(s.GetTransactOpts(s.key, eth), ics20Address, totalTransferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		allowance, err := s.erc20Contract.Allowance(nil, ethereumUserAddress, ics20Address)
		s.Require().NoError(err)
		s.Require().Equal(totalTransferAmount, allowance)
	}))

	var (
		ethSendTxHashes [][]byte
		sendPacket      ics26router.IICS26RouterMsgsPacket
		escrowAddress   ethcommon.Address
	)
	s.Require().True(s.Run("Send packets on Ethereum", func() {
		for i := range numOfTransfers {
			timeout := uint64(time.Now().Add(30 * time.Second).Unix())
			msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
				Denom:            erc20Address,
				Amount:           transferAmount,
				Receiver:         cosmosUserAddress,
				TimeoutTimestamp: timeout,
				SourceClient:     testvalues.CustomClientID,
				Memo:             "testmemo",
			}

			tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
			s.Require().NoError(err)

			receipt, err := eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)

			// We use the first packet in fixture generation
			if i == 0 && s.solidityFixtureGenerator.Enabled {
				sendPacketEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseSendPacket)
				s.Require().NoError(err)
				sendPacket = sendPacketEvent.Packet
			}

			ethSendTxHashes = append(ethSendTxHashes, tx.Hash().Bytes())
		}

		s.Require().True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(new(big.Int).Sub(testvalues.StartingERC20Balance, totalTransferAmount), userBalance)

			// Get the escrow address
			escrowAddress, err = s.ics20Contract.GetEscrow(nil, testvalues.CustomClientID)
			s.Require().NoError(err)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Equal(totalTransferAmount, escrowBalance)
		}))
	}))

	var txBodyBz []byte
	s.Require().True(s.Run("Prefetch relay tx", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    eth.ChainID.String(),
			DstChain:    simd.Config().ChainID,
			SourceTxIds: ethSendTxHashes,
			SrcClientId: testvalues.CustomClientID,
			DstClientId: s.getEthLcClientIDOnCosmos(),
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)
		s.Require().Empty(resp.Address)

		txBodyBz = resp.Tx
	}))

	s.Require().True(s.Run("Wait for timeout", func() {
		time.Sleep(45 * time.Second)
	}))

	s.True(s.Run("Timeout packets on Ethereum", func() {
		var timeoutRelayTx []byte
		s.Require().True(s.Run("Retrieve timeout tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           simd.Config().ChainID,
				DstChain:           eth.ChainID.String(),
				TimeoutTxIds:       ethSendTxHashes,
				SrcClientId:        s.getEthLcClientIDOnCosmos(),
				DstClientId:        testvalues.CustomClientID,
				DstPacketSequences: timeoutFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			timeoutRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			_, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, timeoutRelayTx)
			s.Require().NoError(err)
		}))

		if s.solidityFixtureGenerator.Enabled {
			s.Require().Zero(len(timeoutFilter))
			s.Require().NoError(s.solidityFixtureGenerator.GenerateAndSaveSolidityFixture(
				fmt.Sprintf("timeoutMultiPacket_%d-%s.json", numOfTransfers, pt.String()),
				s.contractAddresses.Erc20, timeoutRelayTx, sendPacket,
			))
		}

		s.Require().True(s.Run("Verify balances on Ethereum", func() {
			// Expected balance
			netTransferAmount := new(big.Int).Sub(totalTransferAmount, refundedAmount)
			expectedUserBalance := new(big.Int).Sub(testvalues.StartingERC20Balance, netTransferAmount)

			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(expectedUserBalance, userBalance)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Equal(escrowBalance.Int64(), netTransferAmount.Int64())
		}))

		s.Require().True(s.Run("Verify no balance on Cosmos chain", func() {
			denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, s.getEthLcClientIDOnCosmos()))

			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().Equal(originalBalance, resp.Balance)
		}))
	}))

	s.Require().True(s.Run("Constructing relay packet after timeout should fail", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    eth.ChainID.String(),
			DstChain:    simd.Config().ChainID,
			SourceTxIds: ethSendTxHashes,
			SrcClientId: testvalues.CustomClientID,
			DstClientId: s.getEthLcClientIDOnCosmos(),
		})
		// TODO: https://github.com/cosmos/solidity-ibc-eureka/issues/363
		// The following assertions should be Error and Nil, but the relayer returns a valid response currently.
		s.Require().NoError(err)
		s.Require().NotNil(resp)
	}))

	s.Require().True(s.Run("Receive packets on Cosmos chain after timeout", func() {
		resp, err := s.BroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, txBodyBz)
		s.Require().Error(err)
		s.Require().Nil(resp)
	}))
}

func (s *IbcEurekaTestSuite) Test_ErrorAckToEthereum() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20ErrorAckToEthereumTest(ctx, proofType)
}

func (s *IbcEurekaTestSuite) ICS20ErrorAckToEthereumTest(
	ctx context.Context, pt types.SupportedProofType,
) {
	s.SetupSuite(ctx, pt)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]
	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	transferAmount := big.NewInt(testvalues.TransferAmount)
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)

	s.Require().True(s.Run("Approve the ICS20Transfer.sol contract to spend the erc20 tokens", func() {
		ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
		tx, err := s.erc20Contract.Approve(s.GetTransactOpts(s.key, eth), ics20Address, transferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		allowance, err := s.erc20Contract.Allowance(nil, ethereumUserAddress, ics20Address)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount, allowance)
	}))

	var (
		ethSendTxHash []byte
		escrowAddress ethcommon.Address
	)
	s.Require().True(s.Run("Send transfer on Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		// Send a transfer to an invalid Cosmos address
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			Amount:           transferAmount,
			Receiver:         ibctesting.InvalidID,
			TimeoutTimestamp: timeout,
			SourceClient:     testvalues.CustomClientID,
			Memo:             "",
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		ethSendTxHash = tx.Hash().Bytes()

		s.Require().True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(new(big.Int).Sub(testvalues.StartingERC20Balance, transferAmount), userBalance)

			// Get the escrow address
			escrowAddress, err = s.ics20Contract.GetEscrow(nil, testvalues.CustomClientID)
			s.Require().NoError(err)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Equal(transferAmount, escrowBalance)
		}))
	}))

	var (
		denomOnCosmos transfertypes.Denom
		ackTxHash     []byte
	)
	s.Require().True(s.Run("Receive packets on Cosmos chain", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, relayTxBodyBz)

			var err error
			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)
		}))

		s.Require().True(s.Run("Verify no balance on Cosmos chain", func() {
			denomOnCosmos = transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, s.getEthLcClientIDOnCosmos()))

			_, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: ibctesting.InvalidID,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().Error(err)
		}))
	}))

	s.Require().True(s.Run("Acknowledge packets on Ethereum", func() {
		var ackRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{ackTxHash},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			ackRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, ackRelayTx)
			s.Require().NoError(err)

			// Verify the ack packet event exists
			_, err = e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseAckPacket)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(testvalues.StartingERC20Balance, userBalance)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Zero(escrowBalance.Int64())
		}))
	}))
}

func (s *IbcEurekaTestSuite) Test_TimeoutPacketFromCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredICS20TimeoutFromCosmosTimeoutTest(ctx, proofType, 1, nil)
}

func (s *IbcEurekaTestSuite) Test_10_TimeoutPacketFromCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredICS20TimeoutFromCosmosTimeoutTest(ctx, proofType, 10, []uint64{1, 2, 3, 4, 5, 6, 7, 8, 9, 10})
}

func (s *IbcEurekaTestSuite) Test_10_FilteredTimeoutPacketFromCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredICS20TimeoutFromCosmosTimeoutTest(ctx, proofType, 10, []uint64{2, 4, 6, 8, 10})
}

func (s *IbcEurekaTestSuite) FilteredICS20TimeoutFromCosmosTimeoutTest(
	ctx context.Context, proofType types.SupportedProofType, numOfTransfers int, timeoutFilter []uint64,
) {
	s.Require().GreaterOrEqual(numOfTransfers, len(timeoutFilter))
	s.Require().Greater(numOfTransfers, 0)

	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	transferAmount := big.NewInt(testvalues.TransferAmount)
	totalTransferAmount := big.NewInt(testvalues.TransferAmount * int64(numOfTransfers))
	if totalTransferAmount.Int64() > testvalues.InitialBalance {
		s.FailNow("Total transfer amount exceeds the initial balance")
	}
	var refundedAmount *big.Int
	if len(timeoutFilter) == 0 {
		refundedAmount = totalTransferAmount
	} else {
		refundedAmount = big.NewInt(testvalues.TransferAmount * int64(len(timeoutFilter)))
	}
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()
	sendMemo := "nonnativesend"

	var (
		transferCoin sdk.Coin
		sendTxHashes [][]byte
	)
	s.Require().True(s.Run("Send transfers on Cosmos chain", func() {
		for range numOfTransfers {
			timeout := uint64(time.Now().Add(20 * time.Second).Unix())
			transferCoin = sdk.NewCoin(simd.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))

			transferPayload := transfertypes.FungibleTokenPacketData{
				Denom:    transferCoin.Denom,
				Amount:   transferCoin.Amount.String(),
				Sender:   cosmosUserAddress,
				Receiver: strings.ToLower(ethereumUserAddress.Hex()),
				Memo:     sendMemo,
			}
			encodedPayload, err := transfertypes.EncodeABIFungibleTokenPacketData(&transferPayload)
			s.Require().NoError(err)

			payload := channeltypesv2.Payload{
				SourcePort:      transfertypes.PortID,
				DestinationPort: transfertypes.PortID,
				Version:         transfertypes.V1,
				Encoding:        transfertypes.EncodingABI,
				Value:           encodedPayload,
			}
			msgSendPacket := channeltypesv2.MsgSendPacket{
				SourceClient:     s.getEthLcClientIDOnCosmos(),
				TimeoutTimestamp: timeout,
				Payloads: []channeltypesv2.Payload{
					payload,
				},
				Signer: cosmosUserWallet.FormattedAddress(),
			}

			resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, 200_000, &msgSendPacket)
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.TxHash)

			txHash, err := hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)

			sendTxHashes = append(sendTxHashes, txHash)
		}

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			// Check the balance of UserB
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   transferCoin.Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance-totalTransferAmount.Int64(), resp.Balance.Amount.Int64())
		}))
	}))

	var txBodyBz []byte
	s.Require().True(s.Run("Prefetch relay tx", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    eth.ChainID.String(),
			SourceTxIds: sendTxHashes,
			SrcClientId: s.getEthLcClientIDOnCosmos(),
			DstClientId: testvalues.CustomClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		txBodyBz = resp.Tx
	}))

	s.Require().True(s.Run("Wait for timeout", func() {
		time.Sleep(60 * time.Second)
	}))

	s.Require().True(s.Run("Timeout packets on Cosmos chain", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx to Cosmos chain", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           eth.ChainID.String(),
				DstChain:           simd.Config().ChainID,
				TimeoutTxIds:       sendTxHashes,
				SrcClientId:        testvalues.CustomClientID,
				DstClientId:        s.getEthLcClientIDOnCosmos(),
				DstPacketSequences: timeoutFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx

			s.wasmFixtureGenerator.AddFixtureStep("timeout_packets", ethereumtypes.RelayerMessages{
				RelayerTxBody: hex.EncodeToString(relayTxBodyBz),
			})
		}))

		s.Require().True(s.Run("Broadcast relay tx on Cosmos chain", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, relayTxBodyBz)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			// Check the balance of UserB
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   transferCoin.Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance-totalTransferAmount.Int64()+refundedAmount.Int64(), resp.Balance.Amount.Int64())
		}))
	}))

	// We are skipping the replay attack test on non-PoS mode
	// In PoW mode, the test below will NOT fail as the mock light client accepts all proofs without verification.
	if os.Getenv(testvalues.EnvKeyEthTestnetType) != testvalues.EthTestnetTypePoS {
		s.T().Skip("Skipping replay attack test on non-PoS mode")
	}

	s.Require().True(s.Run("Receive packets on Ethereum after timeout should fail", func() {
		ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
		_, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, txBodyBz)
		s.Require().Error(err)

		// Prove that the erc20 was not created (and by extension, the packet was not received)
		denomOnEthereum := transfertypes.NewDenom(transferCoin.Denom, transfertypes.NewHop(transfertypes.PortID, testvalues.CustomClientID))
		_, err = s.ics20Contract.IbcERC20Contract(nil, denomOnEthereum.Path())
		// Ethereum side did not received the packet, ERC20 contract corresponding to the denom does not exist
		s.Require().Error(err)
	}))
}

func (s *IbcEurekaTestSuite) Test_TimeoutPacketEthRemintsVouchers() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.TimeoutPacketEthRemintsVouchersTest(ctx, proofType)
}

// TimeoutPacketEthRemintsVouchersTest tests that when a transfer of a voucher (Cosmos native -> Eth)
// from Ethereum back to Cosmos times out, the vouchers are reminted on Ethereum.
func (s *IbcEurekaTestSuite) TimeoutPacketEthRemintsVouchersTest(ctx context.Context, pt types.SupportedProofType) {
	s.SetupSuite(ctx, pt)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	transferAmount := big.NewInt(testvalues.TransferAmount)
	transferCoin := sdk.NewCoin(simd.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	var cosmosSendTxHash []byte
	s.Require().True(s.Run("Send native Cosmos coins to Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    transferCoin.Denom,
			Amount:   transferCoin.Amount.String(),
			Sender:   cosmosUserAddress,
			Receiver: strings.ToLower(ethereumUserAddress.Hex()),
			Memo:     "create-voucher",
		}
		encodedPayload, err := transfertypes.EncodeABIFungibleTokenPacketData(&transferPayload)
		s.Require().NoError(err)

		payload := channeltypesv2.Payload{
			SourcePort:      transfertypes.PortID,
			DestinationPort: transfertypes.PortID,
			Version:         transfertypes.V1,
			Encoding:        transfertypes.EncodingABI,
			Value:           encodedPayload,
		}
		msgSendPacket := channeltypesv2.MsgSendPacket{
			SourceClient:     s.getEthLcClientIDOnCosmos(),
			TimeoutTimestamp: timeout,
			Payloads: []channeltypesv2.Payload{
				payload,
			},
			Signer: cosmosUserWallet.FormattedAddress(),
		}

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, 200_000, &msgSendPacket)
		s.Require().NoError(err)
		cosmosSendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
	}))

	var (
		ibcERC20        *ibcerc20.Contract
		ibcERC20Address ethcommon.Address
		ackTxHash       []byte
	)
	s.Require().True(s.Run("Receive packets on Ethereum", func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{cosmosSendTxHash},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			recvRelayTx = resp.Tx
		}))

		var packet ics26router.IICS26RouterMsgsPacket
		s.Require().True(s.Run("Broadcast relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, recvRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

			ethReceiveAckEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseWriteAcknowledgement)
			s.Require().NoError(err)
			packet = ethReceiveAckEvent.Packet
			ackTxHash = receipt.TxHash.Bytes()
		}))

		// Get voucher contract details
		denomOnEthereum := transfertypes.NewDenom(transferCoin.Denom, transfertypes.NewHop(packet.Payloads[0].DestPort, packet.DestClient))
		var err error
		ibcERC20Address, err = s.ics20Contract.IbcERC20Contract(nil, denomOnEthereum.Path())
		s.Require().NoError(err)
		ibcERC20, err = ibcerc20.NewContract(ibcERC20Address, eth.RPCClient)
		s.Require().NoError(err)

		// Verify initial voucher balance
		userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount, userBalance)
	}))

	s.Require().True(s.Run("Acknowledge packets on Cosmos", func() {
		var ackRelayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{ackTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)
			ackRelayTxBodyBz = resp.Tx
		}))
		s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, ackRelayTxBodyBz)
	}))

	var ethTimeoutSendTxHash []byte
	s.Require().True(s.Run("Send voucher from Eth to Cosmos (timeout)", func() {
		// Approve spending
		tx, err := ibcERC20.Approve(s.GetTransactOpts(s.key, eth), ics20Address, transferAmount)
		s.Require().NoError(err)
		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		// Send transfer with short timeout
		timeout := uint64(time.Now().Add(30 * time.Second).Unix())
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            ibcERC20Address, // Sending the voucher back
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			SourceClient:     testvalues.CustomClientID,
			Memo:             "timeout-voucher-eth-cosmos",
		}

		tx, err = s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
		s.Require().NoError(err)
		receipt, err = eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		ethTimeoutSendTxHash = tx.Hash().Bytes()

		// Verify user voucher balance is now zero
		userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
		s.Require().NoError(err)
		s.Require().Zero(userBalance.Int64())
	}))

	// We retrieve the relay tx before the timeout, and will attempt to relay it after the timeout
	var recvRelayTxBodyBz []byte
	s.Require().True(s.Run("Retrieve relay tx before timeout", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    eth.ChainID.String(),
			DstChain:    simd.Config().ChainID,
			SourceTxIds: [][]byte{ethTimeoutSendTxHash},
			SrcClientId: testvalues.CustomClientID,
			DstClientId: s.getEthLcClientIDOnCosmos(),
		})
		s.Require().NoError(err)
		recvRelayTxBodyBz = resp.Tx
	}))

	s.Require().True(s.Run("Wait for timeout", func() {
		time.Sleep(45 * time.Second)
	}))

	s.Require().True(s.Run("Relay timeout packet to Eth", func() {
		var timeoutRelayTx []byte
		s.Require().True(s.Run("Retrieve timeout tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:     simd.Config().ChainID, // Source of the *timeout* message is Cosmos
				DstChain:     eth.ChainID.String(),  // Destination is Eth
				TimeoutTxIds: [][]byte{ethTimeoutSendTxHash},
				SrcClientId:  s.getEthLcClientIDOnCosmos(),
				DstClientId:  testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			timeoutRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, timeoutRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

			// Verify the timeout packet event exists
			_, err = e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseTimeoutPacket)
			s.Require().NoError(err)
		}))
	}))

	s.Require().True(s.Run("Verify voucher balance restored on Eth", func() {
		userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount, userBalance, "Voucher balance should be restored after timeout")

		// Escrow balance should remain zero (or unchanged if it wasn't zero initially, though it should be)
		escrowBalance, err := ibcERC20.BalanceOf(nil, ics20Address)
		s.Require().NoError(err)
		s.Require().Zero(escrowBalance.Int64(), "Escrow balance should be zero")
	}))

	s.Require().True(s.Run("Verify recvPacket fails on Cosmos after timeout", func() {
		_, err := s.BroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, recvRelayTxBodyBz)
		s.Require().Error(err)
	}))
}

func (s *IbcEurekaTestSuite) Test_TimeoutPacketCosmosRemintsVouchers() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.TimeoutPacketCosmosRemintsVouchersTest(ctx, proofType)
}

// TimeoutPacketCosmosRemintsVouchersTest tests that when a transfer of a voucher (Eth native -> Cosmos)
// from Cosmos back to Ethereum times out, the vouchers are reminted on Cosmos.
func (s *IbcEurekaTestSuite) TimeoutPacketCosmosRemintsVouchersTest(ctx context.Context, pt types.SupportedProofType) {
	s.SetupSuite(ctx, pt)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	transferAmount := big.NewInt(testvalues.TransferAmount)
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	s.Require().True(s.Run("Approve the ICS20Transfer.sol contract to spend the erc20 tokens", func() {
		tx, err := s.erc20Contract.Approve(s.GetTransactOpts(s.key, eth), ics20Address, transferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		allowance, err := s.erc20Contract.Allowance(nil, ethereumUserAddress, ics20Address)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount, allowance)
	}))

	var (
		sendPacket    ics26router.IICS26RouterMsgsPacket
		ethSendTxHash []byte
		escrowAddress ethcommon.Address
	)
	s.Require().True(s.Run("Send ERC20 tokens on Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			SourceClient:     testvalues.CustomClientID,
			Memo:             "create-voucher-cosmos",
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
		s.Require().NoError(err)
		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		ethSendTxHash = tx.Hash().Bytes()

		sendPacketEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseSendPacket)
		s.Require().NoError(err)
		sendPacket = sendPacketEvent.Packet
		s.Require().Equal(uint64(1), sendPacket.Sequence)
		s.Require().Equal(timeout, sendPacket.TimeoutTimestamp)
		s.Require().Len(sendPacket.Payloads, 1)
		s.Require().Equal(transfertypes.PortID, sendPacket.Payloads[0].SourcePort)
		s.Require().Equal(testvalues.CustomClientID, sendPacket.SourceClient)
		s.Require().Equal(transfertypes.PortID, sendPacket.Payloads[0].DestPort)
		s.Require().Equal(s.getEthLcClientIDOnCosmos(), sendPacket.DestClient)
		s.Require().Equal(transfertypes.V1, sendPacket.Payloads[0].Version)
		s.Require().Equal(transfertypes.EncodingABI, sendPacket.Payloads[0].Encoding)

		s.True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(new(big.Int).Sub(testvalues.StartingERC20Balance, transferAmount), userBalance)

			// Get the escrow address
			escrowAddress, err = s.ics20Contract.GetEscrow(nil, testvalues.CustomClientID)
			s.Require().NoError(err)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Equal(transferAmount, escrowBalance)
		}))
	}))

	var (
		denomOnCosmos transfertypes.Denom
		ackTxHash     []byte
	)
	s.Require().True(s.Run("Receive packets on Cosmos chain", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, relayTxBodyBz)
			var err error
			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)

			denomOnCosmos = transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, s.getEthLcClientIDOnCosmos()))

			// Verify initial voucher balance
			balanceResp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().Equal(transferAmount, balanceResp.Balance.Amount.BigInt())
		}))
	}))

	s.Require().True(s.Run("Acknowledge packets on Ethereum", func() {
		var ackRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{ackTxHash},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			ackRelayTx = resp.Tx
		}))
		receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &ics26Address, ackRelayTx)
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

		// Verify the ack packet event exists
		_, err = e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseAckPacket)
		s.Require().NoError(err)
	}))

	var cosmosTimeoutSendTxHash []byte
	s.Require().True(s.Run("Send voucher from Cosmos to Eth (timeout)", func() {
		timeout := uint64(time.Now().Add(30 * time.Second).Unix())
		ibcCoin := sdk.NewCoin(denomOnCosmos.Path(), sdkmath.NewIntFromBigInt(transferAmount))

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    ibcCoin.Denom, // Sending the voucher denom
			Amount:   ibcCoin.Amount.String(),
			Sender:   cosmosUserAddress,
			Receiver: strings.ToLower(ethereumUserAddress.Hex()),
			Memo:     "timeout-voucher-cosmos-eth",
		}
		encodedPayload, err := transfertypes.EncodeABIFungibleTokenPacketData(&transferPayload)
		s.Require().NoError(err)

		payload := channeltypesv2.Payload{
			SourcePort:      transfertypes.PortID,
			DestinationPort: transfertypes.PortID,
			Version:         transfertypes.V1,
			Encoding:        transfertypes.EncodingABI,
			Value:           encodedPayload,
		}
		msgSendPacket := channeltypesv2.MsgSendPacket{
			SourceClient:     s.getEthLcClientIDOnCosmos(),
			TimeoutTimestamp: timeout,
			Payloads: []channeltypesv2.Payload{
				payload,
			},
			Signer: cosmosUserWallet.FormattedAddress(),
		}

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, 200_000, &msgSendPacket)
		s.Require().NoError(err)
		cosmosTimeoutSendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)

		// Verify user voucher balance is now zero
		balanceResp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: cosmosUserAddress,
			Denom:   denomOnCosmos.IBCDenom(),
		})
		s.Require().NoError(err)
		s.Require().Zero(balanceResp.Balance.Amount.Int64())
	}))

	// We retrieve the relay tx before the timeout, and will attempt to relay it after the timeout
	var recvRelayTx []byte
	s.Require().True(s.Run("Retrieve relay tx before timeout", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    eth.ChainID.String(),
			SourceTxIds: [][]byte{cosmosTimeoutSendTxHash},
			SrcClientId: s.getEthLcClientIDOnCosmos(),
			DstClientId: testvalues.CustomClientID,
		})
		s.Require().NoError(err)
		recvRelayTx = resp.Tx
	}))

	s.Require().True(s.Run("Wait for timeout", func() {
		time.Sleep(45 * time.Second)
	}))

	s.Require().True(s.Run("Relay timeout packet to Cosmos", func() {
		var timeoutRelayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve timeout tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:     eth.ChainID.String(),  // Source of the *timeout* message is Eth
				DstChain:     simd.Config().ChainID, // Destination is Cosmos
				TimeoutTxIds: [][]byte{cosmosTimeoutSendTxHash},
				SrcClientId:  testvalues.CustomClientID,
				DstClientId:  s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err)
			timeoutRelayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, timeoutRelayTxBodyBz)

			// Verify voucher balance is restored after timeout
			balanceResp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().Equal(transferAmount, balanceResp.Balance.Amount.BigInt(), "Voucher balance should be restored after timeout")
		}))
	}))

	s.Require().True(s.Run("Verify recvPacket fails on Eth", func() {
		_, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, recvRelayTx)
		s.Require().Error(err)
	}))
}
