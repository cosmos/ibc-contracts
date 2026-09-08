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

	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"
	ibchostv2 "github.com/cosmos/ibc-go/v11/modules/core/24-host/v2"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics20transfer"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/attestor"
	cosmoshelper "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/erc20"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

const (
	// MultiAttestorClientOnEth is the client ID deployed on Ethereum that tracks Cosmos state
	MultiAttestorClientOnEth = "multi-cosmos-1"

	// Config and keystore path templates for multi-attestor tests
	// Eth attestors: read Ethereum state for Eth→Cosmos direction
	MultiAttestorEthConfigPathTemplate   = "/tmp/multi_attestor_eth_%d.toml"
	MultiAttestorEthKeystorePathTemplate = "/tmp/multi_attestor_keystore_eth_%d"
	// Cosmos attestors: read Cosmos state for Cosmos→Eth direction
	MultiAttestorCosmosConfigPathTemplate   = "/tmp/multi_attestor_cosmos_%d.toml"
	MultiAttestorCosmosKeystorePathTemplate = "/tmp/multi_attestor_keystore_cosmos_%d"
)

// MultiAttestorTestSuite tests IBC transfers with multi-attestor aggregation
// between Ethereum and Cosmos chains
type MultiAttestorTestSuite struct {
	e2esuite.TestSuite

	// Ethereum chain contracts and keys
	contractAddresses ethereum.DeployedContracts
	ics26Contract     *ics26router.Contract
	ics20Contract     *ics20transfer.Contract
	erc20Contract     *erc20.Contract
	deployer          *ecdsa.PrivateKey
	userKeyEth        *ecdsa.PrivateKey

	// Cosmos chain user
	cosmosUser ibc.Wallet

	// Ethereum attestors (read Eth state for Eth→Cosmos)
	ethAttestorResult attestor.SetupResult
	// Cosmos attestors (read Cosmos state for Cosmos→Eth)
	cosmosAttestorResult attestor.SetupResult

	// All attestor addresses for registration (including inactive ones)
	allEthAttestorAddresses    []string
	allCosmosAttestorAddresses []string

	// Relayer submitters
	EthRelayerSubmitter  *ecdsa.PrivateKey
	SimdRelayerSubmitter ibc.Wallet

	ProofApiClient proofapitypes.ProofApiServiceClient

	// Test configuration
	totalAttestors  int
	activeAttestors int
	quorumThreshold int
}

func TestWithMultiAttestorTestSuite(t *testing.T) {
	suite.Run(t, new(MultiAttestorTestSuite))
}

func (s *MultiAttestorTestSuite) EthChain() *ethereum.Ethereum {
	return s.Eth.Chains[0]
}

func (s *MultiAttestorTestSuite) CosmosChain() *cosmos.CosmosChain {
	return s.Cosmos.Chains[0]
}

// isNativeAttestor returns true if ETH_LC_ON_COSMOS is attestor-native
func (s *MultiAttestorTestSuite) isNativeAttestor() bool {
	return s.GetEthLightClientType() == testvalues.EthLCOnCosmosTypeAttestorNative
}

// getEthLcClientIDOnCosmos returns the client ID for Ethereum light client on Cosmos
func (s *MultiAttestorTestSuite) getEthLcClientIDOnCosmos() string {
	if s.isNativeAttestor() {
		return testvalues.FirstAttestationsClientID
	}
	return testvalues.FirstWasmClientID
}

func (s *MultiAttestorTestSuite) SetupSuite(ctx context.Context) {
	s.T().Log("Setting up MultiAttestorTestSuite (EVM ↔ Cosmos)")

	// Load configuration from env vars (required)
	s.totalAttestors = mustGetEnvInt(s.T(), testvalues.EnvKeyMultiAttestorCount)
	s.activeAttestors = mustGetEnvInt(s.T(), testvalues.EnvKeyMultiAttestorActive)
	s.quorumThreshold = mustGetEnvInt(s.T(), testvalues.EnvKeyMultiAttestorQuorum)

	s.T().Logf("Multi-attestor config: total=%d, active=%d, quorum=%d",
		s.totalAttestors, s.activeAttestors, s.quorumThreshold)

	if os.Getenv(testvalues.EnvKeyRustLog) == "" {
		os.Setenv(testvalues.EnvKeyRustLog, testvalues.EnvValueRustLog_Info)
	}

	// Configure for single Anvil chain (PoW) + Cosmos
	os.Setenv(testvalues.EnvKeyEthTestnetType, testvalues.EthTestnetTypeAnvil)

	// Set ETH_LC_ON_COSMOS to use native attestor
	ethLcType := os.Getenv(testvalues.EnvKeyEthLcOnCosmos)
	s.Require().Equalf(testvalues.EthLCOnCosmosTypeAttestorNative, ethLcType, "For multi-attestor tests, ETH_LC_ON_COSMOS must be set to '%s'", testvalues.EthLCOnCosmosTypeAttestorNative)
	s.T().Logf("ETH_LC_ON_COSMOS: %s", ethLcType)

	// Force COSMOS_LC_ON_ETH to attestor for this test
	os.Setenv(testvalues.EnvKeyCosmosLcOnEth, testvalues.CosmosLcTypeAttestor)

	s.TestSuite.SetupSuite(ctx)

	err := os.Chdir("../..")
	s.Require().NoError(err)

	eth, simd := s.EthChain(), s.CosmosChain()

	s.T().Logf("Ethereum RPC: %s, Chain ID: %s", eth.RPC, eth.ChainID.String())
	s.T().Logf("Cosmos RPC: %s, Chain ID: %s", simd.GetHostRPCAddress(), simd.Config().ChainID)

	// Create and fund users
	s.Require().True(s.Run("Create and fund users", func() {
		var err error
		s.userKeyEth, err = eth.CreateAndFundUser()
		s.Require().NoError(err)
		s.deployer, err = eth.CreateAndFundUser()
		s.Require().NoError(err)
		s.EthRelayerSubmitter, err = eth.CreateAndFundUser()
		s.Require().NoError(err)

		operatorKey, err := eth.CreateAndFundUser()
		s.Require().NoError(err)
		os.Setenv(testvalues.EnvKeyOperatorPrivateKey, hex.EncodeToString(crypto.FromECDSA(operatorKey)))

		s.cosmosUser = s.CreateAndFundCosmosUser(ctx, simd)
		s.SimdRelayerSubmitter = s.CreateAndFundCosmosUser(ctx, simd)
	}))

	// Deploy contracts on Ethereum
	s.Require().True(s.Run("Deploy contracts on Ethereum", func() {
		os.Setenv(testvalues.EnvKeyEthRPC, eth.RPC)
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

	// Generate keys for ALL attestors (for light client registration)
	s.Require().True(s.Run("Generate all Eth attestor keys", func() {
		var err error
		s.allEthAttestorAddresses, err = attestor.GenerateAttestorKeys(ctx, attestor.GenerateAttestorKeysParams{
			Client:               s.GetDockerClient(),
			NumKeys:              s.totalAttestors,
			KeystorePathTemplate: MultiAttestorEthKeystorePathTemplate,
		})
		s.Require().NoError(err)
		s.T().Logf("Generated %d Eth attestor keys: %v", len(s.allEthAttestorAddresses), s.allEthAttestorAddresses)
	}))

	s.Require().True(s.Run("Generate all Cosmos attestor keys", func() {
		var err error
		s.allCosmosAttestorAddresses, err = attestor.GenerateAttestorKeys(ctx, attestor.GenerateAttestorKeysParams{
			Client:               s.GetDockerClient(),
			NumKeys:              s.totalAttestors,
			KeystorePathTemplate: MultiAttestorCosmosKeystorePathTemplate,
		})
		s.Require().NoError(err)
		s.T().Logf("Generated %d Cosmos attestor keys: %v", len(s.allCosmosAttestorAddresses), s.allCosmosAttestorAddresses)
	}))

	// Start active Eth attestors (read Eth state for Eth→Cosmos direction)
	s.T().Log("Starting active Eth attestors...")
	s.ethAttestorResult = attestor.SetupAttestors(ctx, s.T(), attestor.SetupParams{
		NumAttestors:         s.activeAttestors,
		KeystorePathTemplate: MultiAttestorEthKeystorePathTemplate,
		ChainType:            attestor.ChainTypeEvm,
		AdapterURL:           eth.DockerRPC, // Use Docker internal RPC for container-to-container communication
		RouterAddress:        s.contractAddresses.Ics26Router,
		DockerClient:         s.GetDockerClient(),
		NetworkID:            s.GetNetworkID(),
	})
	s.T().Logf("Started %d of %d Eth attestors", len(s.ethAttestorResult.Addresses), s.totalAttestors)

	// Start active Cosmos attestors (read Cosmos state for Cosmos→Eth direction)
	s.T().Log("Starting active Cosmos attestors...")
	s.cosmosAttestorResult = attestor.SetupAttestors(ctx, s.T(), attestor.SetupParams{
		NumAttestors:         s.activeAttestors,
		KeystorePathTemplate: MultiAttestorCosmosKeystorePathTemplate,
		ChainType:            attestor.ChainTypeCosmos,
		AdapterURL:           simd.GetRPCAddress(),
		RouterAddress:        "", // Cosmos doesn't use router address
		DockerClient:         s.GetDockerClient(),
		NetworkID:            s.GetNetworkID(),
	})
	s.T().Logf("Started %d of %d Cosmos attestors", len(s.cosmosAttestorResult.Addresses), s.totalAttestors)

	// Note: Docker containers cleanup automatically via t.Cleanup() registered in SetupAttestors

	// Start proof API with multi-attestor config
	var proofApiProcess *os.Process
	s.Require().True(s.Run("Start Proof API with multi-attestor config", func() {
		config := proofapi.NewConfigBuilder().
			// Eth → Cosmos direction (uses Eth attestors)
			EthToCosmosAttested(proofapi.EthToCosmosAttestedParams{
				EthChainID:        eth.ChainID.String(),
				CosmosChainID:     simd.Config().ChainID,
				TmRPC:             simd.GetHostRPCAddress(),
				ICS26Address:      s.contractAddresses.Ics26Router,
				EthRPC:            eth.RPC,
				SignerAddress:     s.SimdRelayerSubmitter.FormattedAddress(),
				AttestorEndpoints: s.ethAttestorResult.Endpoints,
				AttestorTimeout:   30000,
				QuorumThreshold:   s.quorumThreshold,
			}).
			// Cosmos → Eth direction (uses Cosmos attestors)
			CosmosToEthAttested(proofapi.CosmosToEthAttestedParams{
				CosmosChainID:     simd.Config().ChainID,
				EthChainID:        eth.ChainID.String(),
				TmRPC:             simd.GetHostRPCAddress(),
				ICS26Address:      s.contractAddresses.Ics26Router,
				EthRPC:            eth.RPC,
				AttestorEndpoints: s.cosmosAttestorResult.Endpoints,
				AttestorTimeout:   30000,
				QuorumThreshold:   s.quorumThreshold,
			}).
			Build()

		s.T().Logf("Proof API config with quorum threshold %d", s.quorumThreshold)

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
			_ = proofApiProcess.Kill()
		}
	})

	s.Require().True(s.Run("Create Proof API Client", func() {
		grpcAddr := proofapi.DefaultProofAPIGRPCAddress()
		s.T().Logf("Connecting to proof API at: %s", grpcAddr)

		var err error
		s.ProofApiClient, err = proofapi.GetGRPCClient(grpcAddr)
		s.Require().NoError(err)

		// Retry connecting to proof API
		var info *proofapitypes.InfoResponse
		for i := 0; i < 10; i++ {
			info, err = s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
				SrcChain: eth.ChainID.String(),
				DstChain: simd.Config().ChainID,
			})
			if err == nil {
				break
			}
			s.T().Logf("Attempt %d: proof API not ready yet: %v", i+1, err)
			time.Sleep(1 * time.Second)
		}
		s.Require().NoError(err, "Proof API Info call failed after retries")
		s.T().Logf("Proof API Info response: src=%s, dst=%s", info.SourceChain.ChainId, info.TargetChain.ChainId)
	}))

	// Deploy Cosmos attestor light client on Ethereum (for Cosmos→Eth direction)
	s.Require().True(s.Run("Deploy attestor light client on Ethereum for Cosmos", func() {
		latestCosmosHeader, err := simd.GetFullNode().Client.Header(ctx, nil)
		s.Require().NoError(err)

		// Register ALL Cosmos attestor addresses
		allAttestorAddresses := s.formatAttestorAddresses(s.allCosmosAttestorAddresses)

		var createClientTxBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: simd.Config().ChainID,
				DstChain: eth.ChainID.String(),
				Parameters: map[string]string{
					testvalues.ParameterKey_AttestorAddresses: allAttestorAddresses,
					testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(s.quorumThreshold),
					testvalues.ParameterKey_height:            strconv.FormatInt(latestCosmosHeader.Header.Height, 10),
					testvalues.ParameterKey_timestamp:         strconv.FormatInt(latestCosmosHeader.Header.Time.Unix(), 10),
					testvalues.ParameterKey_RoleManager:       ethcommon.HexToAddress(s.contractAddresses.Ics26Router).Hex(),
				},
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			createClientTxBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast create client tx on Ethereum", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, nil, createClientTxBz)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			lightClientAddress := receipt.ContractAddress
			s.T().Logf("Attestor light client for Cosmos deployed on Ethereum at: %s", lightClientAddress.Hex())
			s.T().Logf("Registered %d attestors (only %d active) with quorum %d",
				len(s.allCosmosAttestorAddresses), s.activeAttestors, s.quorumThreshold)

			counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
				ClientId:     s.getEthLcClientIDOnCosmos(),
				MerklePrefix: [][]byte{[]byte("")},
			}
			tx, err := s.ics26Contract.AddClient(s.GetTransactOpts(s.deployer, eth), MultiAttestorClientOnEth, counterpartyInfo, lightClientAddress)
			s.Require().NoError(err)

			_, err = eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
		}))
	}))

	// Deploy Ethereum attestor light client on Cosmos (for Eth→Cosmos direction)
	s.Require().True(s.Run("Deploy attestor light client on Cosmos for Ethereum", func() {
		// Store wasm binary if needed (not needed for native attestor)
		checksumHex := s.StoreLightClient(ctx, simd, s.SimdRelayerSubmitter)
		if !s.isNativeAttestor() {
			s.Require().NotEmpty(checksumHex)
		}

		// Get current Ethereum block
		currentBlockHeader, err := eth.RPCClient.HeaderByNumber(ctx, nil)
		s.Require().NoError(err)

		clientHeight := currentBlockHeader.Number.Int64()
		if clientHeight < 1 {
			clientHeight = 1
		}

		blockHeader, err := eth.RPCClient.HeaderByNumber(ctx, big.NewInt(clientHeight))
		s.Require().NoError(err)

		// Register ALL Eth attestor addresses
		allAttestorAddresses := s.formatAttestorAddresses(s.allEthAttestorAddresses)

		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			parameters := map[string]string{
				testvalues.ParameterKey_ChecksumHex:       checksumHex,
				testvalues.ParameterKey_AttestorAddresses: allAttestorAddresses,
				testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(s.quorumThreshold),
				testvalues.ParameterKey_height:            strconv.FormatInt(clientHeight, 10),
				testvalues.ParameterKey_timestamp:         strconv.FormatUint(blockHeader.Time, 10),
			}

			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain:   eth.ChainID.String(),
				DstChain:   simd.Config().ChainID,
				Parameters: parameters,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast create client tx on Cosmos", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, createClientTxBodyBz)
			clientId, err := cosmoshelper.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(s.getEthLcClientIDOnCosmos(), clientId)

			s.T().Logf("Attestor light client for Ethereum deployed on Cosmos: %s", clientId)
			s.T().Logf("Registered %d attestors (only %d active) with quorum %d",
				len(s.allEthAttestorAddresses), s.activeAttestors, s.quorumThreshold)
		}))
	}))

	// Register counterparty on Cosmos
	s.Require().True(s.Run("Register counterparty on Cosmos", func() {
		merklePathPrefix := [][]byte{[]byte("")}

		_, err := s.BroadcastMessages(ctx, simd, s.SimdRelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 s.getEthLcClientIDOnCosmos(),
			CounterpartyMerklePrefix: merklePathPrefix,
			CounterpartyClientId:     MultiAttestorClientOnEth,
			Signer:                   s.SimdRelayerSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))

	// Fund user with ERC20 tokens
	s.Require().True(s.Run("Fund user with ERC20 tokens", func() {
		userAddressEth := crypto.PubkeyToAddress(s.userKeyEth.PublicKey)
		tx, err := s.erc20Contract.Transfer(s.GetTransactOpts(eth.Faucet, eth), userAddressEth, testvalues.StartingERC20Balance)
		s.Require().NoError(err)
		_, err = eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
	}))
}

// formatAttestorAddresses formats a list of attestor addresses for the light client
func (s *MultiAttestorTestSuite) formatAttestorAddresses(addresses []string) string {
	formatted := make([]string, len(addresses))
	for i, addr := range addresses {
		formatted[i] = ethcommon.HexToAddress(addr).Hex()
	}
	return strings.Join(formatted, ",")
}

func (s *MultiAttestorTestSuite) Test_MultiAttestorTransferWithAggregation() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	eth, simd := s.EthChain(), s.CosmosChain()

	transferAmount := big.NewInt(testvalues.TransferAmount)
	userAddressEth := crypto.PubkeyToAddress(s.userKeyEth.PublicKey)
	ics20AddressEth := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	ics26AddressEth := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	erc20AddressEth := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	initialBalanceEth := new(big.Int).Set(testvalues.StartingERC20Balance)

	var ibcDenomOnCosmos transfertypes.Denom

	// Phase 1: Transfer from Ethereum to Cosmos using multi-attestor aggregation
	s.Require().True(s.Run("(Eth -> Cosmos): Approve ICS20 on Ethereum", func() {
		tx, err := s.erc20Contract.Approve(s.GetTransactOpts(s.userKeyEth, eth), ics20AddressEth, transferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	var sendTxHashEthToCosmos []byte
	s.Require().True(s.Run("(Eth -> Cosmos): Send transfer from Ethereum to Cosmos", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20AddressEth,
			Amount:           transferAmount,
			Receiver:         s.cosmosUser.FormattedAddress(),
			TimeoutTimestamp: timeout,
			SourceClient:     MultiAttestorClientOnEth,
			DestPort:         "transfer",
			Memo:             "",
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.userKeyEth, eth), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		sendTxHashEthToCosmos = tx.Hash().Bytes()
		s.T().Logf("Send tx hash Eth->Cosmos: 0x%s", hex.EncodeToString(sendTxHashEthToCosmos))
	}))

	s.Require().True(s.Run("(Eth -> Cosmos): Verify balances after send on Ethereum", func() {
		escrowAddress, err := s.ics20Contract.GetEscrow(nil, MultiAttestorClientOnEth)
		s.Require().NoError(err)

		escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
		s.Require().NoError(err)
		s.Require().Equal(0, transferAmount.Cmp(escrowBalance), "Escrow should hold transfer amount")

		userBalance, err := s.erc20Contract.BalanceOf(nil, userAddressEth)
		s.Require().NoError(err)
		expectedBalance := new(big.Int).Sub(initialBalanceEth, transferAmount)
		s.Require().Equal(0, expectedBalance.Cmp(userBalance), "User Eth balance should decrease")
	}))

	var recvSeqOnCosmos uint64
	var recvTxHashOnCosmos []byte
	s.Require().True(s.Run("(Eth -> Cosmos): Relay packet with multi-attestor aggregation", func() {
		var relayTx []byte
		s.Require().True(s.Run("Retrieve relay tx (aggregates signatures from multiple attestors)", func() {
			s.T().Logf("Requesting relay with %d attestors and quorum %d",
				len(s.ethAttestorResult.Endpoints), s.quorumThreshold)

			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{sendTxHashEthToCosmos},
				SrcClientId: MultiAttestorClientOnEth,
				DstClientId: s.getEthLcClientIDOnCosmos(),
			})
			s.Require().NoError(err, "Multi-attestor aggregation should succeed with %d of %d attestors",
				s.activeAttestors, s.quorumThreshold)
			s.Require().NotEmpty(resp.Tx)

			relayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx on Cosmos", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, relayTx)

			// Capture the recv tx hash for ack relay
			var err error
			recvTxHashOnCosmos, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.T().Logf("Recv tx hash on Cosmos: %s", resp.TxHash)

			recvSeqStr, err := cosmoshelper.GetEventValue(resp.Events, channeltypesv2.EventTypeRecvPacket, channeltypesv2.AttributeKeySequence)
			s.Require().NoError(err)
			recvSeqOnCosmos, err = strconv.ParseUint(recvSeqStr, 10, 64)
			s.Require().NoError(err)
			s.T().Logf("RecvPacket event received for packet seq %d", recvSeqOnCosmos)

			// Get the IBC denom on Cosmos
			destPort := "transfer"
			destClient := s.getEthLcClientIDOnCosmos()
			baseDenom := strings.ToLower(erc20AddressEth.Hex())
			ibcDenomOnCosmos = transfertypes.NewDenom(baseDenom, transfertypes.NewHop(destPort, destClient))
		}))
	}))

	s.Require().True(s.Run("(Eth -> Cosmos): Verify balances on Cosmos after receive", func() {
		denomOnCosmos := ibcDenomOnCosmos.IBCDenom()

		balance, err := simd.GetBalance(ctx, s.cosmosUser.FormattedAddress(), denomOnCosmos)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount.Int64(), balance.Int64(), "User on Cosmos should have received tokens")
		s.T().Logf("User Cosmos balance: %s %s", balance.String(), denomOnCosmos)
	}))

	s.Require().True(s.Run("(Eth -> Cosmos): Verify commitment exists before ack", func() {
		packetCommitmentPath := ibchostv2.PacketCommitmentKey(MultiAttestorClientOnEth, 1)
		var ethPath [32]byte
		copy(ethPath[:], crypto.Keccak256(packetCommitmentPath))

		resp, err := s.ics26Contract.GetCommitment(nil, ethPath)
		s.Require().NoError(err)
		s.Require().NotZero(resp, "Packet commitment should exist before ack")
	}))

	s.Require().True(s.Run("(Eth -> Cosmos): Relay acknowledgement with multi-attestor aggregation", func() {
		var ackRelayTx []byte
		s.Require().True(s.Run("Retrieve ack relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{recvTxHashOnCosmos},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: MultiAttestorClientOnEth,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			ackRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast ack relay tx on Ethereum", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &ics26AddressEth, ackRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

			ackEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseAckPacket)
			s.Require().NoError(err)
			s.T().Logf("AckPacket event received for packet seq %d", ackEvent.Packet.Sequence)
		}))
	}))

	s.Require().True(s.Run("(Eth -> Cosmos): Verify commitment removed after ack", func() {
		packetCommitmentPath := ibchostv2.PacketCommitmentKey(MultiAttestorClientOnEth, 1)
		var ethPath [32]byte
		copy(ethPath[:], crypto.Keccak256(packetCommitmentPath))

		resp, err := s.ics26Contract.GetCommitment(nil, ethPath)
		s.Require().NoError(err)
		s.Require().Zero(resp, "Commitment should be removed after ack")
	}))

	// Phase 2: Transfer back from Cosmos to Ethereum
	cosmosTransferAmount := sdkmath.NewIntFromBigInt(transferAmount)
	transferCoin := sdk.NewCoin(ibcDenomOnCosmos.Path(), cosmosTransferAmount)

	var cosmosSendTxHash []byte
	s.Require().True(s.Run("(Cosmos -> Eth): Send transfer from Cosmos to Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		// Prepare ICS20 v1 transfer payload
		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    transferCoin.Denom,
			Amount:   transferCoin.Amount.String(),
			Sender:   s.cosmosUser.FormattedAddress(),
			Receiver: strings.ToLower(userAddressEth.Hex()),
			Memo:     "",
		}

		msgSendPacket := channeltypesv2.MsgSendPacket{
			SourceClient:     s.getEthLcClientIDOnCosmos(),
			TimeoutTimestamp: timeout,
			Signer:           s.cosmosUser.FormattedAddress(),
			Payloads: []channeltypesv2.Payload{
				{
					SourcePort:      transfertypes.PortID,
					DestinationPort: transfertypes.PortID,
					Version:         transfertypes.V1,
					Encoding:        transfertypes.EncodingABI,
					Value:           must(transfertypes.EncodeABIFungibleTokenPacketData(&transferPayload)),
				},
			},
		}

		resp, err := s.BroadcastMessages(ctx, simd, s.cosmosUser, 200_000, &msgSendPacket)
		s.Require().NoError(err)

		cosmosSendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)

		sendSeqStr, err := cosmoshelper.GetEventValue(resp.Events, channeltypesv2.EventTypeSendPacket, channeltypesv2.AttributeKeySequence)
		s.Require().NoError(err)
		s.T().Logf("SendPacket event with seq %s, tx hash: %s", sendSeqStr, resp.TxHash)
	}))

	s.Require().True(s.Run("(Cosmos -> Eth): Verify balances after send on Cosmos", func() {
		denomOnCosmos := ibcDenomOnCosmos.IBCDenom()

		balance, err := simd.GetBalance(ctx, s.cosmosUser.FormattedAddress(), denomOnCosmos)
		s.Require().NoError(err)
		s.Require().Zero(balance.Int64(), "User Cosmos balance should be zero after sending back")
	}))

	var recvTxHashOnEth []byte
	s.Require().True(s.Run("(Cosmos -> Eth): Relay packet with multi-attestor aggregation", func() {
		var relayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{cosmosSendTxHash},
				SrcClientId: s.getEthLcClientIDOnCosmos(),
				DstClientId: MultiAttestorClientOnEth,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(ics26AddressEth.String(), resp.Address)

			relayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx on Ethereum", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &ics26AddressEth, relayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

			// Capture recv tx hash for ack relay
			recvTxHashOnEth = receipt.TxHash.Bytes()
			s.T().Logf("Recv tx hash on Eth: %s", receipt.TxHash.Hex())

			ackEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseWriteAcknowledgement)
			s.Require().NoError(err)
			s.T().Logf("WriteAcknowledgement event received for packet seq %d", ackEvent.Packet.Sequence)
		}))
	}))

	s.Require().True(s.Run("(Cosmos -> Eth): Verify balances on Ethereum after receive", func() {
		userBalanceEth, err := s.erc20Contract.BalanceOf(nil, userAddressEth)
		s.Require().NoError(err)
		s.Require().Equal(0, initialBalanceEth.Cmp(userBalanceEth), "User Eth should have original balance restored")
		s.T().Logf("User Eth balance restored: %s", userBalanceEth.String())

		escrowAddress, err := s.ics20Contract.GetEscrow(nil, MultiAttestorClientOnEth)
		s.Require().NoError(err)

		escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
		s.Require().NoError(err)
		s.Require().Zero(escrowBalance.Int64(), "Escrow should be empty after unwind")
	}))

	s.Require().True(s.Run("(Cosmos -> Eth): Verify commitment exists before ack", func() {
		resp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
			ClientId: s.getEthLcClientIDOnCosmos(),
			Sequence: 1,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Commitment, "Packet commitment should exist before ack")
	}))

	s.Require().True(s.Run("(Cosmos -> Eth): Relay final acknowledgement", func() {
		var ackRelayTx []byte
		s.Require().True(s.Run("Retrieve ack relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SrcClientId: MultiAttestorClientOnEth,
				DstClientId: s.getEthLcClientIDOnCosmos(),
				SourceTxIds: [][]byte{recvTxHashOnEth},
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			ackRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast ack relay tx on Cosmos", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, ackRelayTx)

			ackSeqStr, err := cosmoshelper.GetEventValue(resp.Events, channeltypesv2.EventTypeAcknowledgePacket, channeltypesv2.AttributeKeySequence)
			s.Require().NoError(err)
			s.T().Logf("Final AckPacket event received for packet seq %s", ackSeqStr)
		}))
	}))

	s.Require().True(s.Run("(Cosmos -> Eth): Verify commitment removed after ack", func() {
		_, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
			ClientId: s.getEthLcClientIDOnCosmos(),
			Sequence: 1,
		})
		s.Require().ErrorContains(err, "packet commitment hash not found", "Commitment should be removed after ack")
	}))

	s.T().Logf("Multi-attestor transfer test completed successfully (EVM ↔ Cosmos) with %d-of-%d multisig",
		s.quorumThreshold, s.totalAttestors)
}

func (s *MultiAttestorTestSuite) Test_UpdateClientCosmosToEth() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	eth := s.EthChain()

	s.Require().True(s.Run("Update client on Ethereum", func() {
		resp, err := s.ProofApiClient.UpdateClient(context.Background(), &proofapitypes.UpdateClientRequest{
			SrcChain:    s.CosmosChain().Config().ChainID,
			DstChain:    eth.ChainID.String(),
			DstClientId: MultiAttestorClientOnEth,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
		receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &ics26Address, resp.Tx)
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))
}

func must[T any](v T, err error) T {
	if err != nil {
		panic(err)
	}
	return v
}

func mustGetEnvInt(t *testing.T, key string) int {
	t.Helper()
	value := os.Getenv(key)
	if value == "" {
		t.Fatalf("required env var %s is not set", key)
	}
	intVal, err := strconv.Atoi(value)
	if err != nil {
		t.Fatalf("env var %s is not a valid integer: %v", key, err)
	}
	return intVal
}
