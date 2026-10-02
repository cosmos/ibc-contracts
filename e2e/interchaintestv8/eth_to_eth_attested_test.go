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

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/attestor"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ibcflow"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

const (
	chainAIndex = 0
	chainBIndex = 1

	// ClientA is deployed on Chain A, tracks Chain B's state
	ClientA = "eth-chain-b"
	// ClientB is deployed on Chain B, tracks Chain A's state
	ClientB = "eth-chain-a"

	// Keystore path templates for eth-to-eth attestors
	ethToEthKeystoreAPathTemplate = "/tmp/eth_to_eth_keystore_a_%d"
	ethToEthKeystoreBPathTemplate = "/tmp/eth_to_eth_keystore_b_%d"
)

// EthToEthAttestedTestSuite tests IBC transfers between two Ethereum chains using attestation
type EthToEthAttestedTestSuite struct {
	e2esuite.TestSuite

	// Chain A (source) and Chain B (destination)
	chainA    *ibcflow.EVMEndpoint
	chainB    *ibcflow.EVMEndpoint
	deployerA *ecdsa.PrivateKey
	deployerB *ecdsa.PrivateKey

	ProofApiClient proofapitypes.ProofApiServiceClient
}

func TestWithEthToEthAttestedTestSuite(t *testing.T) {
	suite.Run(t, new(EthToEthAttestedTestSuite))
}

// EthChainA returns the first Ethereum chain
func (s *EthToEthAttestedTestSuite) EthChainA() *ethereum.Ethereum {
	return s.Eth.Chains[chainAIndex]
}

// EthChainB returns the second Ethereum chain
func (s *EthToEthAttestedTestSuite) EthChainB() *ethereum.Ethereum {
	return s.Eth.Chains[chainBIndex]
}

func (s *EthToEthAttestedTestSuite) SetupSuite(ctx context.Context) {
	s.T().Log("Setting up EthToEthAttestedTestSuite")

	if os.Getenv(testvalues.EnvKeyRustLog) == "" {
		os.Setenv(testvalues.EnvKeyRustLog, testvalues.EnvValueRustLog_Info)
	}

	// Configure for two Anvil chains
	os.Setenv(testvalues.EnvKeyEthTestnetType, testvalues.EthTestnetTypeAnvil)
	os.Setenv(testvalues.EnvKeyEthAnvilCount, "2")

	// Call the base SetupSuite which will create the chains
	s.TestSuite.SetupSuite(ctx)

	err := os.Chdir("../..")
	s.Require().NoError(err)

	s.T().Logf("Chain A RPC: %s, Chain ID: %s", s.EthChainA().RPC, s.EthChainA().ChainID.String())
	s.T().Logf("Chain B RPC: %s, Chain ID: %s", s.EthChainB().RPC, s.EthChainB().ChainID.String())

	// Create and fund users on both chains
	var userKeyA, userKeyB, relayerSubmitterA, relayerSubmitterB *ecdsa.PrivateKey
	s.Require().True(s.Run("Create and fund users", func() {
		var err error
		userKeyA, err = s.EthChainA().CreateAndFundUser()
		s.Require().NoError(err)
		s.deployerA, err = s.EthChainA().CreateAndFundUser()
		s.Require().NoError(err)
		relayerSubmitterA, err = s.EthChainA().CreateAndFundUser()
		s.Require().NoError(err)

		userKeyB, err = s.EthChainB().CreateAndFundUser()
		s.Require().NoError(err)
		s.deployerB, err = s.EthChainB().CreateAndFundUser()
		s.Require().NoError(err)
		relayerSubmitterB, err = s.EthChainB().CreateAndFundUser()
		s.Require().NoError(err)

		// For operator
		operatorKeyA, err := s.EthChainA().CreateAndFundUser()
		s.Require().NoError(err)
		os.Setenv(testvalues.EnvKeyOperatorPrivateKey, hex.EncodeToString(crypto.FromECDSA(operatorKeyA)))
	}))

	// Deploy contracts on Chain A
	s.Require().True(s.Run("Deploy contracts on Chain A", func() {
		os.Setenv(testvalues.EnvKeyEthRPC, s.EthChainA().RPC)
		stdout, err := s.EthChainA().ForgeScript(s.deployerA, testvalues.E2EDeployScriptPath)
		s.Require().NoError(err)

		contracts, err := ethereum.GetEthContractsFromDeployOutput(string(stdout))
		s.Require().NoError(err)

		s.chainA = ibcflow.NewEVMEndpoint(s.T(), s.EthChainA(), contracts, ClientA, userKeyA, relayerSubmitterA)
	}))

	// Deploy contracts on Chain B
	s.Require().True(s.Run("Deploy contracts on Chain B", func() {
		os.Setenv(testvalues.EnvKeyEthRPC, s.EthChainB().RPC)
		stdout, err := s.EthChainB().ForgeScript(s.deployerB, testvalues.E2EDeployScriptPath)
		s.Require().NoError(err)

		contracts, err := ethereum.GetEthContractsFromDeployOutput(string(stdout))
		s.Require().NoError(err)

		s.chainB = ibcflow.NewEVMEndpoint(s.T(), s.EthChainB(), contracts, ClientB, userKeyB, relayerSubmitterB)
	}))

	// Start attestor for Chain A (reads Chain A state)
	var attestorAEndpoint string
	var attestorAAddress string
	s.T().Log("Starting attestor for Chain A...")
	attestorResultA := attestor.SetupAttestors(ctx, s.T(), attestor.SetupParams{
		NumAttestors:         1,
		KeystorePathTemplate: ethToEthKeystoreAPathTemplate,
		ChainType:            attestor.ChainTypeEvm,
		AdapterURL:           s.EthChainA().DockerRPC, // Use Docker internal RPC for container-to-container communication
		RouterAddress:        s.chainA.Contracts.Ics26Router,
		DockerClient:         s.GetDockerClient(),
		NetworkID:            s.GetNetworkID(),
	})
	s.Require().Len(attestorResultA.Containers, 1)
	s.Require().Len(attestorResultA.Endpoints, 1)
	s.Require().Len(attestorResultA.Addresses, 1)
	attestorAEndpoint = attestorResultA.Endpoints[0]
	attestorAAddress = attestorResultA.Addresses[0]

	// Start attestor for Chain B (reads Chain B state)
	var attestorBEndpoint string
	var attestorBAddress string
	s.T().Log("Starting attestor for Chain B...")
	attestorResultB := attestor.SetupAttestors(ctx, s.T(), attestor.SetupParams{
		NumAttestors:         1,
		KeystorePathTemplate: ethToEthKeystoreBPathTemplate,
		ChainType:            attestor.ChainTypeEvm,
		AdapterURL:           s.EthChainB().DockerRPC, // Use Docker internal RPC for container-to-container communication
		RouterAddress:        s.chainB.Contracts.Ics26Router,
		DockerClient:         s.GetDockerClient(),
		NetworkID:            s.GetNetworkID(),
	})
	s.Require().Len(attestorResultB.Containers, 1)
	s.Require().Len(attestorResultB.Endpoints, 1)
	s.Require().Len(attestorResultB.Addresses, 1)
	attestorBEndpoint = attestorResultB.Endpoints[0]
	attestorBAddress = attestorResultB.Addresses[0]

	// Final verification that attestors are accessible before starting proof API
	s.Require().True(s.Run("Verify attestors before proof API start", func() {
		s.T().Logf("[pre-proof-api] Verifying attestor A at %s", attestorAEndpoint)
		err := attestor.CheckAttestorHealth(ctx, attestorAEndpoint)
		s.Require().NoError(err, "Attestor A health check failed before proof API start")

		s.T().Logf("[pre-proof-api] Verifying attestor B at %s", attestorBEndpoint)
		err = attestor.CheckAttestorHealth(ctx, attestorBEndpoint)
		s.Require().NoError(err, "Attestor B health check failed before proof API start")

		s.T().Log("[pre-proof-api] Both attestors verified healthy before starting proof API")
	}))

	// Start proof API with eth-to-eth-attested modules
	var proofApiProcess *os.Process
	s.Require().True(s.Run("Start Proof API", func() {
		// Log the exact endpoints being used
		s.T().Logf("[proofapi-config] Attestor A endpoint: %s", attestorAEndpoint)
		s.T().Logf("[proofapi-config] Attestor B endpoint: %s", attestorBEndpoint)

		// Create custom aggregator configs for each chain
		config := proofapi.NewConfigBuilder().
			EthToEthAttested(proofapi.EthToEthAttestedParams{
				SrcChainID:        s.EthChainA().ChainID.String(),
				DstChainID:        s.EthChainB().ChainID.String(),
				SrcRPC:            s.EthChainA().RPC,
				DstRPC:            s.EthChainB().RPC,
				SrcICS26:          s.chainA.Contracts.Ics26Router,
				DstICS26:          s.chainB.Contracts.Ics26Router,
				AttestorEndpoints: []string{attestorAEndpoint},
				AttestorTimeout:   30000,
			}).
			EthToEthAttested(proofapi.EthToEthAttestedParams{
				SrcChainID:        s.EthChainB().ChainID.String(),
				DstChainID:        s.EthChainA().ChainID.String(),
				SrcRPC:            s.EthChainB().RPC,
				DstRPC:            s.EthChainA().RPC,
				SrcICS26:          s.chainB.Contracts.Ics26Router,
				DstICS26:          s.chainA.Contracts.Ics26Router,
				AttestorEndpoints: []string{attestorBEndpoint},
				AttestorTimeout:   30000,
			}).
			Build()

		s.T().Logf("Proof API config: %+v", config)

		err := config.GenerateConfigFile(testvalues.ProofAPIConfigFilePath)
		s.Require().NoError(err)

		// Log the generated config file for debugging
		configContent, readErr := os.ReadFile(testvalues.ProofAPIConfigFilePath)
		if readErr == nil {
			s.T().Logf("[proofapi-config] Generated config file:\n%s", string(configContent))
		}

		proofApiProcess, err = proofapi.StartProofAPI(testvalues.ProofAPIConfigFilePath)
		s.Require().NoError(err)

		s.T().Logf("[proof-api] Process started with PID: %d", proofApiProcess.Pid)

		s.T().Cleanup(func() {
			os.Remove(testvalues.ProofAPIConfigFilePath)
		})
	}))

	// Move proof API cleanup outside the subtest so it doesn't run immediately after subtest completion
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

		// Retry connecting to proof API with backoff
		var info *proofapitypes.InfoResponse
		for i := range 10 {
			info, err = s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
				SrcChain: s.EthChainA().ChainID.String(),
				DstChain: s.EthChainB().ChainID.String(),
			})
			if err == nil {
				break
			}
			s.T().Logf("Attempt %d: proof API not ready yet: %v", i+1, err)
			time.Sleep(1 * time.Second)
		}
		s.Require().NoError(err, "Proof API Info call failed after retries - proof API may have crashed")
		s.T().Logf("Proof API Info response: src=%s, dst=%s", info.SourceChain.ChainId, info.TargetChain.ChainId)
	}))

	// Deploy attestor light client on Chain A (for Chain B's state)
	s.Require().True(s.Run("Deploy attestor light client on Chain A for Chain B", func() {
		// Get current block from Chain B to initialize the light client
		chainBHeader, err := s.EthChainB().RPCClient.HeaderByNumber(ctx, nil)
		s.Require().NoError(err)

		var createClientTxBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			attestorAddrForClient := ethcommon.HexToAddress(attestorBAddress).Hex()

			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: s.EthChainB().ChainID.String(),
				DstChain: s.EthChainA().ChainID.String(),
				Parameters: map[string]string{
					testvalues.ParameterKey_AttestorAddresses: attestorAddrForClient,
					testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(testvalues.DefaultMinRequiredSigs),
					testvalues.ParameterKey_height:            strconv.FormatInt(chainBHeader.Number.Int64(), 10),
					testvalues.ParameterKey_timestamp:         strconv.FormatUint(chainBHeader.Time, 10),
				},
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			createClientTxBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast create client tx on Chain A", func() {
			receipt, err := s.EthChainA().BroadcastTx(ctx, s.chainA.RelayerSubmitterKey, 15_000_000, nil, createClientTxBz)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			lightClientAddress := receipt.ContractAddress
			s.T().Logf("Light client for Chain B deployed on Chain A at: %s", lightClientAddress.Hex())

			// Add client on Chain A that tracks Chain B's state
			// The counterparty client ID is the client on Chain B that tracks Chain A
			counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
				ClientId:     ClientB,
				MerklePrefix: [][]byte{[]byte("")}, // EVM chains don't use store key prefix
			}
			tx, err := s.chainA.ICS26.AddClient(s.GetTransactOpts(s.deployerA, s.EthChainA()), ClientA, counterpartyInfo, lightClientAddress)
			s.Require().NoError(err)

			_, err = s.EthChainA().GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
		}))
	}))

	// Deploy attestor light client on Chain B (for Chain A's state)
	s.Require().True(s.Run("Deploy attestor light client on Chain B for Chain A", func() {
		// Get current block from Chain A to initialize the light client
		chainAHeader, err := s.EthChainA().RPCClient.HeaderByNumber(ctx, nil)
		s.Require().NoError(err)

		var createClientTxBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: s.EthChainA().ChainID.String(),
				DstChain: s.EthChainB().ChainID.String(),
				Parameters: map[string]string{
					testvalues.ParameterKey_AttestorAddresses: ethcommon.HexToAddress(attestorAAddress).Hex(),
					testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(testvalues.DefaultMinRequiredSigs),
					testvalues.ParameterKey_height:            strconv.FormatInt(chainAHeader.Number.Int64(), 10),
					testvalues.ParameterKey_timestamp:         strconv.FormatUint(chainAHeader.Time, 10),
				},
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			createClientTxBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast create client tx on Chain B", func() {
			receipt, err := s.EthChainB().BroadcastTx(ctx, s.chainB.RelayerSubmitterKey, 15_000_000, nil, createClientTxBz)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			lightClientAddress := receipt.ContractAddress
			s.T().Logf("Light client for Chain A deployed on Chain B at: %s", lightClientAddress.Hex())

			// Add client on Chain B that tracks Chain A's state
			// The counterparty client ID is the client on Chain A that tracks Chain B
			counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
				ClientId:     ClientA,
				MerklePrefix: [][]byte{[]byte("")}, // EVM chains don't use store key prefix
			}
			tx, err := s.chainB.ICS26.AddClient(s.GetTransactOpts(s.deployerB, s.EthChainB()), ClientB, counterpartyInfo, lightClientAddress)
			s.Require().NoError(err)

			_, err = s.EthChainB().GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
		}))
	}))

	// Fund users with ERC20 tokens
	s.Require().True(s.Run("Fund users with ERC20 tokens", func() {
		tx, err := s.chainA.ERC20.Transfer(s.GetTransactOpts(s.EthChainA().Faucet, s.EthChainA()), s.chainA.UserAddress(), testvalues.StartingERC20Balance)
		s.Require().NoError(err)
		_, err = s.EthChainA().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)

		tx, err = s.chainB.ERC20.Transfer(s.GetTransactOpts(s.EthChainB().Faucet, s.EthChainB()), s.chainB.UserAddress(), testvalues.StartingERC20Balance)
		s.Require().NoError(err)
		_, err = s.EthChainB().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
	}))
}

func (s *EthToEthAttestedTestSuite) Test_Deploy() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	s.Require().True(s.Run("Verify ICS26 on Chain A", func() {
		transferAddress, err := s.chainA.ICS26.GetIBCApp(nil, "transfer")
		s.Require().NoError(err)
		s.Require().Equal(strings.ToLower(s.chainA.Contracts.Ics20Transfer), strings.ToLower(transferAddress.Hex()))
	}))

	s.Require().True(s.Run("Verify ICS26 on Chain B", func() {
		transferAddress, err := s.chainB.ICS26.GetIBCApp(nil, "transfer")
		s.Require().NoError(err)
		s.Require().Equal(strings.ToLower(s.chainB.Contracts.Ics20Transfer), strings.ToLower(transferAddress.Hex()))
	}))

	s.Require().True(s.Run("Verify Proof API Info A->B", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: s.EthChainA().ChainID.String(),
			DstChain: s.EthChainB().ChainID.String(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(s.EthChainA().ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(s.EthChainB().ChainID.String(), info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Proof API Info B->A", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: s.EthChainB().ChainID.String(),
			DstChain: s.EthChainA().ChainID.String(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(s.EthChainB().ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(s.EthChainA().ChainID.String(), info.TargetChain.ChainId)
	}))
}

func (s *EthToEthAttestedTestSuite) Test_TransferERC20FromChainAToChainBAndBack() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	ibcflow.Roundtrip(ctx, s.T(), s.ProofApiClient, s.chainA, s.chainB, big.NewInt(testvalues.TransferAmount), 1)
}

func (s *EthToEthAttestedTestSuite) Test_TimeoutPacketFromChainA() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	ibcflow.TimeoutFromA(ctx, s.T(), s.ProofApiClient, s.chainA, s.chainB, big.NewInt(testvalues.TransferAmount))
}

func (s *EthToEthAttestedTestSuite) Test_UpdateClient() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	s.Require().True(s.Run("Update client on Chain B", func() {
		resp, err := s.ProofApiClient.UpdateClient(context.Background(), &proofapitypes.UpdateClientRequest{
			SrcChain:    s.EthChainA().ChainID.String(),
			DstChain:    s.EthChainB().ChainID.String(),
			DstClientId: ClientB,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		// Broadcast the update client tx
		ics26AddressB := ethcommon.HexToAddress(s.chainB.Contracts.Ics26Router)
		receipt, err := s.EthChainB().BroadcastTx(ctx, s.chainB.RelayerSubmitterKey, 15_000_000, &ics26AddressB, resp.Tx)
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))
}

// Test_TimeoutPacket_AsymmetricHeight tests timeout relay with asymmetric block heights
// where Chain A (destination) has a higher height than Chain B (source).
func (s *EthToEthAttestedTestSuite) Test_TimeoutPacket_AsymmetricHeight() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	// Use a short timeout so we can quickly get past it
	const timeoutOffsetSeconds = 10
	// Number of extra blocks to mine on Chain A to create height asymmetry
	const extraBlocksOnChainA = 50

	packetTimeout := uint64(time.Now().Unix()) + timeoutOffsetSeconds

	var sendTxHash []byte
	s.Require().True(s.Run("Send transfer with short timeout", func() {
		erc20AddressA := ethcommon.HexToAddress(s.chainA.Contracts.Erc20)
		receipt, _ := ibcflow.SendTransfer(ctx, s.T(), s.chainA, erc20AddressA, big.NewInt(testvalues.TransferAmount), s.chainB.UserAddress().Hex(), packetTimeout)
		sendTxHash = receipt.TxHash.Bytes()
	}))

	s.Require().True(s.Run("Wait for timeout to pass on both chains", func() {
		s.Require().NoError(e2esuite.WaitForBlockTime(ctx, s.T(), s.EthChainA(), packetTimeout))
		s.Require().NoError(e2esuite.WaitForBlockTime(ctx, s.T(), s.EthChainB(), packetTimeout))
	}))

	// First, demonstrate that RelayByTx works when chains are in sync
	s.Require().True(s.Run("Attempt timeout relay with chains in sync - should succeed", func() {
		// Both chains have similar heights at this point
		chainAHeight, err := s.EthChainA().RPCClient.BlockNumber(ctx)
		s.Require().NoError(err)

		chainBHeight, err := s.EthChainB().RPCClient.BlockNumber(ctx)
		s.Require().NoError(err)

		s.T().Logf("Chains in sync - Chain A height: %d, Chain B height: %d", chainAHeight, chainBHeight)

		// Wait for attestor to catch up
		time.Sleep(3 * time.Second)

		ctxWithTimeout, cancel := context.WithTimeout(ctx, 30*time.Second)
		defer cancel()

		resp, err := s.ProofApiClient.RelayByTx(ctxWithTimeout, &proofapitypes.RelayByTxRequest{
			SrcChain:     s.EthChainB().ChainID.String(),
			DstChain:     s.EthChainA().ChainID.String(),
			TimeoutTxIds: [][]byte{sendTxHash},
			SrcClientId:  ClientB,
			DstClientId:  ClientA,
		})
		s.Require().NoError(err, "RelayByTx should succeed when chains are in sync")
		s.Require().NotEmpty(resp.Tx, "Timeout relay tx should not be empty")
	}))

	// Fast-forward Chain A to create height asymmetry
	var chainAHeight, chainBHeight uint64
	s.Require().True(s.Run("Pause Chain B and mine many blocks on Chain A to create asymmetry", func() {
		// Get current heights
		var err error
		chainBHeight, err = s.EthChainB().RPCClient.BlockNumber(ctx)
		s.Require().NoError(err)

		chainAHeight, err = s.EthChainA().RPCClient.BlockNumber(ctx)
		s.Require().NoError(err)

		s.T().Logf("Before asymmetry - Chain A: %d, Chain B: %d", chainAHeight, chainBHeight)

		// Pause Chain B's block production
		err = s.EthChainB().SetIntervalMining(ctx, 0)
		s.Require().NoError(err)
		s.T().Log("Paused Chain B block production")

		// Mine many blocks on Chain A to create height asymmetry
		for i := 0; i < extraBlocksOnChainA; i++ {
			err = s.EthChainA().MineBlock(ctx)
			s.Require().NoError(err)
		}

		// Get new heights
		newChainAHeight, err := s.EthChainA().RPCClient.BlockNumber(ctx)
		s.Require().NoError(err)

		newChainBHeight, err := s.EthChainB().RPCClient.BlockNumber(ctx)
		s.Require().NoError(err)

		s.T().Logf("After mining - Chain A: %d (+%d blocks), Chain B: %d (paused)",
			newChainAHeight, newChainAHeight-chainAHeight, newChainBHeight)

		s.Require().Greater(newChainAHeight, newChainBHeight+uint64(extraBlocksOnChainA-10),
			"Chain A should have significantly more blocks than Chain B")

		chainAHeight = newChainAHeight
		chainBHeight = newChainBHeight
	}))

	s.Require().True(s.Run("Timeout relay with Chain A height >> Chain B height", func() {
		s.T().Logf("Chain A height: %d, Chain B height: %d (difference: %d)",
			chainAHeight, chainBHeight, chainAHeight-chainBHeight)

		ctxWithTimeout, cancel := context.WithTimeout(ctx, 30*time.Second)
		defer cancel()

		resp, err := s.ProofApiClient.RelayByTx(ctxWithTimeout, &proofapitypes.RelayByTxRequest{
			SrcChain:     s.EthChainB().ChainID.String(),
			DstChain:     s.EthChainA().ChainID.String(),
			TimeoutTxIds: [][]byte{sendTxHash},
			SrcClientId:  ClientB,
			DstClientId:  ClientA,
		})

		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)
	}))
}
