// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"crypto/ecdsa"
	"encoding/hex"
	"fmt"
	"math/big"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/suite"

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
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"

	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ibcerc20"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics20transfer"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/sp1ics07tendermint"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	ibctesting "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ibctesting"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/erc20"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

type MultichainTestSuite struct {
	e2esuite.TestSuite

	// The private key of a test account
	key *ecdsa.PrivateKey
	// The private key of the faucet account of interchaintest
	deployer *ecdsa.PrivateKey

	contractAddresses     ethereum.DeployedContracts
	chainAsp1Ics07Address ethcommon.Address
	chainBsp1Ics07Address ethcommon.Address

	chainASP1Ics07Contract *sp1ics07tendermint.Contract
	chainBSP1Ics07Contract *sp1ics07tendermint.Contract
	ics26Contract          *ics26router.Contract
	ics20Contract          *ics20transfer.Contract
	erc20Contract          *erc20.Contract

	ProofApiClient proofapitypes.ProofApiServiceClient

	SimdARelayerSubmitter ibc.Wallet
	SimdBRelayerSubmitter ibc.Wallet
	EthRelayerSubmitter   *ecdsa.PrivateKey
}

// TestWithMultichainTestSuite is the boilerplate code that allows the test suite to be run
func TestWithMultichainTestSuite(t *testing.T) {
	suite.Run(t, new(MultichainTestSuite))
}

func (s *MultichainTestSuite) SetupSuite(ctx context.Context, proofType types.SupportedProofType) {
	chainconfig.DefaultChainSpecs = append(chainconfig.DefaultChainSpecs, chainconfig.IbcGoChainSpec("ibc-go-simd-2", "simd-2"))

	s.TestSuite.SetupSuite(ctx)

	eth, simdA, simdB := s.Eth.Chains[0], s.Cosmos.Chains[0], s.Cosmos.Chains[1]

	s.T().Logf("Setting up test suite with proof type: %s", proofType.String())

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

		s.SimdARelayerSubmitter = s.CreateAndFundCosmosUser(ctx, simdA)
		s.SimdBRelayerSubmitter = s.CreateAndFundCosmosUser(ctx, simdB)

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

		os.Setenv(testvalues.EnvKeyRustLog, testvalues.EnvValueRustLog_Info)
		os.Setenv(testvalues.EnvKeyEthRPC, eth.RPC)
		os.Setenv(testvalues.EnvKeySp1Prover, prover)
		os.Setenv(testvalues.EnvKeyOperatorPrivateKey, hex.EncodeToString(crypto.FromECDSA(operatorKey)))
	}))

	s.Require().True(s.Run("Deploy ethereum contracts with SimdA client", func() {
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

	var proofApiProcess *os.Process
	s.Require().True(s.Run("Start Proof API", func() {
		beaconAPI := ""
		// The BeaconAPIClient is nil when the testnet is `pow`
		if eth.BeaconAPIClient != nil {
			beaconAPI = eth.BeaconAPIClient.GetBeaconAPIURL()
		}

		sp1Config := proofapi.SP1ProverConfig{
			Type: prover,
		}
		if prover == testvalues.EnvValueSp1Prover_Network {
			sp1Config.PrivateCluster = os.Getenv(testvalues.EnvKeyNetworkPrivateCluster) == testvalues.EnvValueSp1Prover_PrivateCluster
		}

		mockClient := os.Getenv(testvalues.EnvKeyEthTestnetType) == testvalues.EthTestnetTypeAnvil

		// Eth↔ChainA, Eth↔ChainB, ChainA↔ChainB
		config := proofapi.NewConfigBuilder().
			// Eth ↔ ChainA
			EthToCosmos(proofapi.EthToCosmosParams{
				EthChainID:    eth.ChainID.String(),
				CosmosChainID: simdA.Config().ChainID,
				TmRPC:         simdA.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				BeaconAPI:     beaconAPI,
				SignerAddress: s.SimdARelayerSubmitter.FormattedAddress(),
				MockClient:    mockClient,
			}).
			CosmosToEthSP1(proofapi.CosmosToEthSP1Params{
				CosmosChainID: simdA.Config().ChainID,
				EthChainID:    eth.ChainID.String(),
				TmRPC:         simdA.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				Prover:        sp1Config,
			}).
			// Eth ↔ ChainB
			EthToCosmos(proofapi.EthToCosmosParams{
				EthChainID:    eth.ChainID.String(),
				CosmosChainID: simdB.Config().ChainID,
				TmRPC:         simdB.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				BeaconAPI:     beaconAPI,
				SignerAddress: s.SimdBRelayerSubmitter.FormattedAddress(),
				MockClient:    mockClient,
			}).
			CosmosToEthSP1(proofapi.CosmosToEthSP1Params{
				CosmosChainID: simdB.Config().ChainID,
				EthChainID:    eth.ChainID.String(),
				TmRPC:         simdB.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				Prover:        sp1Config,
			}).
			// ChainA ↔ ChainB (signer must exist on destination chain)
			CosmosToCosmos(proofapi.CosmosToCosmosParams{
				SrcChainID:    simdA.Config().ChainID,
				DstChainID:    simdB.Config().ChainID,
				SrcRPC:        simdA.GetHostRPCAddress(),
				DstRPC:        simdB.GetHostRPCAddress(),
				SignerAddress: s.SimdBRelayerSubmitter.FormattedAddress(),
			}).
			CosmosToCosmos(proofapi.CosmosToCosmosParams{
				SrcChainID:    simdB.Config().ChainID,
				DstChainID:    simdA.Config().ChainID,
				SrcRPC:        simdB.GetHostRPCAddress(),
				DstRPC:        simdA.GetHostRPCAddress(),
				SignerAddress: s.SimdARelayerSubmitter.FormattedAddress(),
			}).
			Build()

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
		time.Sleep(5 * time.Second) // wait for the proof API to start

		var err error
		s.ProofApiClient, err = proofapi.GetGRPCClient(proofapi.DefaultProofAPIGRPCAddress())
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Deploy SP1 ICS07 contracts", func() {
		verifierAddress, err := s.contractAddresses.GetVerifierAddress(prover, proofType.String())
		s.Require().NoError(err)

		var createClientTxBz []byte
		s.Require().True(s.Run("Retrieve create client tx for ChainA's client", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: simdA.Config().ChainID,
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

		s.Require().True(s.Run("Broadcast relay tx for ChainA's client", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, nil, createClientTxBz)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			s.chainAsp1Ics07Address = receipt.ContractAddress
			s.chainASP1Ics07Contract, err = sp1ics07tendermint.NewContract(s.chainAsp1Ics07Address, eth.RPCClient)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Retrieve create client tx for ChainB's client", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: simdB.Config().ChainID,
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

		s.Require().True(s.Run("Broadcast relay tx for ChainB's client", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, nil, createClientTxBz)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			s.chainBsp1Ics07Address = receipt.ContractAddress
			s.chainBSP1Ics07Contract, err = sp1ics07tendermint.NewContract(s.chainBsp1Ics07Address, eth.RPCClient)
			s.Require().NoError(err)
		}))
	}))

	s.Require().True(s.Run("Fund address with ERC20", func() {
		tx, err := s.erc20Contract.Transfer(s.GetTransactOpts(eth.Faucet, eth), crypto.PubkeyToAddress(s.key.PublicKey), testvalues.StartingERC20Balance)
		s.Require().NoError(err)

		_, err = eth.GetTxReciept(ctx, tx.Hash()) // wait for the tx to be mined
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Add ethereum light client on SimdA", func() {
		checksumHex := s.StoreLightClient(ctx, simdA, s.SimdARelayerSubmitter)
		s.Require().NotEmpty(checksumHex)

		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: eth.ChainID.String(),
				DstChain: simdA.Config().ChainID,
				Parameters: map[string]string{
					testvalues.ParameterKey_ChecksumHex: checksumHex,
				},
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simdA, s.SimdARelayerSubmitter, 20_000_000, createClientTxBodyBz)
			clientId, err := cosmos.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(testvalues.FirstWasmClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Add simdA client and counterparty on EVM", func() {
		counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
			ClientId:     testvalues.FirstWasmClientID,
			MerklePrefix: [][]byte{[]byte(ibcexported.StoreKey), []byte("")},
		}
		tx, err := s.ics26Contract.AddClient0(s.GetTransactOpts(s.deployer, eth), counterpartyInfo, s.chainAsp1Ics07Address)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)

		event, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseICS02ClientAdded)
		s.Require().NoError(err)
		s.Require().Equal(testvalues.FirstUniversalClientID, event.ClientId)
		s.Require().Equal(testvalues.FirstWasmClientID, event.CounterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Add ethereum light client on SimdB", func() {
		checksumHex := s.StoreLightClient(ctx, simdB, s.SimdBRelayerSubmitter)
		s.Require().NotEmpty(checksumHex)

		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: eth.ChainID.String(),
				DstChain: simdB.Config().ChainID,
				Parameters: map[string]string{
					testvalues.ParameterKey_ChecksumHex: checksumHex,
				},
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simdB, s.SimdBRelayerSubmitter, 20_000_000, createClientTxBodyBz)
			clientId, err := cosmos.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(testvalues.FirstWasmClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Add simdB client and counterparty on EVM", func() {
		counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
			ClientId:     testvalues.FirstWasmClientID,
			MerklePrefix: [][]byte{[]byte(ibcexported.StoreKey), []byte("")},
		}
		tx, err := s.ics26Contract.AddClient0(s.GetTransactOpts(s.deployer, eth), counterpartyInfo, s.chainBsp1Ics07Address)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)

		event, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseICS02ClientAdded)
		s.Require().NoError(err)
		s.Require().Equal(testvalues.SecondUniversalClientID, event.ClientId)
		s.Require().Equal(testvalues.FirstWasmClientID, event.CounterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Register counterparty on SimdA", func() {
		merklePathPrefix := [][]byte{[]byte("")}

		_, err := s.BroadcastMessages(ctx, simdA, s.SimdARelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 testvalues.FirstWasmClientID,
			CounterpartyClientId:     testvalues.FirstUniversalClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			Signer:                   s.SimdARelayerSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Register counterparty on SimdB", func() {
		merklePathPrefix := [][]byte{[]byte("")}

		_, err := s.BroadcastMessages(ctx, simdB, s.SimdBRelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 testvalues.FirstWasmClientID,
			CounterpartyClientId:     testvalues.SecondUniversalClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			Signer:                   s.SimdBRelayerSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Create Light Client of Chain A on Chain B", func() {
		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: simdA.Config().ChainID,
				DstChain: simdB.Config().ChainID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simdB, s.SimdBRelayerSubmitter, 2_000_000, createClientTxBodyBz)
			clientId, err := cosmos.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(ibctesting.SecondClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Create Light Client of Chain B on Chain A", func() {
		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: simdB.Config().ChainID,
				DstChain: simdA.Config().ChainID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simdA, s.SimdARelayerSubmitter, 2_000_000, createClientTxBodyBz)
			clientId, err := cosmos.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(ibctesting.SecondClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Create Channel and register counterparty on Chain A", func() {
		merklePathPrefix := [][]byte{[]byte(ibcexported.StoreKey), []byte("")}

		// We can do this because we know what the counterparty channel ID will be
		_, err := s.BroadcastMessages(ctx, simdA, s.SimdARelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 ibctesting.SecondClientID,
			CounterpartyClientId:     ibctesting.SecondClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			Signer:                   s.SimdARelayerSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Create Channel and register counterparty on Chain B", func() {
		merklePathPrefix := [][]byte{[]byte(ibcexported.StoreKey), []byte("")}

		_, err := s.BroadcastMessages(ctx, simdB, s.SimdBRelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 ibctesting.SecondClientID,
			CounterpartyClientId:     ibctesting.SecondClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			Signer:                   s.SimdBRelayerSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))
}

func (s *MultichainTestSuite) Test_Deploy() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()

	s.SetupSuite(ctx, proofType)

	eth, simdA, simdB := s.Eth.Chains[0], s.Cosmos.Chains[0], s.Cosmos.Chains[1]

	s.Require().True(s.Run("Verify SimdA SP1 Client", func() {
		clientState, err := s.chainASP1Ics07Contract.ClientState(nil)
		s.Require().NoError(err)

		stakingParams, err := simdA.StakingQueryParams(ctx)
		s.Require().NoError(err)

		s.Require().Equal(simdA.Config().ChainID, clientState.ChainId)
		s.Require().Equal(uint8(testvalues.DefaultTrustLevel.Numerator), clientState.TrustLevel.Numerator)
		s.Require().Equal(uint8(testvalues.DefaultTrustLevel.Denominator), clientState.TrustLevel.Denominator)
		s.Require().Equal(uint32(testvalues.DefaultTrustPeriod), clientState.TrustingPeriod)
		s.Require().Equal(uint32(stakingParams.UnbondingTime.Seconds()), clientState.UnbondingPeriod)
		s.Require().False(clientState.IsFrozen)
		s.Require().Equal(uint64(1), clientState.LatestHeight.RevisionNumber)
		s.Require().Greater(clientState.LatestHeight.RevisionHeight, uint64(0))
	}))

	s.Require().True(s.Run("Verify SimdB SP1 Client", func() {
		clientState, err := s.chainBSP1Ics07Contract.ClientState(nil)
		s.Require().NoError(err)

		stakingParams, err := simdB.StakingQueryParams(ctx)
		s.Require().NoError(err)

		s.Require().Equal(simdB.Config().ChainID, clientState.ChainId)
		s.Require().Equal(uint8(testvalues.DefaultTrustLevel.Numerator), clientState.TrustLevel.Numerator)
		s.Require().Equal(uint8(testvalues.DefaultTrustLevel.Denominator), clientState.TrustLevel.Denominator)
		s.Require().Equal(uint32(testvalues.DefaultTrustPeriod), clientState.TrustingPeriod)
		s.Require().Equal(uint32(stakingParams.UnbondingTime.Seconds()), clientState.UnbondingPeriod)
		s.Require().False(clientState.IsFrozen)
		s.Require().Equal(uint64(2), clientState.LatestHeight.RevisionNumber)
		s.Require().Greater(clientState.LatestHeight.RevisionHeight, uint64(0))
	}))

	s.Require().True(s.Run("Verify ICS02 Client", func() {
		clientAddress, err := s.ics26Contract.GetClient(nil, testvalues.FirstUniversalClientID)
		s.Require().NoError(err)
		s.Require().Equal(s.chainAsp1Ics07Address, clientAddress)

		counterpartyInfo, err := s.ics26Contract.GetCounterparty(nil, testvalues.FirstUniversalClientID)
		s.Require().NoError(err)
		s.Require().Equal(testvalues.FirstWasmClientID, counterpartyInfo.ClientId)

		clientAddress, err = s.ics26Contract.GetClient(nil, testvalues.SecondUniversalClientID)
		s.Require().NoError(err)
		s.Require().Equal(s.chainBsp1Ics07Address, clientAddress)

		counterpartyInfo, err = s.ics26Contract.GetCounterparty(nil, testvalues.SecondUniversalClientID)
		s.Require().NoError(err)
		s.Require().Equal(testvalues.FirstWasmClientID, counterpartyInfo.ClientId)
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

	s.Require().True(s.Run("Verify ethereum light client for SimdA", func() {
		_, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simdA, &clienttypes.QueryClientStateRequest{
			ClientId: testvalues.FirstWasmClientID,
		})
		s.Require().NoError(err)

		counterpartyInfoResp, err := e2esuite.GRPCQuery[clienttypesv2.QueryCounterpartyInfoResponse](ctx, simdA, &clienttypesv2.QueryCounterpartyInfoRequest{
			ClientId: testvalues.FirstWasmClientID,
		})
		s.Require().NoError(err)
		s.Require().Equal(testvalues.FirstUniversalClientID, counterpartyInfoResp.CounterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Verify ethereum light client for SimdB", func() {
		_, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simdB, &clienttypes.QueryClientStateRequest{
			ClientId: testvalues.FirstWasmClientID,
		})
		s.Require().NoError(err)

		counterpartyInfoResp, err := e2esuite.GRPCQuery[clienttypesv2.QueryCounterpartyInfoResponse](ctx, simdB, &clienttypesv2.QueryCounterpartyInfoRequest{
			ClientId: testvalues.FirstWasmClientID,
		})
		s.Require().NoError(err)
		s.Require().Equal(testvalues.SecondUniversalClientID, counterpartyInfoResp.CounterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Verify Light Client of Chain A on Chain B", func() {
		clientStateResp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simdB, &clienttypes.QueryClientStateRequest{
			ClientId: ibctesting.SecondClientID,
		})
		s.Require().NoError(err)
		s.Require().NotZero(clientStateResp.ClientState.Value)

		var clientState ibctm.ClientState
		err = proto.Unmarshal(clientStateResp.ClientState.Value, &clientState)
		s.Require().NoError(err)
		s.Require().Equal(simdA.Config().ChainID, clientState.ChainId)
	}))

	s.Require().True(s.Run("Verify Light Client of Chain B on Chain A", func() {
		clientStateResp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simdA, &clienttypes.QueryClientStateRequest{
			ClientId: ibctesting.SecondClientID,
		})
		s.Require().NoError(err)
		s.Require().NotZero(clientStateResp.ClientState.Value)

		var clientState ibctm.ClientState
		err = proto.Unmarshal(clientStateResp.ClientState.Value, &clientState)
		s.Require().NoError(err)
		s.Require().Equal(simdB.Config().ChainID, clientState.ChainId)
	}))

	time.Sleep(5 * time.Second) // wait for the proof API to start

	s.Require().True(s.Run("Verify SimdA to Eth Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: simdA.Config().ChainID,
			DstChain: eth.ChainID.String(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(simdA.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(eth.ChainID.String(), info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Eth to SimdA Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: eth.ChainID.String(),
			DstChain: simdA.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(eth.ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(simdA.Config().ChainID, info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify SimdB to Eth Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: simdB.Config().ChainID,
			DstChain: eth.ChainID.String(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(simdB.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(eth.ChainID.String(), info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Eth to SimdB Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: eth.ChainID.String(),
			DstChain: simdB.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(eth.ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(simdB.Config().ChainID, info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Chain A to Chain B Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: simdA.Config().ChainID,
			DstChain: simdB.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(simdA.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(simdB.Config().ChainID, info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Chain B to Chain A Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: simdB.Config().ChainID,
			DstChain: simdA.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(simdB.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(simdA.Config().ChainID, info.TargetChain.ChainId)
	}))
}

func (s *MultichainTestSuite) Test_TransferCosmosToEthToCosmosAndBack() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()

	s.SetupSuite(ctx, proofType)

	eth, simdA, simdB := s.Eth.Chains[0], s.Cosmos.Chains[0], s.Cosmos.Chains[1]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	transferAmount := big.NewInt(testvalues.TransferAmount)
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	simdAUser, simdBUser := s.Cosmos.Users[0], s.Cosmos.Users[1]

	var simdASendTxHash []byte
	s.Require().True(s.Run("Send transfer on SimdA chain", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		transferCoin := sdk.NewCoin(simdA.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    transferCoin.Denom,
			Amount:   transferCoin.Amount.String(),
			Sender:   simdAUser.FormattedAddress(),
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
		msgSendPacket := channeltypesv2.MsgSendPacket{
			SourceClient:     testvalues.FirstWasmClientID,
			TimeoutTimestamp: timeout,
			Payloads: []channeltypesv2.Payload{
				payload,
			},
			Signer: simdAUser.FormattedAddress(),
		}

		resp, err := s.BroadcastMessages(ctx, simdA, simdAUser, 200_000, &msgSendPacket)
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		simdASendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			// Check the balance of UserB
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdA, &banktypes.QueryBalanceRequest{
				Address: simdAUser.FormattedAddress(),
				Denom:   transferCoin.Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance-testvalues.TransferAmount, resp.Balance.Amount.Int64())
		}))
	}))

	var (
		ibcERC20        *ibcerc20.Contract
		ibcERC20Address ethcommon.Address
	)
	s.Require().True(s.Run("Receive packet on Ethereum", func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simdA.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{simdASendTxHash},
				SrcClientId: testvalues.FirstWasmClientID,
				DstClientId: testvalues.FirstUniversalClientID,
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
			// ackTxHash = receipt.TxHash.Bytes()
			// NOTE: ackTxHash is not used in the test since acking the packet is not necessary
		}))

		// Recreate the full denom path
		transferCoin := sdk.NewCoin(simdA.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))
		denomOnEthereum := transfertypes.NewDenom(transferCoin.Denom, transfertypes.NewHop(packet.Payloads[0].DestPort, packet.DestClient))

		var err error
		ibcERC20Address, err = s.ics20Contract.IbcERC20Contract(nil, denomOnEthereum.Path())
		s.Require().NoError(err)

		ibcERC20, err = ibcerc20.NewContract(ibcERC20Address, eth.RPCClient)
		s.Require().NoError(err)

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
	s.Require().True(s.Run("Transfer tokens from Ethereum to SimdB", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            ibcERC20Address,
			Amount:           transferAmount,
			Receiver:         simdBUser.FormattedAddress(),
			SourceClient:     testvalues.SecondUniversalClientID,
			DestPort:         transfertypes.PortID,
			TimeoutTimestamp: timeout,
			Memo:             "",
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		ethSendTxHash = tx.Hash().Bytes()

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

	var finalDenom transfertypes.Denom
	s.Require().True(s.Run("Receive packet on SimdB", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simdB.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: testvalues.SecondUniversalClientID,
				DstClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simdB, s.SimdBRelayerSubmitter, 2_000_000, relayTxBodyBz)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			finalDenom = transfertypes.NewDenom(
				simdA.Config().Denom,
				transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID),
				transfertypes.NewHop(transfertypes.PortID, testvalues.FirstUniversalClientID),
			)

			// Check the balance of UserB
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdB, &banktypes.QueryBalanceRequest{
				Address: simdBUser.FormattedAddress(),
				Denom:   finalDenom.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.TransferAmount, resp.Balance.Amount.Int64())
			s.Require().Equal(finalDenom.IBCDenom(), resp.Balance.Denom)
		}))
	}))

	// Transfer back (unwind)
	s.Require().True(s.Run("Transfer tokens from SimdB to Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    finalDenom.Path(), // XXX: IBCDenom()?
			Amount:   transferAmount.String(),
			Sender:   simdBUser.FormattedAddress(),
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

		resp, err := s.BroadcastMessages(ctx, simdB, simdBUser, 200_000, &channeltypesv2.MsgSendPacket{
			SourceClient:     testvalues.FirstWasmClientID,
			TimeoutTimestamp: timeout,
			Payloads: []channeltypesv2.Payload{
				payload,
			},
			Signer: simdBUser.FormattedAddress(),
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		simdBSendTxHash, err := hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)

		s.Require().True(s.Run("Receive packet on Ethereum", func() {
			var relayTxBodyBz []byte
			s.Require().True(s.Run("Retrieve relay tx", func() {
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    simdB.Config().ChainID,
					DstChain:    eth.ChainID.String(),
					SourceTxIds: [][]byte{simdBSendTxHash},
					SrcClientId: testvalues.FirstWasmClientID,
					DstClientId: testvalues.SecondUniversalClientID,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				s.Require().Equal(ics26Address.String(), resp.Address)

				relayTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run("Submit relay tx", func() {
				receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, relayTxBodyBz)
				s.Require().NoError(err)
				s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))
			}))

			s.True(s.Run("Verify balances on Ethereum", func() {
				userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
				s.Require().NoError(err)
				s.Require().Equal(transferAmount, userBalance)

				// ICS20 contract balance on Ethereum
				ics20TransferBalance, err := ibcERC20.BalanceOf(nil, ics20Address)
				s.Require().NoError(err)
				s.Require().Zero(ics20TransferBalance.Int64())
			}))
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

	s.Require().True(s.Run("Transfer tokens from Ethereum to SimdA", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            ibcERC20Address,
			Amount:           transferAmount,
			Receiver:         simdAUser.FormattedAddress(),
			SourceClient:     testvalues.FirstUniversalClientID,
			DestPort:         transfertypes.PortID,
			TimeoutTimestamp: timeout,
			Memo:             "",
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		ethSendTxHash = tx.Hash().Bytes()
	}))

	s.Require().True(s.Run("Receive packet on SimdA", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simdA.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: testvalues.FirstUniversalClientID,
				DstClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simdA, s.SimdARelayerSubmitter, 2_000_000, relayTxBodyBz)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdA, &banktypes.QueryBalanceRequest{
				Address: simdAUser.FormattedAddress(),
				Denom:   simdA.Config().Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance, resp.Balance.Amount.Int64())
		}))
	}))
}

func (s *MultichainTestSuite) Test_TransferEthToCosmosToCosmosAndBack() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()

	s.SetupSuite(ctx, proofType)

	eth, simdA, simdB := s.Eth.Chains[0], s.Cosmos.Chains[0], s.Cosmos.Chains[1]

	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	transferAmount := big.NewInt(testvalues.TransferAmount)
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	simdAUser, simdBUser := s.Cosmos.Users[0], s.Cosmos.Users[1]

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
		ethSendTxHash []byte
		escrowAddress ethcommon.Address
	)
	s.Require().True(s.Run("Send from Ethereum to SimdA", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			Amount:           transferAmount,
			Receiver:         simdAUser.FormattedAddress(),
			SourceClient:     testvalues.FirstUniversalClientID,
			DestPort:         transfertypes.PortID,
			TimeoutTimestamp: timeout,
			Memo:             "",
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
		s.Require().NoError(err)
		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		ethSendTxHash = tx.Hash().Bytes()

		s.True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(new(big.Int).Sub(testvalues.StartingERC20Balance, transferAmount), userBalance)

			// Get the escrow contract address
			escrowAddress, err = s.ics20Contract.GetEscrow(nil, testvalues.FirstUniversalClientID)
			s.Require().NoError(err)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Equal(transferAmount, escrowBalance)
		}))
	}))

	s.Require().True(s.Run("Receive packets on SimdA", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simdA.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: testvalues.FirstUniversalClientID,
				DstClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simdA, s.SimdARelayerSubmitter, 2_000_000, relayTxBodyBz)
			// NOTE: We don't need to check the response since we don't need to acknowledge the packet
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			denomOnSimdA := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))

			// User balance on Cosmos chain
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdA, &banktypes.QueryBalanceRequest{
				Address: simdAUser.FormattedAddress(),
				Denom:   denomOnSimdA.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(transferAmount, resp.Balance.Amount.BigInt())
			s.Require().Equal(denomOnSimdA.IBCDenom(), resp.Balance.Denom)
		}))
	}))

	var simdASendTxHash []byte
	s.Require().True(s.Run("Send from SimdA to SimdB", func() {
		denomOnSimdA := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    denomOnSimdA.Path(),
			Amount:   transferAmount.String(),
			Sender:   simdAUser.FormattedAddress(),
			Receiver: simdBUser.FormattedAddress(),
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

		resp, err := s.BroadcastMessages(ctx, simdA, simdAUser, 2_000_000, &channeltypesv2.MsgSendPacket{
			SourceClient:     ibctesting.SecondClientID,
			TimeoutTimestamp: timeout,
			Payloads: []channeltypesv2.Payload{
				payload,
			},
			Signer: simdAUser.FormattedAddress(),
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		simdASendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
	}))

	var finalDenom transfertypes.Denom
	s.Require().True(s.Run("Receive packet on SimdB", func() {
		var txBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx to SimdB", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simdA.Config().ChainID,
				DstChain:    simdB.Config().ChainID,
				SourceTxIds: [][]byte{simdASendTxHash},
				SrcClientId: ibctesting.SecondClientID,
				DstClientId: ibctesting.SecondClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			txBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx on SimdB", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simdB, s.SimdBRelayerSubmitter, 2_000_000, txBodyBz)
			// NOTE: We don't need to check the response since we don't need to acknowledge the packet
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			finalDenom = transfertypes.NewDenom(
				s.contractAddresses.Erc20,
				transfertypes.NewHop(transfertypes.PortID, ibctesting.SecondClientID),
				transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID),
			)

			// User balance on Cosmos chain
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdB, &banktypes.QueryBalanceRequest{
				Address: simdBUser.FormattedAddress(),
				Denom:   finalDenom.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(transferAmount, resp.Balance.Amount.BigInt())
			s.Require().Equal(finalDenom.IBCDenom(), resp.Balance.Denom)
		}))
	}))

	// Transfer back (unwind)
	var simdBTransferTxHash []byte
	s.Require().True(s.Run("Transfer tokens from SimdB to SimdA", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    finalDenom.Path(), // XXX: IBCDenom()?
			Amount:   transferAmount.String(),
			Sender:   simdBUser.FormattedAddress(),
			Receiver: simdAUser.FormattedAddress(),
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

		resp, err := s.BroadcastMessages(ctx, simdB, simdBUser, 2_000_000, &channeltypesv2.MsgSendPacket{
			SourceClient:     ibctesting.SecondClientID,
			TimeoutTimestamp: timeout,
			Payloads:         []channeltypesv2.Payload{payload},
			Signer:           simdBUser.FormattedAddress(),
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		simdBTransferTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Receive packet on SimdA", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simdB.Config().ChainID,
				DstChain:    simdA.Config().ChainID,
				SourceTxIds: [][]byte{simdBTransferTxHash},
				SrcClientId: ibctesting.SecondClientID,
				DstClientId: ibctesting.SecondClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx on SimdA", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simdA, s.SimdARelayerSubmitter, 2_000_000, relayTxBodyBz)
		}))

		s.Require().True(s.Run("Verify balances on SimdA", func() {
			denomOnSimdA := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdA, &banktypes.QueryBalanceRequest{
				Address: simdAUser.FormattedAddress(),
				Denom:   denomOnSimdA.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(transferAmount, resp.Balance.Amount.BigInt())
			s.Require().Equal(denomOnSimdA.IBCDenom(), resp.Balance.Denom)
		}))
	}))

	s.Require().True(s.Run("Transfer tokens from SimdA to Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		denomOnSimdA := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))
		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    denomOnSimdA.Path(),
			Amount:   transferAmount.String(),
			Sender:   simdAUser.FormattedAddress(),
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

		resp, err := s.BroadcastMessages(ctx, simdA, simdAUser, 2_000_000, &channeltypesv2.MsgSendPacket{
			SourceClient:     testvalues.FirstWasmClientID,
			TimeoutTimestamp: timeout,
			Payloads:         []channeltypesv2.Payload{payload},
			Signer:           simdAUser.FormattedAddress(),
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		simdASendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)

		s.Require().True(s.Run("Receive packet on Ethereum", func() {
			ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)

			var relayTxBodyBz []byte
			s.Require().True(s.Run("Retrieve relay tx", func() {
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    simdA.Config().ChainID,
					DstChain:    eth.ChainID.String(),
					SourceTxIds: [][]byte{simdASendTxHash},
					SrcClientId: testvalues.FirstWasmClientID,
					DstClientId: testvalues.FirstUniversalClientID,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				s.Require().Equal(ics26Address.String(), resp.Address)

				relayTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run("Submit relay tx", func() {
				receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, relayTxBodyBz)
				s.Require().NoError(err)
				s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))
			}))

			s.True(s.Run("Verify balances on Ethereum", func() {
				userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
				s.Require().NoError(err)
				s.Require().Equal(testvalues.StartingERC20Balance, userBalance)
			}))
		}))
	}))
}

func (s *MultichainTestSuite) Test_TransferCosmosToCosmosToEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()

	s.SetupSuite(ctx, proofType)

	eth, simdA, simdB := s.Eth.Chains[0], s.Cosmos.Chains[0], s.Cosmos.Chains[1]

	transferAmount := big.NewInt(testvalues.TransferAmount)
	transferCoin := sdk.NewCoin(simdA.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))
	simdAUser := s.Cosmos.Users[0]
	simdBUser := s.Cosmos.Users[1]
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)

	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)

	var simdASendTxHash []byte
	s.Require().True(s.Run("Send from SimdA to SimdB", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    transferCoin.Denom,
			Amount:   transferCoin.Amount.String(),
			Sender:   simdAUser.FormattedAddress(),
			Receiver: simdBUser.FormattedAddress(),
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

		resp, err := s.BroadcastMessages(ctx, simdA, simdAUser, 2_000_000, &channeltypesv2.MsgSendPacket{
			SourceClient:     ibctesting.SecondClientID,
			TimeoutTimestamp: timeout,
			Payloads:         []channeltypesv2.Payload{payload},
			Signer:           simdAUser.FormattedAddress(),
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		simdASendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
	}))

	denomOnSimdB := transfertypes.NewDenom(
		transferCoin.Denom,
		transfertypes.NewHop(transfertypes.PortID, ibctesting.SecondClientID),
	)
	s.Require().True(s.Run("Receive packet on SimdB", func() {
		var txBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx to SimdB", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simdA.Config().ChainID,
				DstChain:    simdB.Config().ChainID,
				SourceTxIds: [][]byte{simdASendTxHash},
				SrcClientId: ibctesting.SecondClientID,
				DstClientId: ibctesting.SecondClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			txBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx on SimdB", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simdB, s.SimdBRelayerSubmitter, 2_000_000, txBodyBz)
		}))

		s.Require().True(s.Run("Verify balances on SimdB", func() {
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdB, &banktypes.QueryBalanceRequest{
				Address: simdBUser.FormattedAddress(),
				Denom:   denomOnSimdB.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(transferAmount, resp.Balance.Amount.BigInt())
			s.Require().Equal(denomOnSimdB.IBCDenom(), resp.Balance.Denom)
		}))
	}))

	var simdBTransferTxHash []byte
	s.Require().True(s.Run("Transfer tokens from SimdB to Eth", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		transferCoin := sdk.NewCoin(denomOnSimdB.IBCDenom(), sdkmath.NewIntFromBigInt(transferAmount))
		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    denomOnSimdB.Path(),
			Amount:   transferCoin.Amount.String(),
			Sender:   simdBUser.FormattedAddress(),
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
		msgSendPacket := &channeltypesv2.MsgSendPacket{
			SourceClient:     testvalues.FirstWasmClientID,
			TimeoutTimestamp: timeout,
			Payloads:         []channeltypesv2.Payload{payload},
			Signer:           simdBUser.FormattedAddress(),
		}

		resp, err := s.BroadcastMessages(ctx, simdB, simdBUser, 2_000_000, msgSendPacket)
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		simdBTransferTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
	}))

	var denomOnEthereum transfertypes.Denom
	var ibcERC20 *ibcerc20.Contract
	s.Require().True(s.Run("Receive packet on Ethereum", func() {
		ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)

		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simdB.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{simdBTransferTxHash},
				SrcClientId: testvalues.FirstWasmClientID,
				DstClientId: testvalues.SecondUniversalClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(ics26Address.String(), resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, relayTxBodyBz)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))
			denomOnEthereum = transfertypes.NewDenom(
				simdA.Config().Denom,
				transfertypes.NewHop(transfertypes.PortID, testvalues.SecondUniversalClientID),
				transfertypes.NewHop(transfertypes.PortID, ibctesting.SecondClientID),
			)
		}))

		s.True(s.Run("Verify balances on Ethereum", func() {
			ibcERC20Address, err := s.ics20Contract.IbcERC20Contract(nil, denomOnEthereum.Path())
			s.Require().NoError(err)

			ibcERC20, err = ibcerc20.NewContract(ibcERC20Address, eth.RPCClient)
			s.Require().NoError(err)

			userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(transferAmount, userBalance)
		}))
	}))

	var ethReturnSendTxHash []byte
	s.Require().True(s.Run("Transfer tokens from Ethereum to SimdB", func() {
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

		s.Require().True(s.Run("Send packet on Ethereum", func() {
			ibcERC20Address, err := s.ics20Contract.IbcERC20Contract(nil, denomOnEthereum.Path())
			s.Require().NoError(err)
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
				Denom:            ibcERC20Address,
				Amount:           transferAmount,
				Receiver:         simdBUser.FormattedAddress(),
				TimeoutTimestamp: timeout,
				SourceClient:     testvalues.SecondUniversalClientID,
				DestPort:         transfertypes.PortID,
				Memo:             "testmemo",
			}

			tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
			s.Require().NoError(err)

			receipt, err := eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

			ethReturnSendTxHash = tx.Hash().Bytes()
		}))

		s.Require().True(s.Run("Receive packet on SimdB", func() {
			var returnRelayTxBodyBz []byte
			s.Require().True(s.Run("Retrieve relay tx", func() {
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    eth.ChainID.String(),
					DstChain:    simdB.Config().ChainID,
					SourceTxIds: [][]byte{ethReturnSendTxHash},
					SrcClientId: testvalues.SecondUniversalClientID,
					DstClientId: testvalues.FirstWasmClientID,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				s.Require().Empty(resp.Address)

				returnRelayTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run("Broadcast relay tx on SimdB", func() {
				_ = s.MustBroadcastSdkTxBody(ctx, simdB, s.SimdBRelayerSubmitter, 2_000_000, returnRelayTxBodyBz)
			}))

			s.Require().True(s.Run("Verify balances on SimdB", func() {
				resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdB, &banktypes.QueryBalanceRequest{
					Address: simdBUser.FormattedAddress(),
					Denom:   denomOnSimdB.IBCDenom(),
				})
				s.Require().NoError(err)
				s.Require().NotNil(resp.Balance)
				s.Require().Equal(transferAmount, resp.Balance.Amount.BigInt())
				s.Require().Equal(denomOnSimdB.IBCDenom(), resp.Balance.Denom)
			}))
		}))
	}))

	s.Require().True(s.Run("Transfer tokens from SimdB to SimdA", func() {
		s.Require().True(s.Run("Send packet on SimdB", func() {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			transferPayload := transfertypes.FungibleTokenPacketData{
				Denom:    denomOnSimdB.Path(),
				Amount:   transferAmount.String(),
				Sender:   simdBUser.FormattedAddress(),
				Receiver: simdAUser.FormattedAddress(),
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

			msgSendPacket := &channeltypesv2.MsgSendPacket{
				SourceClient:     ibctesting.SecondClientID,
				TimeoutTimestamp: timeout,
				Payloads:         []channeltypesv2.Payload{payload},
				Signer:           simdBUser.FormattedAddress(),
			}

			resp, err := s.BroadcastMessages(ctx, simdB, simdBUser, 2_000_000, msgSendPacket)
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.TxHash)

			simdBTransferTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Receive packet on SimdA", func() {
			var returnRelayTxBodyBz []byte
			s.Require().True(s.Run("Retrieve relay tx", func() {
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    simdB.Config().ChainID,
					DstChain:    simdA.Config().ChainID,
					SourceTxIds: [][]byte{simdBTransferTxHash},
					SrcClientId: ibctesting.SecondClientID,
					DstClientId: ibctesting.SecondClientID,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				s.Require().Empty(resp.Address)

				returnRelayTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run("Broadcast relay tx on SimdA", func() {
				_ = s.MustBroadcastSdkTxBody(ctx, simdA, s.SimdARelayerSubmitter, 2_000_000, returnRelayTxBodyBz)
			}))

			s.Require().True(s.Run("Verify balances on SimdA", func() {
				resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simdA, &banktypes.QueryBalanceRequest{
					Address: simdAUser.FormattedAddress(),
					Denom:   simdA.Config().Denom,
				})
				s.Require().NoError(err)
				s.Require().NotNil(resp.Balance)
				s.Require().Equal(testvalues.InitialBalance, resp.Balance.Amount.Int64())
			}))
		}))
	}))
}
