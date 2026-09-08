// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"encoding/hex"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"

	gmptypes "github.com/cosmos/ibc-go/v11/modules/apps/27-gmp/types"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"

	interchaintest "github.com/cosmos/interchaintest/v11"
	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	cosmosutils "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	ibctesting "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ibctesting"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
	ifttypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/wfchain/ift"
	tokenfactorytypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/wfchain/tokenfactory"
)

// CosmosIFTTestSuite tests IFT transfers between two wfchain instances
type CosmosIFTTestSuite struct {
	e2esuite.TestSuite

	ChainA *cosmos.CosmosChain
	ChainB *cosmos.CosmosChain

	ChainASubmitter ibc.Wallet
	ChainBSubmitter ibc.Wallet

	ProofApiClient proofapitypes.ProofApiServiceClient
}

func TestWithCosmosIFTTestSuite(t *testing.T) {
	suite.Run(t, new(CosmosIFTTestSuite))
}

func (s *CosmosIFTTestSuite) SetupSuite(ctx context.Context) {
	// Use two wfchain instances
	chainconfig.DefaultChainSpecs = []*interchaintest.ChainSpec{
		chainconfig.WfchainChainSpec("wfchain-1", "wfchain-1"),
		chainconfig.WfchainChainSpec("wfchain-2", "wfchain-2"),
	}

	os.Setenv(testvalues.EnvKeyEthTestnetType, testvalues.EthTestnetType_None)
	os.Setenv(testvalues.EnvKeySolanaTestnetType, testvalues.SolanaTestnetType_None)

	s.TestSuite.SetupSuite(ctx)

	s.ChainA, s.ChainB = s.Cosmos.Chains[0], s.Cosmos.Chains[1]
	s.ChainASubmitter = s.CreateAndFundCosmosUser(ctx, s.ChainA)
	s.ChainBSubmitter = s.CreateAndFundCosmosUser(ctx, s.ChainB)

	var proofApiProcess *os.Process
	s.Require().True(s.Run("Start Proof API", func() {
		err := os.Chdir("../..")
		s.Require().NoError(err)

		config := proofapi.NewConfigBuilder().
			CosmosToCosmos(proofapi.CosmosToCosmosParams{
				SrcChainID:    s.ChainA.Config().ChainID,
				DstChainID:    s.ChainB.Config().ChainID,
				SrcRPC:        s.ChainA.GetHostRPCAddress(),
				DstRPC:        s.ChainB.GetHostRPCAddress(),
				SignerAddress: s.ChainBSubmitter.FormattedAddress(),
			}).
			CosmosToCosmos(proofapi.CosmosToCosmosParams{
				SrcChainID:    s.ChainB.Config().ChainID,
				DstChainID:    s.ChainA.Config().ChainID,
				SrcRPC:        s.ChainB.GetHostRPCAddress(),
				DstRPC:        s.ChainA.GetHostRPCAddress(),
				SignerAddress: s.ChainASubmitter.FormattedAddress(),
			}).
			Build()

		err = config.GenerateConfigFile(testvalues.ProofAPIConfigFilePath)
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
		var err error
		s.ProofApiClient, err = proofapi.GetGRPCClient(proofapi.DefaultProofAPIGRPCAddress())
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Verify Proof API Info A->B", func() {
		info, err := s.ProofApiClient.Info(ctx, &proofapitypes.InfoRequest{
			SrcChain: s.ChainA.Config().ChainID,
			DstChain: s.ChainB.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(s.ChainA.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(s.ChainB.Config().ChainID, info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Proof API Info B->A", func() {
		info, err := s.ProofApiClient.Info(ctx, &proofapitypes.InfoRequest{
			SrcChain: s.ChainB.Config().ChainID,
			DstChain: s.ChainA.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(s.ChainB.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(s.ChainA.Config().ChainID, info.TargetChain.ChainId)
	}))
}

func (s *CosmosIFTTestSuite) createLightClients(ctx context.Context) {
	s.Require().True(s.Run("Create Light Client of Chain A on Chain B", func() {
		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: s.ChainA.Config().ChainID,
				DstChain: s.ChainB.Config().ChainID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast create client tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, s.ChainB, s.ChainBSubmitter, 2_000_000, createClientTxBodyBz)
			clientId, err := cosmosutils.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(ibctesting.FirstClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Create Light Client of Chain B on Chain A", func() {
		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: s.ChainB.Config().ChainID,
				DstChain: s.ChainA.Config().ChainID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast create client tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, s.ChainA, s.ChainASubmitter, 2_000_000, createClientTxBodyBz)
			clientId, err := cosmosutils.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(ibctesting.FirstClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Register counterparty on Chain A", func() {
		merklePathPrefix := [][]byte{[]byte(ibcexported.StoreKey), []byte("")}

		_, err := s.BroadcastMessages(ctx, s.ChainA, s.ChainASubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 ibctesting.FirstClientID,
			CounterpartyClientId:     ibctesting.FirstClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			Signer:                   s.ChainASubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Register counterparty on Chain B", func() {
		merklePathPrefix := [][]byte{[]byte(ibcexported.StoreKey), []byte("")}

		_, err := s.BroadcastMessages(ctx, s.ChainB, s.ChainBSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 ibctesting.FirstClientID,
			CounterpartyClientId:     ibctesting.FirstClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			Signer:                   s.ChainBSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))
}

func (s *CosmosIFTTestSuite) Test_Deploy() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	s.Require().True(s.Run("Verify Chain A is running", func() {
		height, err := s.ChainA.Height(ctx)
		s.Require().NoError(err)
		s.Require().Greater(height, int64(0))
		s.T().Logf("Chain A height: %d", height)
	}))

	s.Require().True(s.Run("Verify Chain B is running", func() {
		height, err := s.ChainB.Height(ctx)
		s.Require().NoError(err)
		s.Require().Greater(height, int64(0))
		s.T().Logf("Chain B height: %d", height)
	}))

	s.Require().True(s.Run("Verify Proof API Info A->B", func() {
		info, err := s.ProofApiClient.Info(ctx, &proofapitypes.InfoRequest{
			SrcChain: s.ChainA.Config().ChainID,
			DstChain: s.ChainB.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
	}))

	s.Require().True(s.Run("Verify Proof API Info B->A", func() {
		info, err := s.ProofApiClient.Info(ctx, &proofapitypes.InfoRequest{
			SrcChain: s.ChainB.Config().ChainID,
			DstChain: s.ChainA.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
	}))

	s.Require().True(s.Run("Verify IFT module on Chain A", func() {
		resp, err := e2esuite.GRPCQuery[ifttypes.QueryParamsResponse](ctx, s.ChainA, &ifttypes.QueryParamsRequest{})
		s.Require().NoError(err)
		s.Require().NotNil(resp)
	}))

	s.Require().True(s.Run("Verify IFT module on Chain B", func() {
		resp, err := e2esuite.GRPCQuery[ifttypes.QueryParamsResponse](ctx, s.ChainB, &ifttypes.QueryParamsRequest{})
		s.Require().NoError(err)
		s.Require().NotNil(resp)
	}))
}

func (s *CosmosIFTTestSuite) Test_IFTTransfer() {
	ctx := context.Background()
	s.SetupSuite(ctx)
	s.createLightClients(ctx)

	userA := s.Cosmos.Users[0]
	userB := s.Cosmos.Users[1]
	transferAmount := sdkmath.NewInt(1_000_000)
	subdenom := testvalues.IFTTestDenom

	var denomA, denomB string

	s.Require().True(s.Run("Create denom on Chain A", func() {
		denomA = s.createTokenFactoryDenom(ctx, s.ChainA, s.ChainASubmitter, subdenom)
		s.T().Logf("Created denom on Chain A: %s", denomA)
	}))

	s.Require().True(s.Run("Create denom on Chain B", func() {
		denomB = s.createTokenFactoryDenom(ctx, s.ChainB, s.ChainBSubmitter, subdenom)
		s.T().Logf("Created denom on Chain B: %s", denomB)
	}))

	var iftModuleAddrA, iftModuleAddrB string
	s.Require().True(s.Run("Get IFT module addresses", func() {
		iftModuleAddrA = s.getIFTModuleAddress(ctx, s.ChainA)
		iftModuleAddrB = s.getIFTModuleAddress(ctx, s.ChainB)
		s.T().Logf("IFT module address on Chain A: %s", iftModuleAddrA)
		s.T().Logf("IFT module address on Chain B: %s", iftModuleAddrB)
	}))

	s.Require().True(s.Run("Register IFT bridge on Chain A", func() {
		s.registerIFTBridge(ctx, s.ChainA, s.ChainASubmitter, denomA, ibctesting.FirstClientID, iftModuleAddrB, testvalues.IFTSendCallConstructorCosmos)
	}))

	s.Require().True(s.Run("Register IFT bridge on Chain B", func() {
		s.registerIFTBridge(ctx, s.ChainB, s.ChainBSubmitter, denomB, ibctesting.FirstClientID, iftModuleAddrA, testvalues.IFTSendCallConstructorCosmos)
	}))

	s.Require().True(s.Run("Mint tokens to user on Chain A", func() {
		s.mintTokens(ctx, s.ChainA, s.ChainASubmitter, denomA, transferAmount, userA.FormattedAddress())
	}))

	s.Require().True(s.Run("Verify initial balance on Chain A", func() {
		balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount), "expected %s, got %s", transferAmount, balance)
		s.T().Logf("User balance on Chain A: %s", balance)
	}))

	s.Require().True(s.Run("Verify initial balance on Chain B is zero", func() {
		balance, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
		s.Require().NoError(err)
		s.Require().True(balance.IsZero(), "expected 0, got %s", balance)
	}))

	var ackTxHash []byte
	s.Require().True(s.Run("Transfer A to B", func() {
		var sendTxHash string
		s.Require().True(s.Run("Execute IFT transfer", func() {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			sendTxHash = s.iftTransfer(ctx, s.ChainA, userA, denomA, ibctesting.FirstClientID, userB.FormattedAddress(), transferAmount, timeout)
			s.Require().NotEmpty(sendTxHash)
			s.T().Logf("IFT Transfer tx hash: %s", sendTxHash)
		}))

		s.Require().True(s.Run("Verify balance burned on Chain A", func() {
			balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
			s.Require().NoError(err)
			s.Require().True(balance.IsZero(), "expected 0, got %s", balance)
		}))

		s.Require().True(s.Run("Relay packet to Chain B", func() {
			sendTxHashBytes, err := hex.DecodeString(sendTxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.ChainA.Config().ChainID,
				DstChain:    s.ChainB.Config().ChainID,
				SourceTxIds: [][]byte{sendTxHashBytes},
				SrcClientId: ibctesting.FirstClientID,
				DstClientId: ibctesting.FirstClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			broadcastResp := s.MustBroadcastSdkTxBody(ctx, s.ChainB, s.ChainBSubmitter, 2_000_000, resp.Tx)
			ackTxHash, err = hex.DecodeString(broadcastResp.TxHash)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Verify balance minted on Chain B", func() {
			balance, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
			s.Require().NoError(err)
			s.Require().True(balance.Equal(transferAmount), "expected %s, got %s", transferAmount, balance)
		}))

		s.Require().True(s.Run("Verify pending transfer exists before ack", func() {
			resp, err := s.queryPendingTransfer(ctx, s.ChainA, denomA, ibctesting.FirstClientID, 1)
			s.Require().NoError(err)
			s.Require().Equal(userA.FormattedAddress(), resp.PendingTransfer.Sender)
			s.Require().Equal(transferAmount.String(), resp.PendingTransfer.Amount.String())
			s.T().Logf("Pending transfer exists: sender=%s, amount=%s", resp.PendingTransfer.Sender, resp.PendingTransfer.Amount)
		}))

		s.Require().True(s.Run("Relay acknowledgement to Chain A", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.ChainB.Config().ChainID,
				DstChain:    s.ChainA.Config().ChainID,
				SourceTxIds: [][]byte{ackTxHash},
				SrcClientId: ibctesting.FirstClientID,
				DstClientId: ibctesting.FirstClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			_ = s.MustBroadcastSdkTxBody(ctx, s.ChainA, s.ChainASubmitter, 2_000_000, resp.Tx)
		}))

		s.Require().True(s.Run("Verify pending transfer removed after ack", func() {
			_, err := s.queryPendingTransfer(ctx, s.ChainA, denomA, ibctesting.FirstClientID, 1)
			s.Require().Error(err, "pending transfer should be removed after ack")
			s.T().Logf("Pending transfer removed as expected: %v", err)
		}))

		s.Require().True(s.Run("Verify final balances", func() {
			balanceA, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
			s.Require().NoError(err)
			balanceB, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
			s.Require().NoError(err)
			s.Require().True(balanceA.IsZero(), "userA should have 0, got %s", balanceA)
			s.Require().True(balanceB.Equal(transferAmount), "userB should have %s, got %s", transferAmount, balanceB)
			s.T().Logf("After A->B: userA=%s, userB=%s", balanceA, balanceB)
		}))
	}))

	s.Require().True(s.Run("Transfer B to A", func() {
		var sendTxHash string
		var ackTxHashB []byte

		s.Require().True(s.Run("Execute IFT transfer", func() {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			sendTxHash = s.iftTransfer(ctx, s.ChainB, userB, denomB, ibctesting.FirstClientID, userA.FormattedAddress(), transferAmount, timeout)
			s.Require().NotEmpty(sendTxHash)
			s.T().Logf("IFT Transfer tx hash: %s", sendTxHash)
		}))

		s.Require().True(s.Run("Verify balance burned on Chain B", func() {
			balance, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
			s.Require().NoError(err)
			s.Require().True(balance.IsZero(), "expected 0, got %s", balance)
		}))

		s.Require().True(s.Run("Relay packet to Chain A", func() {
			sendTxHashBytes, err := hex.DecodeString(sendTxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.ChainB.Config().ChainID,
				DstChain:    s.ChainA.Config().ChainID,
				SourceTxIds: [][]byte{sendTxHashBytes},
				SrcClientId: ibctesting.FirstClientID,
				DstClientId: ibctesting.FirstClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			broadcastResp := s.MustBroadcastSdkTxBody(ctx, s.ChainA, s.ChainASubmitter, 2_000_000, resp.Tx)
			ackTxHashB, err = hex.DecodeString(broadcastResp.TxHash)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Verify balance minted on Chain A", func() {
			balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
			s.Require().NoError(err)
			s.Require().True(balance.Equal(transferAmount), "expected %s, got %s", transferAmount, balance)
		}))

		s.Require().True(s.Run("Verify pending transfer exists before ack", func() {
			resp, err := s.queryPendingTransfer(ctx, s.ChainB, denomB, ibctesting.FirstClientID, 1)
			s.Require().NoError(err)
			s.Require().Equal(userB.FormattedAddress(), resp.PendingTransfer.Sender)
			s.Require().Equal(transferAmount.String(), resp.PendingTransfer.Amount.String())
			s.T().Logf("Pending transfer exists: sender=%s, amount=%s", resp.PendingTransfer.Sender, resp.PendingTransfer.Amount)
		}))

		s.Require().True(s.Run("Relay acknowledgement to Chain B", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.ChainA.Config().ChainID,
				DstChain:    s.ChainB.Config().ChainID,
				SourceTxIds: [][]byte{ackTxHashB},
				SrcClientId: ibctesting.FirstClientID,
				DstClientId: ibctesting.FirstClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			_ = s.MustBroadcastSdkTxBody(ctx, s.ChainB, s.ChainBSubmitter, 2_000_000, resp.Tx)
		}))

		s.Require().True(s.Run("Verify pending transfer removed after ack", func() {
			_, err := s.queryPendingTransfer(ctx, s.ChainB, denomB, ibctesting.FirstClientID, 1)
			s.Require().Error(err, "pending transfer should be removed after ack")
			s.T().Logf("Pending transfer removed as expected: %v", err)
		}))

		s.Require().True(s.Run("Verify final balances", func() {
			balanceA, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
			s.Require().NoError(err)
			balanceB, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
			s.Require().NoError(err)
			s.Require().True(balanceA.Equal(transferAmount), "userA should have %s, got %s", transferAmount, balanceA)
			s.Require().True(balanceB.IsZero(), "userB should have 0, got %s", balanceB)
			s.T().Logf("After B->A: userA=%s, userB=%s", balanceA, balanceB)
		}))
	}))
}

func (s *CosmosIFTTestSuite) Test_IFTTransferTimeout() {
	ctx := context.Background()
	s.SetupSuite(ctx)
	s.createLightClients(ctx)

	userA := s.Cosmos.Users[0]
	transferAmount := sdkmath.NewInt(1_000_000)
	subdenom := testvalues.IFTTestDenom

	var denomA, denomB string

	s.Require().True(s.Run("Create denom on Chain A", func() {
		denomA = s.createTokenFactoryDenom(ctx, s.ChainA, s.ChainASubmitter, subdenom)
		s.T().Logf("Created denom on Chain A: %s", denomA)
	}))

	s.Require().True(s.Run("Create denom on Chain B", func() {
		denomB = s.createTokenFactoryDenom(ctx, s.ChainB, s.ChainBSubmitter, subdenom)
		s.T().Logf("Created denom on Chain B: %s", denomB)
	}))

	var iftModuleAddrA, iftModuleAddrB string
	s.Require().True(s.Run("Get IFT module addresses", func() {
		iftModuleAddrA = s.getIFTModuleAddress(ctx, s.ChainA)
		iftModuleAddrB = s.getIFTModuleAddress(ctx, s.ChainB)
		s.T().Logf("IFT module address on Chain A: %s", iftModuleAddrA)
		s.T().Logf("IFT module address on Chain B: %s", iftModuleAddrB)
	}))

	s.Require().True(s.Run("Register IFT bridge on Chain A", func() {
		s.registerIFTBridge(ctx, s.ChainA, s.ChainASubmitter, denomA, ibctesting.FirstClientID, iftModuleAddrB, testvalues.IFTSendCallConstructorCosmos)
	}))

	s.Require().True(s.Run("Register IFT bridge on Chain B", func() {
		s.registerIFTBridge(ctx, s.ChainB, s.ChainBSubmitter, denomB, ibctesting.FirstClientID, iftModuleAddrA, testvalues.IFTSendCallConstructorCosmos)
	}))

	s.Require().True(s.Run("Mint tokens to user on Chain A", func() {
		s.mintTokens(ctx, s.ChainA, s.ChainASubmitter, denomA, transferAmount, userA.FormattedAddress())
	}))

	s.Require().True(s.Run("Verify initial balance on Chain A", func() {
		balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount), "expected %s, got %s", transferAmount, balance)
		s.T().Logf("User balance on Chain A: %s", balance)
	}))

	var sendTxHash string
	s.Require().True(s.Run("Send transfer with short timeout", func() {
		// Use 30 seconds to give enough time for tx confirmation and prefetch before timeout
		timeout := uint64(time.Now().Add(30 * time.Second).Unix())
		sendTxHash = s.iftTransfer(ctx, s.ChainA, userA, denomA, ibctesting.FirstClientID, userA.FormattedAddress(), transferAmount, timeout)
		s.Require().NotEmpty(sendTxHash)
		s.T().Logf("IFT Transfer tx hash: %s", sendTxHash)
	}))

	s.Require().True(s.Run("Verify balance burned on Chain A", func() {
		balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
		s.Require().NoError(err)
		s.Require().True(balance.IsZero(), "expected 0, got %s", balance)
	}))

	s.Require().True(s.Run("Verify pending transfer exists", func() {
		resp, err := s.queryPendingTransfer(ctx, s.ChainA, denomA, ibctesting.FirstClientID, 1)
		s.Require().NoError(err)
		s.Require().Equal(userA.FormattedAddress(), resp.PendingTransfer.Sender)
		s.Require().Equal(transferAmount.String(), resp.PendingTransfer.Amount.String())
		s.T().Logf("Pending transfer exists: sender=%s, amount=%s", resp.PendingTransfer.Sender, resp.PendingTransfer.Amount)
	}))

	var prefetchedRelayTx []byte
	s.Require().True(s.Run("Prefetch relay tx before timeout", func() {
		sendTxHashBytes, err := hex.DecodeString(sendTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    s.ChainA.Config().ChainID,
			DstChain:    s.ChainB.Config().ChainID,
			SourceTxIds: [][]byte{sendTxHashBytes},
			SrcClientId: ibctesting.FirstClientID,
			DstClientId: ibctesting.FirstClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)
		prefetchedRelayTx = resp.Tx
		s.T().Log("Successfully prefetched relay tx before timeout")
	}))

	s.Require().True(s.Run("Wait for timeout to expire", func() {
		s.T().Log("Waiting 35 seconds for timeout to expire...")
		time.Sleep(35 * time.Second)
	}))

	s.Require().True(s.Run("Relay timeout packet back to Chain A", func() {
		sendTxHashBytes, err := hex.DecodeString(sendTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:     s.ChainB.Config().ChainID,
			DstChain:     s.ChainA.Config().ChainID,
			TimeoutTxIds: [][]byte{sendTxHashBytes},
			SrcClientId:  ibctesting.FirstClientID,
			DstClientId:  ibctesting.FirstClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "relayer should generate timeout tx")

		_ = s.MustBroadcastSdkTxBody(ctx, s.ChainA, s.ChainASubmitter, 2_000_000, resp.Tx)
	}))

	s.Require().True(s.Run("Verify tokens refunded on Chain A", func() {
		balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount), "expected %s (refunded), got %s", transferAmount, balance)
		s.T().Logf("User balance after timeout refund: %s", balance)
	}))

	s.Require().True(s.Run("Verify pending transfer removed after timeout", func() {
		_, err := s.queryPendingTransfer(ctx, s.ChainA, denomA, ibctesting.FirstClientID, 1)
		s.Require().Error(err, "pending transfer should be removed after timeout")
		s.T().Logf("Pending transfer removed as expected: %v", err)
	}))

	s.Require().True(s.Run("Verify no balance on Chain B", func() {
		balance, err := s.ChainB.GetBalance(ctx, userA.FormattedAddress(), denomB)
		s.Require().NoError(err)
		s.Require().True(balance.IsZero(), "Chain B should have no tokens since transfer timed out, got %s", balance)
		s.T().Logf("Chain B balance is zero as expected")
	}))

	s.Require().True(s.Run("Constructing relay packet after timeout should fail", func() {
		sendTxHashBytes, err := hex.DecodeString(sendTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    s.ChainA.Config().ChainID,
			DstChain:    s.ChainB.Config().ChainID,
			SourceTxIds: [][]byte{sendTxHashBytes},
			SrcClientId: ibctesting.FirstClientID,
			DstClientId: ibctesting.FirstClientID,
		})
		s.Require().Error(err, "relayer should reject timed-out packet")
		s.Require().Nil(resp)
		s.T().Logf("Relayer correctly rejected timed-out packet: %v", err)
	}))

	s.Require().True(s.Run("Receiving packets on Chain B after timeout should fail", func() {
		resp, err := s.BroadcastSdkTxBody(ctx, s.ChainB, s.ChainBSubmitter, 2_000_000, prefetchedRelayTx)
		s.Require().Error(err, "chain should reject timed-out packet")
		s.Require().Nil(resp)
		s.T().Logf("Chain B correctly rejected timed-out packet: %v", err)
	}))
}

// Test_IFTTransferFailedReceive tests error acknowledgment handling when receive fails.
// The test intentionally skips registering the IFT bridge on Chain B while registering it
// on Chain A. When Chain A sends an IFT transfer, Chain B's IFT module fails because no
// bridge is registered for the client ID. This generates an error ack that is relayed back
// to Chain A to refund the sender.
func (s *CosmosIFTTestSuite) Test_IFTTransferFailedReceive() {
	ctx := context.Background()
	s.SetupSuite(ctx)
	s.createLightClients(ctx)

	userA := s.Cosmos.Users[0]
	userB := s.Cosmos.Users[1]
	transferAmount := sdkmath.NewInt(1_000_000)
	subdenom := testvalues.IFTTestDenom

	var denomA string

	s.Require().True(s.Run("Create denom on Chain A", func() {
		denomA = s.createTokenFactoryDenom(ctx, s.ChainA, s.ChainASubmitter, subdenom)
	}))

	// NOTE: We intentionally do NOT create denom or register bridge on Chain B
	// This will cause the receive to fail

	var iftModuleAddrB string
	s.Require().True(s.Run("Get IFT module address on Chain B", func() {
		iftModuleAddrB = s.getIFTModuleAddress(ctx, s.ChainB)
	}))

	s.Require().True(s.Run("Register IFT bridge on Chain A only", func() {
		s.registerIFTBridge(ctx, s.ChainA, s.ChainASubmitter, denomA, ibctesting.FirstClientID, iftModuleAddrB, testvalues.IFTSendCallConstructorCosmos)
	}))

	// NOTE: Intentionally NOT registering the IFT bridge on Chain B

	s.Require().True(s.Run("Mint tokens to user on Chain A", func() {
		s.mintTokens(ctx, s.ChainA, s.ChainASubmitter, denomA, transferAmount, userA.FormattedAddress())
		balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount))
	}))

	var sendTxHash string
	s.Require().True(s.Run("Send transfer to Chain B", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		sendTxHash = s.iftTransfer(ctx, s.ChainA, userA, denomA, ibctesting.FirstClientID, userB.FormattedAddress(), transferAmount, timeout)
		s.Require().NotEmpty(sendTxHash)
	}))

	s.Require().True(s.Run("Verify balance burned on Chain A", func() {
		balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
		s.Require().NoError(err)
		s.Require().True(balance.IsZero())
	}))

	s.Require().True(s.Run("Verify pending transfer exists", func() {
		resp, err := s.queryPendingTransfer(ctx, s.ChainA, denomA, ibctesting.FirstClientID, 1)
		s.Require().NoError(err)
		s.Require().Equal(userA.FormattedAddress(), resp.PendingTransfer.Sender)
		s.Require().Equal(transferAmount.String(), resp.PendingTransfer.Amount.String())
	}))

	var ackTxHash []byte
	s.Require().True(s.Run("Relay packet to Chain B (execution fails)", func() {
		sendTxHashBytes, err := hex.DecodeString(sendTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    s.ChainA.Config().ChainID,
			DstChain:    s.ChainB.Config().ChainID,
			SourceTxIds: [][]byte{sendTxHashBytes},
			SrcClientId: ibctesting.FirstClientID,
			DstClientId: ibctesting.FirstClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		broadcastResp := s.MustBroadcastSdkTxBody(ctx, s.ChainB, s.ChainBSubmitter, 2_000_000, resp.Tx)
		ackTxHash, err = hex.DecodeString(broadcastResp.TxHash)
		s.Require().NoError(err)
	}))

	// Note: The receive fails on Chain B because no IFT bridge is registered.
	// We verify the error ack refunds tokens to the sender.

	s.Require().True(s.Run("Relay error ack to Chain A", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    s.ChainB.Config().ChainID,
			DstChain:    s.ChainA.Config().ChainID,
			SourceTxIds: [][]byte{ackTxHash},
			SrcClientId: ibctesting.FirstClientID,
			DstClientId: ibctesting.FirstClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		_ = s.MustBroadcastSdkTxBody(ctx, s.ChainA, s.ChainASubmitter, 2_000_000, resp.Tx)
	}))

	s.Require().True(s.Run("Verify tokens refunded on Chain A", func() {
		balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount), "expected %s (refunded), got %s", transferAmount, balance)
	}))

	s.Require().True(s.Run("Verify pending transfer removed after error ack", func() {
		_, err := s.queryPendingTransfer(ctx, s.ChainA, denomA, ibctesting.FirstClientID, 1)
		s.Require().Error(err, "pending transfer should be removed after error ack")
	}))
}

// Test_IFTTransferMultipleSequential tests that multiple transfers can be sent before
// relaying acks, verifying that sequence numbers are tracked correctly. Each transfer
// gets a unique sequence number and can be acknowledged independently.
func (s *CosmosIFTTestSuite) Test_IFTTransferMultipleSequential() {
	ctx := context.Background()
	s.SetupSuite(ctx)
	s.createLightClients(ctx)

	userA := s.Cosmos.Users[0]
	userB := s.Cosmos.Users[1]
	transferAmount := sdkmath.NewInt(1_000_000)
	totalAmount := transferAmount.MulRaw(3)
	subdenom := testvalues.IFTTestDenom

	var denomA, denomB string

	s.Require().True(s.Run("Create denom on Chain A", func() {
		denomA = s.createTokenFactoryDenom(ctx, s.ChainA, s.ChainASubmitter, subdenom)
	}))

	s.Require().True(s.Run("Create denom on Chain B", func() {
		denomB = s.createTokenFactoryDenom(ctx, s.ChainB, s.ChainBSubmitter, subdenom)
	}))

	var iftModuleAddrA, iftModuleAddrB string
	s.Require().True(s.Run("Get IFT module addresses", func() {
		iftModuleAddrA = s.getIFTModuleAddress(ctx, s.ChainA)
		iftModuleAddrB = s.getIFTModuleAddress(ctx, s.ChainB)
	}))

	s.Require().True(s.Run("Register IFT bridges", func() {
		s.registerIFTBridge(ctx, s.ChainA, s.ChainASubmitter, denomA, ibctesting.FirstClientID, iftModuleAddrB, testvalues.IFTSendCallConstructorCosmos)
		s.registerIFTBridge(ctx, s.ChainB, s.ChainBSubmitter, denomB, ibctesting.FirstClientID, iftModuleAddrA, testvalues.IFTSendCallConstructorCosmos)
	}))

	s.Require().True(s.Run("Mint tokens to user on Chain A", func() {
		s.mintTokens(ctx, s.ChainA, s.ChainASubmitter, denomA, totalAmount, userA.FormattedAddress())
		balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(totalAmount))
	}))

	var sendTxHashes []string
	var ackTxHashes [][]byte
	s.Require().True(s.Run("Transfer A to B", func() {
		sendTxHashes = make([]string, 3)
		s.Require().True(s.Run("Send 3 transfers without relaying", func() {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

			for i := 0; i < 3; i++ {
				s.Require().True(s.Run(fmt.Sprintf("Send transfer %d", i+1), func() {
					sendTxHashes[i] = s.iftTransfer(ctx, s.ChainA, userA, denomA, ibctesting.FirstClientID, userB.FormattedAddress(), transferAmount, timeout)
					s.Require().NotEmpty(sendTxHashes[i])
				}))
			}
		}))

		s.Require().True(s.Run("Verify all tokens burned on Chain A", func() {
			balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
			s.Require().NoError(err)
			s.Require().True(balance.IsZero(), "expected 0, got %s", balance)
		}))

		s.Require().True(s.Run("Verify 3 pending transfers exist with correct sequences", func() {
			for seq := uint64(1); seq <= 3; seq++ {
				s.Require().True(s.Run(fmt.Sprintf("Verify pending transfer seq=%d", seq), func() {
					resp, err := s.queryPendingTransfer(ctx, s.ChainA, denomA, ibctesting.FirstClientID, seq)
					s.Require().NoError(err)
					s.Require().Equal(userA.FormattedAddress(), resp.PendingTransfer.Sender)
					s.Require().Equal(transferAmount.String(), resp.PendingTransfer.Amount.String())
				}))
			}
		}))

		ackTxHashes = make([][]byte, 3)
		s.Require().True(s.Run("Relay all packets to Chain B", func() {
			for i := 0; i < 3; i++ {
				s.Require().True(s.Run(fmt.Sprintf("Relay packet %d", i+1), func() {
					sendTxHashBytes, err := hex.DecodeString(sendTxHashes[i])
					s.Require().NoError(err)

					resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
						SrcChain:    s.ChainA.Config().ChainID,
						DstChain:    s.ChainB.Config().ChainID,
						SourceTxIds: [][]byte{sendTxHashBytes},
						SrcClientId: ibctesting.FirstClientID,
						DstClientId: ibctesting.FirstClientID,
					})
					s.Require().NoError(err)
					s.Require().NotEmpty(resp.Tx)

					broadcastResp := s.MustBroadcastSdkTxBody(ctx, s.ChainB, s.ChainBSubmitter, 2_000_000, resp.Tx)
					ackTxHashes[i], err = hex.DecodeString(broadcastResp.TxHash)
					s.Require().NoError(err)
				}))
			}
		}))

		s.Require().True(s.Run("Verify total balance minted on Chain B", func() {
			balance, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
			s.Require().NoError(err)
			s.Require().True(balance.Equal(totalAmount), "expected %s, got %s", totalAmount, balance)
		}))

		s.Require().True(s.Run("Relay all acks to Chain A", func() {
			for i := 0; i < 3; i++ {
				s.Require().True(s.Run(fmt.Sprintf("Relay ack %d", i+1), func() {
					resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
						SrcChain:    s.ChainB.Config().ChainID,
						DstChain:    s.ChainA.Config().ChainID,
						SourceTxIds: [][]byte{ackTxHashes[i]},
						SrcClientId: ibctesting.FirstClientID,
						DstClientId: ibctesting.FirstClientID,
					})
					s.Require().NoError(err)
					s.Require().NotEmpty(resp.Tx)

					_ = s.MustBroadcastSdkTxBody(ctx, s.ChainA, s.ChainASubmitter, 2_000_000, resp.Tx)
				}))
			}
		}))

		s.Require().True(s.Run("Verify all pending transfers cleared", func() {
			for seq := uint64(1); seq <= 3; seq++ {
				s.Require().True(s.Run(fmt.Sprintf("Verify pending transfer seq=%d cleared", seq), func() {
					_, err := s.queryPendingTransfer(ctx, s.ChainA, denomA, ibctesting.FirstClientID, seq)
					s.Require().Error(err, "pending transfer seq=%d should be cleared", seq)
				}))
			}
		}))

		s.Require().True(s.Run("Verify balances after A to B transfers", func() {
			balanceA, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
			s.Require().NoError(err)
			balanceB, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
			s.Require().NoError(err)
			s.Require().True(balanceA.IsZero(), "Chain A user should have 0")
			s.Require().True(balanceB.Equal(totalAmount), "Chain B user should have %s", totalAmount)
		}))
	}))

	var sendTxHashesBA []string
	var ackTxHashesBA [][]byte
	s.Require().True(s.Run("Transfer B to A", func() {
		sendTxHashesBA = make([]string, 3)
		s.Require().True(s.Run("Send 3 transfers without relaying", func() {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

			for i := 0; i < 3; i++ {
				s.Require().True(s.Run(fmt.Sprintf("Send transfer %d", i+1), func() {
					sendTxHashesBA[i] = s.iftTransfer(ctx, s.ChainB, userB, denomB, ibctesting.FirstClientID, userA.FormattedAddress(), transferAmount, timeout)
					s.Require().NotEmpty(sendTxHashesBA[i])
				}))
			}
		}))

		s.Require().True(s.Run("Verify all tokens burned on Chain B", func() {
			balance, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
			s.Require().NoError(err)
			s.Require().True(balance.IsZero(), "expected 0, got %s", balance)
		}))

		s.Require().True(s.Run("Verify 3 pending transfers exist on Chain B", func() {
			for seq := uint64(1); seq <= 3; seq++ {
				s.Require().True(s.Run(fmt.Sprintf("Verify pending transfer seq=%d", seq), func() {
					resp, err := s.queryPendingTransfer(ctx, s.ChainB, denomB, ibctesting.FirstClientID, seq)
					s.Require().NoError(err)
					s.Require().Equal(userB.FormattedAddress(), resp.PendingTransfer.Sender)
					s.Require().Equal(transferAmount.String(), resp.PendingTransfer.Amount.String())
				}))
			}
		}))

		ackTxHashesBA = make([][]byte, 3)
		s.Require().True(s.Run("Relay all packets to Chain A", func() {
			for i := 0; i < 3; i++ {
				s.Require().True(s.Run(fmt.Sprintf("Relay packet %d", i+1), func() {
					sendTxHashBytes, err := hex.DecodeString(sendTxHashesBA[i])
					s.Require().NoError(err)

					resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
						SrcChain:    s.ChainB.Config().ChainID,
						DstChain:    s.ChainA.Config().ChainID,
						SourceTxIds: [][]byte{sendTxHashBytes},
						SrcClientId: ibctesting.FirstClientID,
						DstClientId: ibctesting.FirstClientID,
					})
					s.Require().NoError(err)
					s.Require().NotEmpty(resp.Tx)

					broadcastResp := s.MustBroadcastSdkTxBody(ctx, s.ChainA, s.ChainASubmitter, 2_000_000, resp.Tx)
					ackTxHashesBA[i], err = hex.DecodeString(broadcastResp.TxHash)
					s.Require().NoError(err)
				}))
			}
		}))

		s.Require().True(s.Run("Verify total balance minted on Chain A", func() {
			balance, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
			s.Require().NoError(err)
			s.Require().True(balance.Equal(totalAmount), "expected %s, got %s", totalAmount, balance)
		}))

		s.Require().True(s.Run("Relay all acks to Chain B", func() {
			for i := 0; i < 3; i++ {
				s.Require().True(s.Run(fmt.Sprintf("Relay ack %d", i+1), func() {
					resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
						SrcChain:    s.ChainA.Config().ChainID,
						DstChain:    s.ChainB.Config().ChainID,
						SourceTxIds: [][]byte{ackTxHashesBA[i]},
						SrcClientId: ibctesting.FirstClientID,
						DstClientId: ibctesting.FirstClientID,
					})
					s.Require().NoError(err)
					s.Require().NotEmpty(resp.Tx)

					_ = s.MustBroadcastSdkTxBody(ctx, s.ChainB, s.ChainBSubmitter, 2_000_000, resp.Tx)
				}))
			}
		}))

		s.Require().True(s.Run("Verify all pending transfers cleared on Chain B", func() {
			for seq := uint64(1); seq <= 3; seq++ {
				s.Require().True(s.Run(fmt.Sprintf("Verify pending transfer seq=%d cleared", seq), func() {
					_, err := s.queryPendingTransfer(ctx, s.ChainB, denomB, ibctesting.FirstClientID, seq)
					s.Require().Error(err, "pending transfer seq=%d should be cleared", seq)
				}))
			}
		}))

		s.Require().True(s.Run("Verify final balances", func() {
			balanceA, err := s.ChainA.GetBalance(ctx, userA.FormattedAddress(), denomA)
			s.Require().NoError(err)
			balanceB, err := s.ChainB.GetBalance(ctx, userB.FormattedAddress(), denomB)
			s.Require().NoError(err)
			s.Require().True(balanceA.Equal(totalAmount), "Chain A user should have %s", totalAmount)
			s.Require().True(balanceB.IsZero(), "Chain B user should have 0")
		}))
	}))
}

// Test_GMPPacketNotBlockedByIFT verifies that IFT's callback handler doesn't interfere
// with other GMP applications. IFT is registered as the ContractKeeper for all GMP callbacks,
// so it receives callbacks for all GMP packets. It must gracefully ignore non-IFT packets.
func (s *CosmosIFTTestSuite) Test_GMPPacketNotBlockedByIFT() {
	ctx := context.Background()
	s.SetupSuite(ctx)
	s.createLightClients(ctx)

	user := s.Cosmos.Users[0]

	// Send a GMP packet directly from a user (not through IFT module).
	// This simulates another application using GMP on the same chain.
	var sendTxHash string
	s.Require().True(s.Run("User sends GMP packet directly", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		// Send to a non-existent receiver - this will cause an error ack,
		// which is fine for this test. We just want to verify the ack callback
		// doesn't break when IFT receives it.
		resp, err := s.BroadcastMessages(ctx, s.ChainA, user, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     ibctesting.FirstClientID,
			Sender:           user.FormattedAddress(),
			Receiver:         "nonexistent-receiver-address",
			Payload:          []byte("test payload"),
			TimeoutTimestamp: timeout,
		})
		s.Require().NoError(err)
		sendTxHash = resp.TxHash
		s.Require().NotEmpty(sendTxHash)
	}))

	var recvTxHash []byte
	s.Require().True(s.Run("Relay packet to Chain B", func() {
		sendTxHashBytes, err := hex.DecodeString(sendTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    s.ChainA.Config().ChainID,
			DstChain:    s.ChainB.Config().ChainID,
			SourceTxIds: [][]byte{sendTxHashBytes},
			SrcClientId: ibctesting.FirstClientID,
			DstClientId: ibctesting.FirstClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		// Broadcast to Chain B - this will likely produce an error ack
		// since the receiver doesn't exist, but that's expected
		broadcastResp := s.MustBroadcastSdkTxBody(ctx, s.ChainB, s.ChainBSubmitter, 2_000_000, resp.Tx)
		recvTxHash, err = hex.DecodeString(broadcastResp.TxHash)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Relay ack back to Chain A", func() {
		// This is the critical part: when the ack is relayed back,
		// IFT's IBCOnAcknowledgementPacketCallback will be invoked
		// (since IFT is the ContractKeeper for GMP).
		// IFT should gracefully ignore this packet (return nil, not error)
		// because packetSender != IFT module address.
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    s.ChainB.Config().ChainID,
			DstChain:    s.ChainA.Config().ChainID,
			SourceTxIds: [][]byte{recvTxHash},
			SrcClientId: ibctesting.FirstClientID,
			DstClientId: ibctesting.FirstClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		// If IFT returned an error for non-IFT packets, this broadcast would fail.
		// The fact that it succeeds proves IFT gracefully ignores non-IFT packets.
		_ = s.MustBroadcastSdkTxBody(ctx, s.ChainA, s.ChainASubmitter, 2_000_000, resp.Tx)
	}))
}

// Helper functions

func (s *CosmosIFTTestSuite) createTokenFactoryDenom(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, denom string) string {
	msg := &tokenfactorytypes.MsgCreateDenom{
		Sender: user.FormattedAddress(),
		Denom:  denom,
	}

	_, err := s.BroadcastMessages(ctx, chain, user, 200_000, msg)
	s.Require().NoError(err)

	return denom
}

func (s *CosmosIFTTestSuite) mintTokens(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, denom string, amount sdkmath.Int, recipient string) {
	msg := &tokenfactorytypes.MsgMint{
		From:    user.FormattedAddress(),
		Address: recipient,
		Amount:  sdk.Coin{Denom: denom, Amount: amount},
	}

	_, err := s.BroadcastMessages(ctx, chain, user, 200_000, msg)
	s.Require().NoError(err)
}

func (s *CosmosIFTTestSuite) registerIFTBridge(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, denom, clientId, counterpartyIftAddr, constructor string) {
	govModuleAddr, err := chain.AuthQueryModuleAddress(ctx, govtypes.ModuleName)
	s.Require().NoError(err)

	msg := &ifttypes.MsgRegisterIFTBridge{
		Signer:                 govModuleAddr,
		Denom:                  denom,
		ClientId:               clientId,
		CounterpartyIftAddress: counterpartyIftAddr,
		IftSendCallConstructor: constructor,
	}

	err = s.ExecuteGovV1Proposal(ctx, msg, chain, user)
	s.Require().NoError(err)
}

func (s *CosmosIFTTestSuite) iftTransfer(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, denom, clientId, receiver string, amount sdkmath.Int, timeoutTimestamp uint64) string {
	msg := &ifttypes.MsgIFTTransfer{
		Signer:           user.FormattedAddress(),
		Denom:            denom,
		ClientId:         clientId,
		Receiver:         receiver,
		Amount:           amount,
		TimeoutTimestamp: timeoutTimestamp,
	}

	resp, err := s.BroadcastMessages(ctx, chain, user, 200_000, msg)
	s.Require().NoError(err)

	return resp.TxHash
}

func (s *CosmosIFTTestSuite) queryPendingTransfer(ctx context.Context, chain *cosmos.CosmosChain, denom, clientID string, sequence uint64) (*ifttypes.QueryPendingTransferResponse, error) {
	return e2esuite.GRPCQuery[ifttypes.QueryPendingTransferResponse](ctx, chain, &ifttypes.QueryPendingTransferRequest{
		Denom:    denom,
		ClientId: clientID,
		Sequence: sequence,
	})
}

func (s *CosmosIFTTestSuite) getIFTModuleAddress(ctx context.Context, chain *cosmos.CosmosChain) string {
	iftAddr := authtypes.NewModuleAddress(testvalues.IFTModuleName)
	bech32Addr, err := sdk.Bech32ifyAddressBytes(chain.Config().Bech32Prefix, iftAddr)
	s.Require().NoError(err)

	return bech32Addr
}
