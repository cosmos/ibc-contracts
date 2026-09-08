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

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ibcerc20"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics20transfer"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/attestor"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/erc20"
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

	// Chain A (source) - contracts and keys
	contractAddressesA ethereum.DeployedContracts
	ics26ContractA     *ics26router.Contract
	ics20ContractA     *ics20transfer.Contract
	erc20ContractA     *erc20.Contract
	deployerA          *ecdsa.PrivateKey
	userKeyA           *ecdsa.PrivateKey

	// Chain B (destination) - contracts and keys
	contractAddressesB ethereum.DeployedContracts
	ics26ContractB     *ics26router.Contract
	ics20ContractB     *ics20transfer.Contract
	erc20ContractB     *erc20.Contract
	deployerB          *ecdsa.PrivateKey
	userKeyB           *ecdsa.PrivateKey

	// Relayer submitters
	EthRelayerSubmitterA *ecdsa.PrivateKey
	EthRelayerSubmitterB *ecdsa.PrivateKey

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
	s.Require().True(s.Run("Create and fund users", func() {
		var err error
		s.userKeyA, err = s.EthChainA().CreateAndFundUser()
		s.Require().NoError(err)
		s.deployerA, err = s.EthChainA().CreateAndFundUser()
		s.Require().NoError(err)
		s.EthRelayerSubmitterA, err = s.EthChainA().CreateAndFundUser()
		s.Require().NoError(err)

		s.userKeyB, err = s.EthChainB().CreateAndFundUser()
		s.Require().NoError(err)
		s.deployerB, err = s.EthChainB().CreateAndFundUser()
		s.Require().NoError(err)
		s.EthRelayerSubmitterB, err = s.EthChainB().CreateAndFundUser()
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

		s.contractAddressesA, err = ethereum.GetEthContractsFromDeployOutput(string(stdout))
		s.Require().NoError(err)

		s.ics26ContractA, err = ics26router.NewContract(ethcommon.HexToAddress(s.contractAddressesA.Ics26Router), s.EthChainA().RPCClient)
		s.Require().NoError(err)
		s.ics20ContractA, err = ics20transfer.NewContract(ethcommon.HexToAddress(s.contractAddressesA.Ics20Transfer), s.EthChainA().RPCClient)
		s.Require().NoError(err)
		s.erc20ContractA, err = erc20.NewContract(ethcommon.HexToAddress(s.contractAddressesA.Erc20), s.EthChainA().RPCClient)
		s.Require().NoError(err)
	}))

	// Deploy contracts on Chain B
	s.Require().True(s.Run("Deploy contracts on Chain B", func() {
		os.Setenv(testvalues.EnvKeyEthRPC, s.EthChainB().RPC)
		stdout, err := s.EthChainB().ForgeScript(s.deployerB, testvalues.E2EDeployScriptPath)
		s.Require().NoError(err)

		s.contractAddressesB, err = ethereum.GetEthContractsFromDeployOutput(string(stdout))
		s.Require().NoError(err)

		s.ics26ContractB, err = ics26router.NewContract(ethcommon.HexToAddress(s.contractAddressesB.Ics26Router), s.EthChainB().RPCClient)
		s.Require().NoError(err)
		s.ics20ContractB, err = ics20transfer.NewContract(ethcommon.HexToAddress(s.contractAddressesB.Ics20Transfer), s.EthChainB().RPCClient)
		s.Require().NoError(err)
		s.erc20ContractB, err = erc20.NewContract(ethcommon.HexToAddress(s.contractAddressesB.Erc20), s.EthChainB().RPCClient)
		s.Require().NoError(err)
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
		RouterAddress:        s.contractAddressesA.Ics26Router,
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
		RouterAddress:        s.contractAddressesB.Ics26Router,
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
				SrcICS26:          s.contractAddressesA.Ics26Router,
				DstICS26:          s.contractAddressesB.Ics26Router,
				AttestorEndpoints: []string{attestorAEndpoint},
				AttestorTimeout:   30000,
			}).
			EthToEthAttested(proofapi.EthToEthAttestedParams{
				SrcChainID:        s.EthChainB().ChainID.String(),
				DstChainID:        s.EthChainA().ChainID.String(),
				SrcRPC:            s.EthChainB().RPC,
				DstRPC:            s.EthChainA().RPC,
				SrcICS26:          s.contractAddressesB.Ics26Router,
				DstICS26:          s.contractAddressesA.Ics26Router,
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
			receipt, err := s.EthChainA().BroadcastTx(ctx, s.EthRelayerSubmitterA, 15_000_000, nil, createClientTxBz)
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
			tx, err := s.ics26ContractA.AddClient(s.GetTransactOpts(s.deployerA, s.EthChainA()), ClientA, counterpartyInfo, lightClientAddress)
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
			receipt, err := s.EthChainB().BroadcastTx(ctx, s.EthRelayerSubmitterB, 15_000_000, nil, createClientTxBz)
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
			tx, err := s.ics26ContractB.AddClient(s.GetTransactOpts(s.deployerB, s.EthChainB()), ClientB, counterpartyInfo, lightClientAddress)
			s.Require().NoError(err)

			_, err = s.EthChainB().GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
		}))
	}))

	// Fund users with ERC20 tokens
	s.Require().True(s.Run("Fund users with ERC20 tokens", func() {
		userAddressA := crypto.PubkeyToAddress(s.userKeyA.PublicKey)
		tx, err := s.erc20ContractA.Transfer(s.GetTransactOpts(s.EthChainA().Faucet, s.EthChainA()), userAddressA, testvalues.StartingERC20Balance)
		s.Require().NoError(err)
		_, err = s.EthChainA().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)

		userAddressB := crypto.PubkeyToAddress(s.userKeyB.PublicKey)
		tx, err = s.erc20ContractB.Transfer(s.GetTransactOpts(s.EthChainB().Faucet, s.EthChainB()), userAddressB, testvalues.StartingERC20Balance)
		s.Require().NoError(err)
		_, err = s.EthChainB().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
	}))
}

func (s *EthToEthAttestedTestSuite) Test_Deploy() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	s.Require().True(s.Run("Verify ICS26 on Chain A", func() {
		transferAddress, err := s.ics26ContractA.GetIBCApp(nil, "transfer")
		s.Require().NoError(err)
		s.Require().Equal(strings.ToLower(s.contractAddressesA.Ics20Transfer), strings.ToLower(transferAddress.Hex()))
	}))

	s.Require().True(s.Run("Verify ICS26 on Chain B", func() {
		transferAddress, err := s.ics26ContractB.GetIBCApp(nil, "transfer")
		s.Require().NoError(err)
		s.Require().Equal(strings.ToLower(s.contractAddressesB.Ics20Transfer), strings.ToLower(transferAddress.Hex()))
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

	transferAmount := big.NewInt(testvalues.TransferAmount)
	userAddressA := crypto.PubkeyToAddress(s.userKeyA.PublicKey)
	userAddressB := crypto.PubkeyToAddress(s.userKeyB.PublicKey)
	ics20AddressA := ethcommon.HexToAddress(s.contractAddressesA.Ics20Transfer)
	ics20AddressB := ethcommon.HexToAddress(s.contractAddressesB.Ics20Transfer)
	ics26AddressA := ethcommon.HexToAddress(s.contractAddressesA.Ics26Router)
	ics26AddressB := ethcommon.HexToAddress(s.contractAddressesB.Ics26Router)
	erc20AddressA := ethcommon.HexToAddress(s.contractAddressesA.Erc20)

	// Store initial balance
	initialBalanceA := new(big.Int).Set(testvalues.StartingERC20Balance)

	// Variables to track IBC denom and contract on Chain B
	var ibcDenomOnB string
	var ibcERC20OnB *ibcerc20.Contract
	var ibcERC20AddressOnB ethcommon.Address

	// ========== PHASE 1: Transfer from Chain A to Chain B ==========

	s.Require().True(s.Run("(A -> B): Approve ICS20 on Chain A", func() {
		tx, err := s.erc20ContractA.Approve(s.GetTransactOpts(s.userKeyA, s.EthChainA()), ics20AddressA, transferAmount)
		s.Require().NoError(err)

		receipt, err := s.EthChainA().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	var sendTxHashAtoB []byte
	s.Require().True(s.Run("(A -> B): Send transfer from Chain A to Chain B", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20AddressA,
			Amount:           transferAmount,
			Receiver:         strings.ToLower(userAddressB.Hex()),
			TimeoutTimestamp: timeout,
			SourceClient:     ClientA,
			DestPort:         "transfer",
			Memo:             "",
		}

		tx, err := s.ics20ContractA.SendTransfer(s.GetTransactOpts(s.userKeyA, s.EthChainA()), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := s.EthChainA().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		sendTxHashAtoB = tx.Hash().Bytes()
		s.T().Logf("Send tx hash A->B: %s", tx.Hash().Hex())
	}))

	s.Require().True(s.Run("(A -> B): Verify balances after send on Chain A", func() {
		// Verify escrow balance on Chain A
		escrowAddress, err := s.ics20ContractA.GetEscrow(nil, ClientA)
		s.Require().NoError(err)

		escrowBalance, err := s.erc20ContractA.BalanceOf(nil, escrowAddress)
		s.Require().NoError(err)
		s.Require().Equal(0, transferAmount.Cmp(escrowBalance), "Escrow should hold transfer amount")

		// Verify user balance decreased
		userBalance, err := s.erc20ContractA.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		expectedBalance := new(big.Int).Sub(initialBalanceA, transferAmount)
		s.Require().Equal(0, expectedBalance.Cmp(userBalance), "User A balance should decrease by transfer amount")
	}))

	var recvTxHashOnB []byte
	s.Require().True(s.Run("(A -> B): Relay packet to Chain B and receive", func() {
		var relayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.EthChainA().ChainID.String(),
				DstChain:    s.EthChainB().ChainID.String(),
				SourceTxIds: [][]byte{sendTxHashAtoB},
				SrcClientId: ClientA,
				DstClientId: ClientB,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(ics26AddressB.String(), resp.Address)

			relayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx on Chain B", func() {
			receipt, err := s.EthChainB().BroadcastTx(ctx, s.EthRelayerSubmitterB, 15_000_000, &ics26AddressB, relayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			recvTxHashOnB = receipt.TxHash.Bytes()

			// Verify WriteAcknowledgement event
			ackEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26ContractB.ParseWriteAcknowledgement)
			s.Require().NoError(err)
			s.T().Logf("WriteAcknowledgement event received for packet seq %d", ackEvent.Packet.Sequence)

			// Store the IBC denom for later use
			destPort := ackEvent.Packet.Payloads[0].DestPort
			destClient := ackEvent.Packet.DestClient
			ibcDenomOnB = fmt.Sprintf("%s/%s/%s", destPort, destClient, strings.ToLower(erc20AddressA.Hex()))

			var err2 error
			ibcERC20AddressOnB, err2 = s.ics20ContractB.IbcERC20Contract(nil, ibcDenomOnB)
			s.Require().NoError(err2)

			ibcERC20OnB, err2 = ibcerc20.NewContract(ibcERC20AddressOnB, s.EthChainB().RPCClient)
			s.Require().NoError(err2)
		}))
	}))

	s.Require().True(s.Run("(A -> B): Verify balances on Chain B after receive", func() {
		// Verify user balance on Chain B
		userBalanceB, err := ibcERC20OnB.BalanceOf(nil, userAddressB)
		s.Require().NoError(err)
		s.Require().Equal(0, transferAmount.Cmp(userBalanceB), "User B should have received tokens")
		s.T().Logf("User B balance on Chain B: %s", userBalanceB.String())
	}))

	s.Require().True(s.Run("(A -> B): Relay acknowledgement to Chain A", func() {
		var ackRelayTx []byte
		s.Require().True(s.Run("Retrieve ack relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.EthChainB().ChainID.String(),
				DstChain:    s.EthChainA().ChainID.String(),
				SourceTxIds: [][]byte{recvTxHashOnB},
				SrcClientId: ClientB,
				DstClientId: ClientA,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			ackRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast ack relay tx on Chain A", func() {
			receipt, err := s.EthChainA().BroadcastTx(ctx, s.EthRelayerSubmitterA, 15_000_000, &ics26AddressA, ackRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Ack tx failed: %+v", receipt))

			// Verify AckPacket event
			ackEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26ContractA.ParseAckPacket)
			s.Require().NoError(err)
			s.T().Logf("AckPacket event received for packet seq %d", ackEvent.Packet.Sequence)
		}))
	}))

	// ========== PHASE 2: Transfer back from Chain B to Chain A ==========

	s.Require().True(s.Run("(B -> A): Approve ICS20 on Chain B", func() {
		tx, err := ibcERC20OnB.Approve(s.GetTransactOpts(s.userKeyB, s.EthChainB()), ics20AddressB, transferAmount)
		s.Require().NoError(err)

		receipt, err := s.EthChainB().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	var sendTxHashBtoA []byte
	s.Require().True(s.Run("(B -> A): Send transfer from Chain B to Chain A", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            ibcERC20AddressOnB,
			Amount:           transferAmount,
			Receiver:         strings.ToLower(userAddressA.Hex()),
			TimeoutTimestamp: timeout,
			SourceClient:     ClientB,
			DestPort:         "transfer",
			Memo:             "",
		}

		tx, err := s.ics20ContractB.SendTransfer(s.GetTransactOpts(s.userKeyB, s.EthChainB()), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := s.EthChainB().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		sendTxHashBtoA = tx.Hash().Bytes()
		s.T().Logf("Send tx hash B->A: %s", tx.Hash().Hex())
	}))

	s.Require().True(s.Run("(B -> A): Verify balances after send on Chain B", func() {
		// User B balance should be zero (tokens burned for unwind)
		userBalanceB, err := ibcERC20OnB.BalanceOf(nil, userAddressB)
		s.Require().NoError(err)
		s.Require().Zero(userBalanceB.Int64(), "User B balance should be zero after sending back")
	}))

	var recvTxHashOnA []byte
	s.Require().True(s.Run("(B -> A): Relay packet to Chain A and receive", func() {
		var relayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.EthChainB().ChainID.String(),
				DstChain:    s.EthChainA().ChainID.String(),
				SourceTxIds: [][]byte{sendTxHashBtoA},
				SrcClientId: ClientB,
				DstClientId: ClientA,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(ics26AddressA.String(), resp.Address)

			relayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx on Chain A", func() {
			receipt, err := s.EthChainA().BroadcastTx(ctx, s.EthRelayerSubmitterA, 15_000_000, &ics26AddressA, relayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			recvTxHashOnA = receipt.TxHash.Bytes()

			// Verify WriteAcknowledgement event
			ackEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26ContractA.ParseWriteAcknowledgement)
			s.Require().NoError(err)
			s.T().Logf("WriteAcknowledgement event received for packet seq %d", ackEvent.Packet.Sequence)
		}))
	}))

	s.Require().True(s.Run("(B -> A): Verify balances on Chain A after receive", func() {
		// User A should have their tokens back
		userBalanceA, err := s.erc20ContractA.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		s.Require().Equal(0, initialBalanceA.Cmp(userBalanceA), "User A should have original balance restored")
		s.T().Logf("User A balance restored: %s", userBalanceA.String())

		// Escrow should be empty
		escrowAddress, err := s.ics20ContractA.GetEscrow(nil, ClientA)
		s.Require().NoError(err)

		escrowBalance, err := s.erc20ContractA.BalanceOf(nil, escrowAddress)
		s.Require().NoError(err)
		s.Require().Zero(escrowBalance.Int64(), "Escrow should be empty after unwind")
	}))

	s.Require().True(s.Run("(B -> A): Relay final acknowledgement to Chain B", func() {
		var ackRelayTx []byte
		s.Require().True(s.Run("Retrieve ack relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.EthChainA().ChainID.String(),
				DstChain:    s.EthChainB().ChainID.String(),
				SourceTxIds: [][]byte{recvTxHashOnA},
				SrcClientId: ClientA,
				DstClientId: ClientB,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			ackRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast ack relay tx on Chain B", func() {
			receipt, err := s.EthChainB().BroadcastTx(ctx, s.EthRelayerSubmitterB, 15_000_000, &ics26AddressB, ackRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Ack tx failed: %+v", receipt))

			// Verify AckPacket event
			ackEvent, err := e2esuite.GetEvmEvent(receipt, s.ics26ContractB.ParseAckPacket)
			s.Require().NoError(err)
			s.T().Logf("Final AckPacket event received for packet seq %d", ackEvent.Packet.Sequence)
		}))
	}))
}

func (s *EthToEthAttestedTestSuite) Test_TimeoutPacketFromChainA() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	transferAmount := big.NewInt(testvalues.TransferAmount)
	userAddressA := crypto.PubkeyToAddress(s.userKeyA.PublicKey)
	userAddressB := crypto.PubkeyToAddress(s.userKeyB.PublicKey)
	ics20AddressA := ethcommon.HexToAddress(s.contractAddressesA.Ics20Transfer)
	ics26AddressA := ethcommon.HexToAddress(s.contractAddressesA.Ics26Router)
	erc20AddressA := ethcommon.HexToAddress(s.contractAddressesA.Erc20)

	var originalBalanceA *big.Int
	s.Require().True(s.Run("Get initial balances", func() {
		var err error
		originalBalanceA, err = s.erc20ContractA.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		s.T().Logf("User A initial balance: %s", originalBalanceA.String())
	}))

	s.Require().True(s.Run("Approve ICS20 on Chain A", func() {
		tx, err := s.erc20ContractA.Approve(s.GetTransactOpts(s.userKeyA, s.EthChainA()), ics20AddressA, transferAmount)
		s.Require().NoError(err)

		receipt, err := s.EthChainA().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	var sendTxHash []byte
	var packetTimeout uint64
	s.Require().True(s.Run("Send transfer with short timeout", func() {
		// Set timeout to 30 seconds from now
		packetTimeout = uint64(time.Now().Add(30 * time.Second).Unix())
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20AddressA,
			Amount:           transferAmount,
			Receiver:         strings.ToLower(userAddressB.Hex()),
			TimeoutTimestamp: packetTimeout,
			SourceClient:     ClientA,
			DestPort:         "transfer",
			Memo:             "",
		}

		tx, err := s.ics20ContractA.SendTransfer(s.GetTransactOpts(s.userKeyA, s.EthChainA()), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := s.EthChainA().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		sendTxHash = tx.Hash().Bytes()
		s.T().Logf("Send tx hash: %s", tx.Hash().Hex())

		// Verify user balance decreased
		userBalance, err := s.erc20ContractA.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		expectedBalance := new(big.Int).Sub(originalBalanceA, transferAmount)
		s.Require().Equal(expectedBalance, userBalance)
	}))

	s.Require().True(s.Run("Wait for timeout to elapse on Chain B", func() {
		startTime := time.Now()
		s.T().Logf("Waiting for Chain B timestamp to exceed packet timeout %d", packetTimeout)
		for {
			header, err := s.EthChainB().RPCClient.HeaderByNumber(ctx, nil)
			s.Require().NoError(err)
			chainBTimestamp := header.Time
			if chainBTimestamp > packetTimeout {
				s.T().Logf("Chain B timestamp %d exceeded packet timeout %d (waited %s)", chainBTimestamp, packetTimeout, time.Since(startTime))
				break
			}
			time.Sleep(1 * time.Second)
		}
	}))

	var timeoutRelayTx []byte
	s.Require().True(s.Run("Retrieve timeout relay tx", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:     s.EthChainB().ChainID.String(),
			DstChain:     s.EthChainA().ChainID.String(),
			TimeoutTxIds: [][]byte{sendTxHash},
			SrcClientId:  ClientB,
			DstClientId:  ClientA,
		})
		s.Require().NoError(err, "Failed to get timeout relay tx from proof API")
		s.Require().NotEmpty(resp.Tx, "Timeout relay tx should not be empty")
		timeoutRelayTx = resp.Tx
	}))

	s.Require().True(s.Run("Broadcast timeout tx on Chain A", func() {
		receipt, err := s.EthChainA().BroadcastTx(ctx, s.EthRelayerSubmitterA, 15_000_000, &ics26AddressA, timeoutRelayTx)
		s.Require().NoError(err, "Failed to broadcast timeout tx")
		if receipt.Status != ethtypes.ReceiptStatusSuccessful {
			s.T().Logf("Timeout tx failed with status %d, gas used: %d, logs: %+v", receipt.Status, receipt.GasUsed, receipt.Logs)
			// Try to get revert reason
			s.T().Logf("Receipt block: %d, tx hash: %s", receipt.BlockNumber, receipt.TxHash.Hex())
		}
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status,
			"Timeout tx failed - check logs above for details")
	}))

	s.Require().True(s.Run("Verify tokens refunded to user", func() {
		// After successful timeout, tokens should be refunded from escrow
		escrowAddress, err := s.ics20ContractA.GetEscrow(nil, ClientA)
		s.Require().NoError(err)

		escrowBalance, err := s.erc20ContractA.BalanceOf(nil, escrowAddress)
		s.Require().NoError(err)
		s.Require().Zero(escrowBalance.Int64(), "Escrow should be empty after timeout refund")

		// User balance should be restored
		userBalance, err := s.erc20ContractA.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		s.Require().Equal(0, originalBalanceA.Cmp(userBalance), "User balance should be restored after timeout")
	}))

	s.T().Log("Timeout packet from Chain A completed successfully")
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
		ics26AddressB := ethcommon.HexToAddress(s.contractAddressesB.Ics26Router)
		receipt, err := s.EthChainB().BroadcastTx(ctx, s.EthRelayerSubmitterB, 15_000_000, &ics26AddressB, resp.Tx)
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

	transferAmount := big.NewInt(testvalues.TransferAmount)
	userAddressB := crypto.PubkeyToAddress(s.userKeyB.PublicKey)
	ics20AddressA := ethcommon.HexToAddress(s.contractAddressesA.Ics20Transfer)
	erc20AddressA := ethcommon.HexToAddress(s.contractAddressesA.Erc20)

	s.Require().True(s.Run("Approve ICS20 on Chain A", func() {
		tx, err := s.erc20ContractA.Approve(s.GetTransactOpts(s.userKeyA, s.EthChainA()), ics20AddressA, transferAmount)
		s.Require().NoError(err)

		receipt, err := s.EthChainA().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	packetTimeout := uint64(time.Now().Unix()) + timeoutOffsetSeconds

	var sendTxHash []byte
	s.Require().True(s.Run("Send transfer with short timeout", func() {
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20AddressA,
			Amount:           transferAmount,
			Receiver:         strings.ToLower(userAddressB.Hex()),
			TimeoutTimestamp: packetTimeout,
			SourceClient:     ClientA,
			DestPort:         "transfer",
			Memo:             "",
		}

		tx, err := s.ics20ContractA.SendTransfer(s.GetTransactOpts(s.userKeyA, s.EthChainA()), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := s.EthChainA().GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		sendTxHash = tx.Hash().Bytes()
		s.T().Logf("Send tx hash: %s, packet timeout: %d (%d seconds from chain time)",
			tx.Hash().Hex(), packetTimeout, timeoutOffsetSeconds)
	}))

	s.Require().True(s.Run("Wait for timeout to pass on both chains", func() {
		// Wait for both chains to naturally pass the timeout timestamp
		// This ensures the packet is actually timed out
		s.T().Logf("Waiting for timeout to pass naturally...")
		time.Sleep(time.Duration(timeoutOffsetSeconds+5) * time.Second)

		// Verify both chains are past timeout
		headerA, err := s.EthChainA().RPCClient.HeaderByNumber(ctx, nil)
		s.Require().NoError(err)
		s.T().Logf("Chain A timestamp: %d (should be > timeout %d)", headerA.Time, packetTimeout)
		s.Require().Greater(headerA.Time, packetTimeout, "Chain A should be past timeout")

		headerB, err := s.EthChainB().RPCClient.HeaderByNumber(ctx, nil)
		s.Require().NoError(err)
		s.T().Logf("Chain B timestamp: %d (should be > timeout %d)", headerB.Time, packetTimeout)
		s.Require().Greater(headerB.Time, packetTimeout, "Chain B should be past timeout")
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
