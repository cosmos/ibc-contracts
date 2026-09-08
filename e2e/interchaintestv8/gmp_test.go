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

	"github.com/ethereum/go-ethereum/accounts/abi"
	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	gmptypes "github.com/cosmos/ibc-go/v11/modules/apps/27-gmp/types"
	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"

	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics27gmp"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/sp1ics07tendermint"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/erc20"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/gmphelpers"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

// IbcEurekaGmpTestSuite is a suite of tests that wraps TestSuite
// and can provide additional functionality
type IbcEurekaGmpTestSuite struct {
	e2esuite.TestSuite

	// The private key of a test account
	key *ecdsa.PrivateKey
	// The private key of the faucet account of interchaintest
	deployer *ecdsa.PrivateKey

	contractAddresses ethereum.DeployedContracts
	sp1Ics07Address   ethcommon.Address

	sp1Ics07Contract *sp1ics07tendermint.Contract
	ics26Contract    *ics26router.Contract
	ics27Contract    *ics27gmp.Contract
	erc20Contract    *erc20.Contract

	ProofApiClient proofapitypes.ProofApiServiceClient

	SimdRelayerSubmitter ibc.Wallet
	EthRelayerSubmitter  *ecdsa.PrivateKey
}

// TestWithIbcEurekaGmpTestSuite is the boilerplate code that allows the test suite to be run
func TestWithIbcEurekaGmpTestSuite(t *testing.T) {
	suite.Run(t, new(IbcEurekaGmpTestSuite))
}

func (s *IbcEurekaGmpTestSuite) SetupSuite(ctx context.Context, proofType types.SupportedProofType) {
	s.TestSuite.SetupSuite(ctx)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

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

	s.Require().True(s.Run("Deploy IBC contracts", func() {
		stdout, err := eth.ForgeScript(s.deployer, testvalues.E2EDeployScriptPath)
		s.Require().NoError(err)

		s.contractAddresses, err = ethereum.GetEthContractsFromDeployOutput(string(stdout))
		s.Require().NoError(err)
		s.ics26Contract, err = ics26router.NewContract(ethcommon.HexToAddress(s.contractAddresses.Ics26Router), eth.RPCClient)
		s.Require().NoError(err)
		s.ics27Contract, err = ics27gmp.NewContract(ethcommon.HexToAddress(s.contractAddresses.Ics27Gmp), eth.RPCClient)
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

		config := proofapi.NewConfigBuilder().
			EthToCosmos(proofapi.EthToCosmosParams{
				EthChainID:    eth.ChainID.String(),
				CosmosChainID: simd.Config().ChainID,
				TmRPC:         simd.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				BeaconAPI:     beaconAPI,
				SignerAddress: s.SimdRelayerSubmitter.FormattedAddress(),
				MockClient:    os.Getenv(testvalues.EnvKeyEthTestnetType) == testvalues.EthTestnetTypeAnvil,
			}).
			CosmosToEthSP1(proofapi.CosmosToEthSP1Params{
				EthChainID:    eth.ChainID.String(),
				CosmosChainID: simd.Config().ChainID,
				TmRPC:         simd.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				Prover: proofapi.SP1ProverConfig{
					Type:           prover,
					PrivateCluster: os.Getenv(testvalues.EnvKeyNetworkPrivateCluster) == testvalues.EnvValueSp1Prover_PrivateCluster,
				},
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

	s.Require().True(s.Run("Fund address with ERC20", func() {
		tx, err := s.erc20Contract.Transfer(s.GetTransactOpts(eth.Faucet, eth), crypto.PubkeyToAddress(s.key.PublicKey), testvalues.StartingERC20Balance)
		s.Require().NoError(err)

		_, err = eth.GetTxReciept(ctx, tx.Hash()) // wait for the tx to be mined
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Create ethereum light client on Cosmos chain", func() {
		checksumHex := s.StoreLightClient(ctx, simd, s.SimdRelayerSubmitter)
		s.Require().NotEmpty(checksumHex)

		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: eth.ChainID.String(),
				DstChain: simd.Config().ChainID,
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
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 20_000_000, createClientTxBodyBz)
			clientId, err := cosmos.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(testvalues.FirstWasmClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Add client and counterparty on EVM", func() {
		counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
			ClientId:     testvalues.FirstWasmClientID,
			MerklePrefix: [][]byte{[]byte(ibcexported.StoreKey), []byte("")},
		}
		tx, err := s.ics26Contract.AddClient(s.GetTransactOpts(s.deployer, eth), testvalues.CustomClientID, counterpartyInfo, s.sp1Ics07Address)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)

		event, err := e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseICS02ClientAdded)
		s.Require().NoError(err)
		s.Require().Equal(testvalues.CustomClientID, event.ClientId)
		s.Require().Equal(testvalues.FirstWasmClientID, event.CounterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Register counterparty on Cosmos chain", func() {
		merklePathPrefix := [][]byte{[]byte("")}

		_, err := s.BroadcastMessages(ctx, simd, s.SimdRelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 testvalues.FirstWasmClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			CounterpartyClientId:     testvalues.CustomClientID,
			Signer:                   s.SimdRelayerSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))
}

func (s *IbcEurekaGmpTestSuite) TestDeploy() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.DeployTest(ctx, proofType)
}

func (s *IbcEurekaGmpTestSuite) DeployTest(ctx context.Context, proofType types.SupportedProofType) {
	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

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

	s.Require().True(s.Run("Verify ICS02 Client", func() {
		clientAddress, err := s.ics26Contract.GetClient(nil, testvalues.CustomClientID)
		s.Require().NoError(err)
		s.Require().Equal(s.sp1Ics07Address, clientAddress)

		counterpartyInfo, err := s.ics26Contract.GetCounterparty(nil, testvalues.CustomClientID)
		s.Require().NoError(err)
		s.Require().Equal(testvalues.FirstWasmClientID, counterpartyInfo.ClientId)
	}))

	s.Require().True(s.Run("Verify ICS26 Router", func() {
		transferAddress, err := s.ics26Contract.GetIBCApp(nil, transfertypes.PortID)
		s.Require().NoError(err)
		s.Require().Equal(s.contractAddresses.Ics20Transfer, strings.ToLower(transferAddress.Hex()))
	}))

	s.Require().True(s.Run("Verify ethereum light client", func() {
		_, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simd, &clienttypes.QueryClientStateRequest{
			ClientId: testvalues.FirstWasmClientID,
		})
		s.Require().NoError(err)

		counterpartyInfoResp, err := e2esuite.GRPCQuery[clienttypesv2.QueryCounterpartyInfoResponse](ctx, simd, &clienttypesv2.QueryCounterpartyInfoRequest{
			ClientId: testvalues.FirstWasmClientID,
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

// TestSendCallFromCosmos tests the SendCall from Cosmos to Ethereum
func (s *IbcEurekaGmpTestSuite) TestSendCallFromCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.SendCallFromCosmosTest(ctx, proofType)
}

func (s *IbcEurekaGmpTestSuite) SendCallFromCosmosTest(ctx context.Context, proofType types.SupportedProofType) {
	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]
	simdUser := s.Cosmos.Users[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	testAmount := big.NewInt(1)

	testUserKey, err := eth.CreateUser()
	s.Require().NoError(err)
	testUserAddress := crypto.PubkeyToAddress(testUserKey.PublicKey)

	erc20Abi, err := abi.JSON(strings.NewReader(erc20.ContractABI))
	s.Require().NoError(err)

	var computedAddress ethcommon.Address
	s.Require().True(s.Run("Fund pre-computed ICS27 address", func() {
		computedAddress, err = s.ics27Contract.GetOrComputeAccountAddress(nil, ics27gmp.IICS27GMPMsgsAccountIdentifier{
			ClientId: testvalues.CustomClientID,
			Sender:   simdUser.FormattedAddress(),
			Salt:     nil,
		})
		s.Require().NoError(err)

		tx, err := s.erc20Contract.Transfer(s.GetTransactOpts(eth.Faucet, eth), computedAddress, testAmount)
		s.Require().NoError(err)

		_, err = eth.GetTxReciept(ctx, tx.Hash()) // wait for the tx to be mined
		s.Require().NoError(err)
	}))

	s.True(s.Run("Verify initial balances on Ethereum", func() {
		ics27AccBal, err := s.erc20Contract.BalanceOf(nil, computedAddress)
		s.Require().NoError(err)
		s.Require().Equal(testAmount, ics27AccBal)

		testBalance, err := s.erc20Contract.BalanceOf(nil, testUserAddress)
		s.Require().NoError(err)
		s.Require().Zero(testBalance.Int64())
	}))

	var sendTxHash []byte
	s.Require().True(s.Run("Send call from Cosmos", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		encodedCall, err := erc20Abi.Pack("transfer", testUserAddress, testAmount)
		s.Require().NoError(err)

		resp, err := s.BroadcastMessages(ctx, simd, simdUser, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     testvalues.FirstWasmClientID,
			Sender:           simdUser.FormattedAddress(),
			Receiver:         s.contractAddresses.Erc20,
			Salt:             nil,
			Payload:          encodedCall,
			TimeoutTimestamp: timeout,
			Memo:             "",
			Encoding:         testvalues.Ics27AbiEncoding,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		sendTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
	}))

	var ackTxHash []byte
	s.Require().True(s.Run("Receive packet in Ethereum", func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{sendTxHash},
				SrcClientId: testvalues.FirstWasmClientID,
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			recvRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 2_000_000, &ics26Address, recvRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			ackTxHash = receipt.TxHash.Bytes()
		}))

		s.True(s.Run("Verify balances on Ethereum", func() {
			// ICS27Account balance should be zero
			ics27AccBal, err := s.erc20Contract.BalanceOf(nil, computedAddress)
			s.Require().NoError(err)
			s.Require().Zero(ics27AccBal.Int64())

			testBalance, err := s.erc20Contract.BalanceOf(nil, testUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(testAmount, testBalance)
		}))
	}))

	s.Require().True(s.Run("Acknowledge packet in Cosmos", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{ackTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: testvalues.FirstWasmClientID,
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
	}))
}

// TestSendCallFromEth tests the SendCall from Ethereum to Cosmos
func (s *IbcEurekaGmpTestSuite) TestSendCallFromEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.SendCallFromEthTest(ctx, proofType)
}

func (s *IbcEurekaGmpTestSuite) SendCallFromEthTest(ctx context.Context, proofType types.SupportedProofType) {
	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]
	ethUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	testAmount := sdk.NewCoins(sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(1)))
	testSimdUser := s.CreateAndFundCosmosUserWithBalance(ctx, simd, testAmount[0].Amount.Int64())

	s.T().Logf("Eth user address: %s", ethUserAddress.Hex())

	var computedAddress sdk.AccAddress
	s.Require().True(s.Run("Fund pre-computed ICS27 address", func() {
		res, err := e2esuite.GRPCQuery[gmptypes.QueryAccountAddressResponse](ctx, simd, &gmptypes.QueryAccountAddressRequest{
			ClientId: testvalues.FirstWasmClientID,
			Sender:   ethUserAddress.String(),
			Salt:     "",
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(res.AccountAddress)

		computedAddress, err = sdk.AccAddressFromBech32(res.AccountAddress)
		s.Require().NoError(err)

		_, err = s.BroadcastMessages(ctx, simd, testSimdUser, 2_000_000, &banktypes.MsgSend{
			FromAddress: testSimdUser.FormattedAddress(),
			ToAddress:   computedAddress.String(),
			Amount:      testAmount,
		})
		s.Require().NoError(err)
	}))

	s.True(s.Run("Verify initial balances on Cosmos", func() {
		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: computedAddress.String(),
			Denom:   simd.Config().Denom,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.Balance)
		s.Require().Equal(testAmount[0].Denom, resp.Balance.Denom)
		s.Require().Equal(testAmount[0].Amount.Int64(), resp.Balance.Amount.Int64())
	}))

	var sendTxHash []byte
	s.Require().True(s.Run("Send call from Eth", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		msgSend := &banktypes.MsgSend{
			FromAddress: computedAddress.String(),
			ToAddress:   testSimdUser.FormattedAddress(),
			Amount:      testAmount,
		}

		sendCallMsg, err := gmphelpers.NewPayload_FromProto([]proto.Message{msgSend})
		s.Require().NoError(err)

		tx, err := s.ics27Contract.SendCall(s.GetTransactOpts(s.key, eth), ics27gmp.IICS27GMPMsgsSendCallMsg{
			SourceClient:     testvalues.CustomClientID,
			Receiver:         "",
			Salt:             nil,
			Payload:          sendCallMsg,
			TimeoutTimestamp: timeout,
			Memo:             "",
		})
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))
		s.Require().NotEmpty(receipt.TxHash)

		sendTxHash = receipt.TxHash.Bytes()
	}))

	var ackTxHash []byte
	s.Require().True(s.Run("Receive packet in Cosmos", func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{sendTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			recvRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, recvRelayTx)
			s.Require().Equal(uint32(0), receipt.Code, fmt.Sprintf("Tx failed: %+v", receipt))
			s.Require().NotEmpty(receipt.TxHash)

			var err error
			ackTxHash, err = hex.DecodeString(receipt.TxHash)
			s.Require().NoError(err)
		}))

		s.True(s.Run("Verify balances on Cosmos", func() {
			// ICS27Account balance should be zero
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: computedAddress.String(),
				Denom:   simd.Config().Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testAmount[0].Denom, resp.Balance.Denom)
			s.Require().Zero(resp.Balance.Amount.Int64())

			resp, err = e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: testSimdUser.FormattedAddress(),
				Denom:   simd.Config().Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testAmount[0].Denom, resp.Balance.Denom)
			s.Require().Equal(testAmount[0].Amount.Int64(), resp.Balance.Amount.Int64())
		}))
	}))

	s.Require().True(s.Run("Acknowledge packet in Eth", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{ackTxHash},
				SrcClientId: testvalues.FirstWasmClientID,
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 2_000_000, &ics26Address, relayTxBodyBz)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))
		}))
	}))
}
