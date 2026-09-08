// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"encoding/hex"
	"math/big"
	"os"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/suite"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"
	ibchost "github.com/cosmos/ibc-go/v11/modules/core/24-host"
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"
	ibctmtypes "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	cosmosutils "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	ibctesting "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ibctesting"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	e2etypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

// CosmosProofAPITestSuite is a struct that holds the test suite for two Cosmos chains.
type CosmosProofAPITestSuite struct {
	e2esuite.TestSuite

	SimdA *cosmos.CosmosChain
	SimdB *cosmos.CosmosChain

	SimdASubmitter ibc.Wallet
	SimdBSubmitter ibc.Wallet

	ProofApiClient proofapitypes.ProofApiServiceClient

	// Fixture generation
	TendermintLightClientFixtures *e2etypes.TendermintLightClientFixtureGenerator
}

// TestWithIbcEurekaTestSuite is the boilerplate code that allows the test suite to be run
func TestWithCosmosProofAPITestSuite(t *testing.T) {
	suite.Run(t, new(CosmosProofAPITestSuite))
}

// SetupSuite calls the underlying IbcEurekaTestSuite's SetupSuite method
// and deploys the IbcEureka contract
func (s *CosmosProofAPITestSuite) SetupSuite(ctx context.Context) {
	chainconfig.DefaultChainSpecs = append(chainconfig.DefaultChainSpecs, chainconfig.IbcGoChainSpec("ibc-go-simd-2", "simd-2"))

	os.Setenv(testvalues.EnvKeyEthTestnetType, testvalues.EthTestnetType_None)

	// Initialize fixture generation
	s.TendermintLightClientFixtures = e2etypes.NewTendermintLightClientFixtureGenerator(&s.Suite)

	s.TestSuite.SetupSuite(ctx)

	s.SimdA, s.SimdB = s.Cosmos.Chains[0], s.Cosmos.Chains[1]
	s.SimdASubmitter = s.CreateAndFundCosmosUser(ctx, s.SimdA)
	s.SimdBSubmitter = s.CreateAndFundCosmosUser(ctx, s.SimdB)

	var proofApiProcess *os.Process
	s.Require().True(s.Run("Start Proof API", func() {
		err := os.Chdir("../..")
		s.Require().NoError(err)

		config := proofapi.NewConfigBuilder().
			CosmosToCosmos(proofapi.CosmosToCosmosParams{
				SrcChainID:    s.SimdA.Config().ChainID,
				DstChainID:    s.SimdB.Config().ChainID,
				SrcRPC:        s.SimdA.GetHostRPCAddress(),
				DstRPC:        s.SimdB.GetHostRPCAddress(),
				SignerAddress: s.SimdBSubmitter.FormattedAddress(),
			}).
			CosmosToCosmos(proofapi.CosmosToCosmosParams{
				SrcChainID:    s.SimdB.Config().ChainID,
				DstChainID:    s.SimdA.Config().ChainID,
				SrcRPC:        s.SimdB.GetHostRPCAddress(),
				DstRPC:        s.SimdA.GetHostRPCAddress(),
				SignerAddress: s.SimdASubmitter.FormattedAddress(),
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

	s.Require().True(s.Run("Create Light Client of Chain A on Chain B", func() {
		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: s.SimdA.Config().ChainID,
				DstChain: s.SimdB.Config().ChainID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, s.SimdB, s.SimdBSubmitter, 2_000_000, createClientTxBodyBz)
			clientId, err := cosmosutils.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(ibctesting.FirstClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Create Light Client of Chain B on Chain A", func() {
		var createClientTxBodyBz []byte
		s.Require().True(s.Run("Retrieve create client tx", func() {
			resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
				SrcChain: s.SimdB.Config().ChainID,
				DstChain: s.SimdA.Config().ChainID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			createClientTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, s.SimdA, s.SimdASubmitter, 2_000_000, createClientTxBodyBz)
			clientId, err := cosmosutils.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
			s.Require().NoError(err)
			s.Require().Equal(ibctesting.FirstClientID, clientId)
		}))
	}))

	s.Require().True(s.Run("Register counterparty on Chain A", func() {
		merklePathPrefix := [][]byte{[]byte(ibcexported.StoreKey), []byte("")}

		// We can do this because we know what the counterparty client ID will be
		_, err := s.BroadcastMessages(ctx, s.SimdA, s.SimdASubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 ibctesting.FirstClientID,
			CounterpartyClientId:     ibctesting.FirstClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			Signer:                   s.SimdASubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Register counterparty on Chain B", func() {
		merklePathPrefix := [][]byte{[]byte(ibcexported.StoreKey), []byte("")}

		_, err := s.BroadcastMessages(ctx, s.SimdB, s.SimdBSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
			ClientId:                 ibctesting.FirstClientID,
			CounterpartyClientId:     ibctesting.FirstClientID,
			CounterpartyMerklePrefix: merklePathPrefix,
			Signer:                   s.SimdBSubmitter.FormattedAddress(),
		})
		s.Require().NoError(err)
	}))
}

// TestRelayer is a test that runs the relayer
func (s *CosmosProofAPITestSuite) Test_ProofAPIInfo() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	s.Require().True(s.Run("Verify Chain A to Chain B Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: s.SimdA.Config().ChainID,
			DstChain: s.SimdB.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(s.SimdA.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(s.SimdB.Config().ChainID, info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Chain B to Chain A Proof API Info", func() {
		info, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: s.SimdB.Config().ChainID,
			DstChain: s.SimdA.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(s.SimdB.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(s.SimdA.Config().ChainID, info.TargetChain.ChainId)
	}))
}

func (s *CosmosProofAPITestSuite) Test_ICS20RecvAndAckPacket() {
	ctx := context.Background()
	s.FilteredICS20RecvAndAckPacketTest(ctx, 1, nil)
}

func (s *CosmosProofAPITestSuite) Test_10_ICS20RecvAndAckPacket() {
	ctx := context.Background()
	s.FilteredICS20RecvAndAckPacketTest(ctx, 10, []uint64{1, 2, 3, 4, 5, 6, 7, 8, 9, 10})
}

func (s *CosmosProofAPITestSuite) Test_10_FilteredICS20RecvAndAckPacket() {
	ctx := context.Background()
	s.FilteredICS20RecvAndAckPacketTest(ctx, 10, []uint64{1, 5, 6, 7})
}

func (s *CosmosProofAPITestSuite) FilteredICS20RecvAndAckPacketTest(ctx context.Context, numOfTransfers int, recvAndAckFilter []uint64) {
	s.Require().GreaterOrEqual(numOfTransfers, len(recvAndAckFilter))
	s.Require().Greater(numOfTransfers, 0)

	s.SetupSuite(ctx)

	simdAUser, simdBUser := s.Cosmos.Users[0], s.Cosmos.Users[1]
	transferAmount := big.NewInt(testvalues.TransferAmount)
	totalTransferAmount := testvalues.TransferAmount * int64(numOfTransfers)
	var relayedAmount int64
	if len(recvAndAckFilter) == 0 {
		relayedAmount = totalTransferAmount
	} else {
		relayedAmount = testvalues.TransferAmount * int64(len(recvAndAckFilter))
	}

	var txHashes [][]byte
	s.Require().True(s.Run("Send transfers on Chain A", func() {
		for range numOfTransfers {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			transferCoin := sdk.NewCoin(s.SimdA.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))

			transferPayload := transfertypes.FungibleTokenPacketData{
				Denom:    transferCoin.Denom,
				Amount:   transferCoin.Amount.String(),
				Sender:   simdAUser.FormattedAddress(),
				Receiver: simdBUser.FormattedAddress(),
				Memo:     "",
			}

			payload := channeltypesv2.Payload{
				SourcePort:      transfertypes.PortID,
				DestinationPort: transfertypes.PortID,
				Version:         transfertypes.V1,
				Encoding:        transfertypes.EncodingJSON,
				Value:           transferPayload.GetBytes(),
			}
			msgSendPacket := channeltypesv2.MsgSendPacket{
				SourceClient:     ibctesting.FirstClientID,
				TimeoutTimestamp: timeout,
				Payloads: []channeltypesv2.Payload{
					payload,
				},
				Signer: simdAUser.FormattedAddress(),
			}

			resp, err := s.BroadcastMessages(ctx, s.SimdA, simdAUser, 200_000, &msgSendPacket)
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.TxHash)

			txHash, err := hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(txHash)

			txHashes = append(txHashes, txHash)
		}

		s.Require().True(s.Run("Verify balances on Chain A", func() {
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, s.SimdA, &banktypes.QueryBalanceRequest{
				Address: simdAUser.FormattedAddress(),
				Denom:   s.SimdA.Config().Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance-totalTransferAmount, resp.Balance.Amount.Int64())
		}))
	}))

	var ackTxHash []byte
	s.Require().True(s.Run("Receive packets on Chain B", func() {
		var txBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           s.SimdA.Config().ChainID,
				DstChain:           s.SimdB.Config().ChainID,
				SourceTxIds:        txHashes,
				SrcClientId:        ibctesting.FirstClientID,
				DstClientId:        ibctesting.FirstClientID,
				SrcPacketSequences: recvAndAckFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			txBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, s.SimdB, s.SimdBSubmitter, 2_000_000, txBodyBz)

			var err error
			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)

			s.Require().True(s.Run("Verify balances on Chain B", func() {
				ibcDenom := transfertypes.NewDenom(s.SimdA.Config().Denom, transfertypes.NewHop(transfertypes.PortID, ibctesting.FirstClientID)).IBCDenom()
				// User balance on Cosmos chain
				resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, s.SimdB, &banktypes.QueryBalanceRequest{
					Address: simdBUser.FormattedAddress(),
					Denom:   ibcDenom,
				})
				s.Require().NoError(err)
				s.Require().NotNil(resp.Balance)
				s.Require().Equal(relayedAmount, resp.Balance.Amount.Int64())
				s.Require().Equal(ibcDenom, resp.Balance.Denom)
			}))
		}))
	}))

	s.Require().True(s.Run("Acknowledge packets on Chain A", func() {
		s.Require().True(s.Run("Verify commitments exists", func() {
			for _, seq := range recvAndAckFilter {
				resp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, s.SimdA, &channeltypesv2.QueryPacketCommitmentRequest{
					ClientId: ibctesting.FirstClientID,
					Sequence: seq,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Commitment)
			}
		}))

		var ackTxBodyBz []byte
		s.Require().True(s.Run("Retrieve ack tx to Chain A", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           s.SimdB.Config().ChainID,
				DstChain:           s.SimdA.Config().ChainID,
				SourceTxIds:        [][]byte{ackTxHash},
				SrcClientId:        ibctesting.FirstClientID,
				DstClientId:        ibctesting.FirstClientID,
				DstPacketSequences: recvAndAckFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			ackTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast ack tx on Chain A", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, s.SimdA, s.SimdASubmitter, 2_000_000, ackTxBodyBz)
		}))

		s.Require().True(s.Run("Verify commitments removed", func() {
			for _, seq := range recvAndAckFilter {
				_, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, s.SimdA, &channeltypesv2.QueryPacketCommitmentRequest{
					ClientId: ibctesting.FirstClientID,
					Sequence: seq,
				})
				s.Require().ErrorContains(err, "packet commitment hash not found")
			}
		}))
	}))
}

func (s *CosmosProofAPITestSuite) Test_ICS20TimeoutPacket() {
	ctx := context.Background()
	s.FilteredICS20TimeoutPacketTest(ctx, 1, nil)
}

func (s *CosmosProofAPITestSuite) Test_10_ICS20TimeoutPacket() {
	ctx := context.Background()
	s.FilteredICS20TimeoutPacketTest(ctx, 10, nil)
}

func (s *CosmosProofAPITestSuite) Test_10_FilteredICS20TimeoutPacket() {
	ctx := context.Background()
	s.FilteredICS20TimeoutPacketTest(ctx, 10, []uint64{1})
}

func (s *CosmosProofAPITestSuite) FilteredICS20TimeoutPacketTest(ctx context.Context, numOfTransfers int, timeoutFilter []uint64) {
	s.Require().Greater(numOfTransfers, len(timeoutFilter))

	s.SetupSuite(ctx)

	simdAUser, simdBUser := s.Cosmos.Users[0], s.Cosmos.Users[1]
	transferAmount := big.NewInt(testvalues.TransferAmount)
	totalTransferAmount := testvalues.TransferAmount * int64(numOfTransfers)
	var refundedAmount int64
	if len(timeoutFilter) == 0 {
		refundedAmount = totalTransferAmount
	} else {
		refundedAmount = testvalues.TransferAmount * int64(len(timeoutFilter))
	}

	var txHashes [][]byte
	s.Require().True(s.Run("Send transfers on Chain A", func() {
		for range numOfTransfers {
			timeout := uint64(time.Now().Add(30 * time.Second).Unix())
			transferCoin := sdk.NewCoin(s.SimdA.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))

			transferPayload := transfertypes.FungibleTokenPacketData{
				Denom:    transferCoin.Denom,
				Amount:   transferCoin.Amount.String(),
				Sender:   simdAUser.FormattedAddress(),
				Receiver: simdBUser.FormattedAddress(),
				Memo:     "",
			}

			payload := channeltypesv2.Payload{
				SourcePort:      transfertypes.PortID,
				DestinationPort: transfertypes.PortID,
				Version:         transfertypes.V1,
				Encoding:        transfertypes.EncodingJSON,
				Value:           transferPayload.GetBytes(),
			}
			msgSendPacket := channeltypesv2.MsgSendPacket{
				SourceClient:     ibctesting.FirstClientID,
				TimeoutTimestamp: timeout,
				Payloads: []channeltypesv2.Payload{
					payload,
				},
				Signer: simdAUser.FormattedAddress(),
			}

			resp, err := s.BroadcastMessages(ctx, s.SimdA, simdAUser, 200_000, &msgSendPacket)
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.TxHash)

			txHash, err := hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(txHash)

			txHashes = append(txHashes, txHash)
		}

		s.Require().True(s.Run("Verify balances on Chain A", func() {
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, s.SimdA, &banktypes.QueryBalanceRequest{
				Address: simdAUser.FormattedAddress(),
				Denom:   s.SimdA.Config().Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance-totalTransferAmount, resp.Balance.Amount.Int64())
		}))
	}))

	// Prefetching the relay tx before the timeout
	var txBodyBz []byte
	s.Require().True(s.Run("Retrieve relay tx", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    s.SimdA.Config().ChainID,
			DstChain:    s.SimdB.Config().ChainID,
			SourceTxIds: txHashes,
			SrcClientId: ibctesting.FirstClientID,
			DstClientId: ibctesting.FirstClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)
		s.Require().Empty(resp.Address)

		txBodyBz = resp.Tx
	}))

	// Wait until timeout
	time.Sleep(30 * time.Second)

	s.Require().True(s.Run("Timeout packet on Chain A", func() {
		var timeoutTxBodyBz []byte
		s.Require().True(s.Run("Retrieve timeout tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           s.SimdB.Config().ChainID,
				DstChain:           s.SimdA.Config().ChainID,
				TimeoutTxIds:       txHashes,
				SrcClientId:        ibctesting.FirstClientID,
				DstClientId:        ibctesting.FirstClientID,
				DstPacketSequences: timeoutFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			timeoutTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast timeout tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, s.SimdA, s.SimdASubmitter, 2_000_000, timeoutTxBodyBz)
		}))

		s.Require().True(s.Run("Verify balances on Chain A", func() {
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, s.SimdA, &banktypes.QueryBalanceRequest{
				Address: simdAUser.FormattedAddress(),
				Denom:   s.SimdA.Config().Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testvalues.InitialBalance-totalTransferAmount+refundedAmount, resp.Balance.Amount.Int64())
		}))
	}))

	s.Require().True(s.Run("Constructing relay packet after timeout should fail", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:     s.SimdA.Config().ChainID,
			DstChain:     s.SimdB.Config().ChainID,
			TimeoutTxIds: txHashes,
			SrcClientId:  ibctesting.FirstClientID,
			DstClientId:  ibctesting.FirstClientID,
		})
		s.Require().Error(err)
		s.Require().Nil(resp)
	}))

	s.Require().True(s.Run("Receiving packets on Chain B after timeout should fail", func() {
		resp, err := s.BroadcastSdkTxBody(ctx, s.SimdB, s.SimdBSubmitter, 2_000_000, txBodyBz)
		s.Require().ErrorContains(err, "timeout elapsed")
		s.Require().Nil(resp)
	}))
}

func (s *CosmosProofAPITestSuite) Test_UpdateClient() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var initialHeight uint64
	s.Require().True(s.Run("Get the initial height", func() {
		resp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, s.SimdA, &clienttypes.QueryClientStateRequest{
			ClientId: ibctesting.FirstClientID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.ClientState)

		var tmClientState ibctmtypes.ClientState
		err = proto.Unmarshal(resp.ClientState.Value, &tmClientState)
		s.Require().NoError(err)
		s.Require().NotZero(tmClientState.LatestHeight.RevisionHeight)

		initialHeight = tmClientState.LatestHeight.RevisionHeight
	}))

	s.Require().True(s.Run("Update client on Chain A", func() {
		var updateTxBodyBz []byte
		s.Require().True(s.Run("Retrieve update client tx", func() {
			resp, err := s.ProofApiClient.UpdateClient(context.Background(), &proofapitypes.UpdateClientRequest{
				SrcChain:    s.SimdB.Config().ChainID,
				DstChain:    s.SimdA.Config().ChainID,
				DstClientId: ibctesting.FirstClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			updateTxBodyBz = resp.Tx

			s.TendermintLightClientFixtures.GenerateUpdateClientHappyPath(ctx, s.SimdA, updateTxBodyBz)
		}))

		s.Require().True(s.Run("Broadcast update client tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, s.SimdA, s.SimdASubmitter, 2_000_000, updateTxBodyBz)
		}))

		s.Require().True(s.Run("Generate membership fixtures", func() {
			nonExistingClientID := "07-tendermint-001"

			keyPaths := []e2etypes.KeyPath{
				{Key: string(ibchost.FullClientStateKey(ibctesting.FirstClientID)), Membership: true},
				{Key: string(ibchost.FullClientStateKey(nonExistingClientID)), Membership: false},
			}

			s.TendermintLightClientFixtures.GenerateMembershipVerificationScenarios(ctx, s.SimdA, s.SimdB, keyPaths, ibctesting.FirstClientID)
		}))

		s.Require().True(s.Run("Verify client update on Chain A", func() {
			resp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, s.SimdA, &clienttypes.QueryClientStateRequest{
				ClientId: ibctesting.FirstClientID,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.ClientState)

			var tmClientState ibctmtypes.ClientState
			err = proto.Unmarshal(resp.ClientState.Value, &tmClientState)
			s.Require().NoError(err)
			s.Require().Greater(tmClientState.LatestHeight.RevisionHeight, initialHeight)
		}))
	}))
}
