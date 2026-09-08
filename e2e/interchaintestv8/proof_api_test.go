// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"encoding/hex"
	"fmt"
	"math/big"
	"os"
	"slices"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/suite"
	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi"
	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/rpc"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	ibcwasmtypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/v11/types"
	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"
	ibchostv2 "github.com/cosmos/ibc-go/v11/modules/core/24-host/v2"

	"github.com/cosmos/interchaintest/v11/testutil"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ibcerc20"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics20transfer"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	ethereumtypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/ethereum"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

// ProofAPITestSuite is a suite of tests that wraps IbcEurekaTestSuite
// and can provide additional functionality
type ProofAPITestSuite struct {
	IbcEurekaTestSuite
}

// TestWithIbcEurekaTestSuite is the boilerplate code that allows the test suite to be run
func TestWithProofAPITestSuite(t *testing.T) {
	suite.Run(t, new(ProofAPITestSuite))
}

func (s *ProofAPITestSuite) Test_10_RecvPacketToEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredRecvPacketToEthTest(ctx, proofType, 10, nil)
}

func (s *ProofAPITestSuite) Test_5_RecvPacketToEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredRecvPacketToEthTest(ctx, proofType, 5, []uint64{1, 2, 3, 4, 5})
}

func (s *ProofAPITestSuite) Test_10_FilteredRecvPacketToEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.FilteredRecvPacketToEthTest(ctx, proofType, 10, []uint64{2, 6})
}

func (s *ProofAPITestSuite) FilteredRecvPacketToEthTest(
	ctx context.Context, proofType types.SupportedProofType, numOfTransfers int, recvFilter []uint64,
) {
	s.Require().GreaterOrEqual(numOfTransfers, len(recvFilter))
	s.Require().Greater(numOfTransfers, 0)

	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	transferAmount := big.NewInt(testvalues.TransferAmount)
	totalTransferAmount := big.NewInt(testvalues.TransferAmount * int64(numOfTransfers))
	if totalTransferAmount.Int64() > testvalues.InitialBalance {
		s.FailNow("Total transfer amount exceeds the initial balance")
	}
	var totalRecvAmount *big.Int
	if len(recvFilter) == 0 {
		totalRecvAmount = totalTransferAmount
	} else {
		totalRecvAmount = big.NewInt(testvalues.TransferAmount * int64(len(recvFilter)))
	}
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	var (
		transferCoin sdk.Coin
		sendTxHashes [][]byte
	)
	s.Require().True(s.Run("Send transfers on Cosmos chain", func() {
		for range numOfTransfers {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			transferCoin = sdk.NewCoin(simd.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))

			transferPayload := transfertypes.FungibleTokenPacketData{
				Denom:    transferCoin.Denom,
				Amount:   transferCoin.Amount.String(),
				Sender:   cosmosUserAddress,
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

	s.Require().True(s.Run("Receive packets on Ethereum", func() {
		var relayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           simd.Config().ChainID,
				DstChain:           eth.ChainID.String(),
				SourceTxIds:        sendTxHashes,
				SrcClientId:        testvalues.FirstWasmClientID,
				DstClientId:        testvalues.CustomClientID,
				SrcPacketSequences: recvFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			relayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, relayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))
		}))

		s.Require().True(s.Run("Verify balances on Ethereum", func() {
			denomOnEthereum := transfertypes.NewDenom(transferCoin.Denom, transfertypes.NewHop(transfertypes.PortID, testvalues.CustomClientID))

			ibcERC20Addr, err := s.ics20Contract.IbcERC20Contract(nil, denomOnEthereum.Path())
			s.Require().NoError(err)

			ibcERC20, err := ibcerc20.NewContract(ethcommon.HexToAddress(ibcERC20Addr.Hex()), s.Eth.Chains[0].RPCClient)
			s.Require().NoError(err)

			// User balance on Ethereum
			userBalance, err := ibcERC20.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(totalRecvAmount, userBalance)
		}))
	}))
}

// TestConcurrentRecvPacketToEth tests the concurrent relaying of 2 packets from Cosmos to Ethereum
// NOTE: This test is not included in the CI pipeline as it is flaky
func (s *ProofAPITestSuite) Test_2_ConcurrentRecvPacketToEth() {
	// I've noticed that the prover network drops the requests when sending too many
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ConcurrentRecvPacketToEthTest(ctx, proofType, 2)
}

func (s *ProofAPITestSuite) ConcurrentRecvPacketToEthTest(
	ctx context.Context, proofType types.SupportedProofType, numConcurrentTransfers int,
) {
	s.Require().Greater(numConcurrentTransfers, 0)

	s.SetupSuite(ctx, proofType)

	_, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	transferAmount := big.NewInt(testvalues.TransferAmount)
	transferCoin := sdk.NewCoin(simd.Config().Denom, sdkmath.NewIntFromBigInt(transferAmount))
	totalTransferAmount := big.NewInt(testvalues.TransferAmount * int64(numConcurrentTransfers))
	if totalTransferAmount.Int64() > testvalues.InitialBalance {
		s.FailNow("Total transfer amount exceeds the initial balance")
	}
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	var sendTxHashes [][]byte
	s.Require().True(s.Run("Send transfers on Cosmos chain", func() {
		for range numConcurrentTransfers {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

			transferPayload := transfertypes.FungibleTokenPacketData{
				Denom:    transferCoin.Denom,
				Amount:   transferCoin.Amount.String(),
				Sender:   cosmosUserAddress,
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

	s.Require().True(s.Run("Install circuit artifacts on machine", func() {
		// When running multiple proof API instances, the circuit artifacts need to be installed on the machine
		// to avoid the overhead of installing the artifacts for each proof API instance (which also panics).
		// This is why we make a single request which installs the artifacts on the machine, and discard the response.

		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    s.Eth.Chains[0].ChainID.String(),
			SourceTxIds: sendTxHashes,
			SrcClientId: testvalues.FirstWasmClientID,
			DstClientId: testvalues.CustomClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)
		s.Require().Equal(resp.Address, ics26Address.String())
	}))

	var wg sync.WaitGroup
	wg.Add(numConcurrentTransfers)
	s.Require().True(s.Run("Make concurrent requests", func() {
		// loop over the txHashes and send them concurrently
		for _, txHash := range sendTxHashes {
			// we send the request while the previous request is still being processed
			time.Sleep(3 * time.Second)
			go func() {
				defer wg.Done() // decrement the counter when the request completes
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    simd.Config().ChainID,
					DstChain:    s.Eth.Chains[0].ChainID.String(),
					SourceTxIds: [][]byte{txHash},
					SrcClientId: testvalues.FirstWasmClientID,
					DstClientId: testvalues.CustomClientID,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				s.Require().Equal(resp.Address, ics26Address.String())
			}()
		}
	}))

	s.Require().True(s.Run("Wait for all requests to complete", func() {
		// wait for all requests to complete
		// If the request never completes, we rely on the test timeout to kill the test
		wg.Wait()
	}))
}

func (s *ProofAPITestSuite) Test_10_BatchedAckPacketToEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenBatchedAckToEthTest(ctx, proofType, 10, big.NewInt(testvalues.TransferAmount), nil)
}

func (s *ProofAPITestSuite) Test_5_BatchedAckPacketToEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenBatchedAckToEthTest(ctx, proofType, 5, big.NewInt(testvalues.TransferAmount), []uint64{1, 2, 3, 4, 5})
}

func (s *ProofAPITestSuite) Test_10_FilteredBatchedAckPacketToEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenBatchedAckToEthTest(ctx, proofType, 10, big.NewInt(testvalues.TransferAmount), []uint64{2, 6})
}

// Note that the relayer still only relays one tx, the batching is done
// on the cosmos transaction itself. So that it emits multiple IBC events.
func (s *ProofAPITestSuite) ICS20TransferERC20TokenBatchedAckToEthTest(
	ctx context.Context, proofType types.SupportedProofType, numOfTransfers int, transferAmount *big.Int, ackFilter []uint64,
) {
	s.Require().GreaterOrEqual(numOfTransfers, len(ackFilter))
	s.Require().Greater(numOfTransfers, 0)

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
		sendTxHashes  [][]byte
		escrowAddress ethcommon.Address
	)
	s.Require().True(s.Run(fmt.Sprintf("Send %d transfers on Ethereum", numOfTransfers), func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		transferMulticall := make([][]byte, numOfTransfers)

		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			SourceClient:     testvalues.CustomClientID,
			Denom:            erc20Address,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
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
		sendTxHashes = append(sendTxHashes, tx.Hash().Bytes())

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

	var ackTxHash []byte
	s.Require().True(s.Run("Receive packets on Cosmos chain", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: sendTxHashes,
				SrcClientId: testvalues.CustomClientID,
				DstClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 5_000_000, relayTxBodyBz)

			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))

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
		s.Require().True(s.Run("Verify commitment exists", func() {
			for i := range numOfTransfers {
				seq := uint64(i) + 1
				packetCommitmentPath := ibchostv2.PacketCommitmentKey(testvalues.CustomClientID, seq)
				var ethPath [32]byte
				copy(ethPath[:], crypto.Keccak256(packetCommitmentPath))

				resp, err := s.ics26Contract.GetCommitment(nil, ethPath)
				s.Require().NoError(err)
				s.Require().NotZero(resp)
			}
		}))

		var relayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           simd.Config().ChainID,
				DstChain:           s.Eth.Chains[0].ChainID.String(),
				SourceTxIds:        [][]byte{ackTxHash},
				SrcClientId:        testvalues.FirstWasmClientID,
				DstClientId:        testvalues.CustomClientID,
				DstPacketSequences: ackFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			relayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, relayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

			// Verify the ack packet event exists
			_, err = e2esuite.GetEvmEvent(receipt, s.ics26Contract.ParseAckPacket)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Verify commitment removed", func() {
			for _, seq := range ackFilter {
				packetCommitmentPath := ibchostv2.PacketCommitmentKey(testvalues.CustomClientID, seq)
				var ethPath [32]byte
				copy(ethPath[:], crypto.Keccak256(packetCommitmentPath))

				resp, err := s.ics26Contract.GetCommitment(nil, ethPath)
				s.Require().NoError(err)
				s.Require().Zero(resp)
			}
		}))
	}))
}

func (s *ProofAPITestSuite) Test_MultiPeriodClientUpdateToCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()

	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	transferAmount := big.NewInt(testvalues.TransferAmount)

	s.Require().True(s.Run("Approve the ICS20Transfer.sol contract to spend the erc20 tokens", func() {
		tx, err := s.erc20Contract.Approve(s.GetTransactOpts(s.key, eth), ics20Address, transferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	s.Require().True(s.Run("Wait for period to change twice on Eth before relaying", func() {
		_, ethClientState := s.GetEthereumClientState(ctx, simd, testvalues.FirstWasmClientID)
		clientPeriod := ethClientState.LatestSlot / (ethClientState.EpochsPerSyncCommitteePeriod * ethClientState.SlotsPerEpoch)
		err := testutil.WaitForCondition(time.Minute*30, time.Second*30, func() (bool, error) {
			finalityUpdate, err := eth.BeaconAPIClient.GetFinalityUpdate()
			if err != nil {
				return false, err
			}

			finalitySlot, err := strconv.Atoi(finalityUpdate.Data.FinalizedHeader.Beacon.Slot)
			if err != nil {
				return false, err
			}
			finalityPeriod := uint64(finalitySlot) / (ethClientState.EpochsPerSyncCommitteePeriod * ethClientState.SlotsPerEpoch)

			return finalityPeriod == clientPeriod+2, nil
		})
		s.Require().NoError(err)
	}))

	var (
		sendTxHash    []byte
		escrowAddress ethcommon.Address
	)
	s.Require().True(s.Run("Send transfers on Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		msgSendTransfer := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			SourceClient:     testvalues.CustomClientID,
			DestPort:         transfertypes.PortID,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			Memo:             "",
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendTransfer)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		sendTxHash = tx.Hash().Bytes()

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

	s.Require().True(s.Run("Receive packets on Cosmos chain", func() {
		var relayTxBodyBz []byte
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

			relayTxBodyBz = resp.Tx

			s.wasmFixtureGenerator.AddFixtureStep("receive_packets", ethereumtypes.RelayerMessages{
				RelayerTxBody: hex.EncodeToString(relayTxBodyBz),
			})
		}))

		var ackTxHash []byte
		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, relayTxBodyBz)

			var err error
			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))

			// User balance on Cosmos chain
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(transferAmount, resp.Balance.Amount.BigInt())
			s.Require().Equal(denomOnCosmos.IBCDenom(), resp.Balance.Denom)
		}))
	}))
}

func (s *IbcEurekaTestSuite) Test_5_FinalizedTimeoutPacketFromEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20FinalizedTimeoutPacketFromEthTest(ctx, proofType, 5)
}

func (s *IbcEurekaTestSuite) ICS20FinalizedTimeoutPacketFromEthTest(
	ctx context.Context, pt types.SupportedProofType, numOfTransfers int,
) {
	s.Require().Greater(numOfTransfers, 0)

	s.SetupSuite(ctx, pt)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	transferAmount := big.NewInt(testvalues.TransferAmount)
	totalTransferAmount := new(big.Int).Mul(transferAmount, big.NewInt(int64(numOfTransfers)))
	refundedAmount := totalTransferAmount
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	ics20transferAbi, err := abi.JSON(strings.NewReader(ics20transfer.ContractABI))
	s.Require().NoError(err)

	var originalBalance *sdk.Coin
	s.Require().True(s.Run("Retrieve original balance", func() {
		denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))

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
		ethSendTxHash        []byte
		escrowAddress        ethcommon.Address
		ethSendTxBlockNumber *big.Int
	)
	s.Require().True(s.Run(fmt.Sprintf("Send %d transfers on Ethereum", numOfTransfers), func() {
		timeout := uint64(time.Now().Add(30 * time.Second).Unix())
		transferMulticall := make([][]byte, numOfTransfers)

		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			SourceClient:     testvalues.CustomClientID,
			Memo:             "testmemo",
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

		ethSendTxHash = tx.Hash().Bytes()
		ethSendTxBlockNumber = receipt.BlockNumber

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

	// sleep for 45 seconds to let the packet timeout
	time.Sleep(45 * time.Second)

	s.True(s.Run("Wait for timeout tx to be finalized", func() {
		err = testutil.WaitForCondition(time.Minute*30, time.Second*30, func() (bool, error) {
			finalizedBlock, err := eth.RPCClient.BlockByNumber(ctx, big.NewInt(int64(rpc.FinalizedBlockNumber)))
			s.Require().NoError(err)

			// Check if the block number is greater than or equal to the send tx block number
			return finalizedBlock.Number().Int64() >= ethSendTxBlockNumber.Int64(), nil
		})
	}))

	s.True(s.Run("Timeout packets on Ethereum", func() {
		reqStartTime := time.Now()

		var timeoutRelayTx []byte
		s.Require().True(s.Run("Retrieve timeout tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:     simd.Config().ChainID,
				DstChain:     eth.ChainID.String(),
				TimeoutTxIds: [][]byte{ethSendTxHash},
				SrcClientId:  testvalues.FirstWasmClientID,
				DstClientId:  testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			timeoutRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Verify time constraints", func() {
			elapsed := time.Since(reqStartTime)
			s.Require().LessOrEqual(elapsed, 90*time.Second) // Up to 90 seconds to generate the sp1 proof
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, timeoutRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		}))

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
			denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))

			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().Equal(originalBalance, resp.Balance)
		}))
	}))
}

func (s *ProofAPITestSuite) Test_10_RecvPacketToCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.SetupSuite(ctx, proofType)
	s.FilteredRecvPacketToCosmosTest(ctx, 10, big.NewInt(testvalues.TransferAmount), nil)
}

func (s *ProofAPITestSuite) Test_10_FilteredRecvPacketToCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.SetupSuite(ctx, proofType)
	s.FilteredRecvPacketToCosmosTest(ctx, 10, big.NewInt(testvalues.TransferAmount), []uint64{2, 6})
}

func (s *ProofAPITestSuite) FilteredRecvPacketToCosmosTest(ctx context.Context, numOfTransfers int, transferAmount *big.Int, recvFilter []uint64) {
	s.Require().GreaterOrEqual(numOfTransfers, len(recvFilter))
	s.Require().Greater(numOfTransfers, 0)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	totalTransferAmount := new(big.Int).Mul(transferAmount, big.NewInt(int64(numOfTransfers)))
	var relayedAmount *big.Int
	if len(recvFilter) == 0 {
		relayedAmount = totalTransferAmount
	} else {
		relayedAmount = new(big.Int).Mul(transferAmount, big.NewInt(int64(len(recvFilter))))
	}
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()
	escrowAddress, err := s.ics20Contract.GetEscrow(nil, testvalues.CustomClientID)
	s.Require().NoError(err)

	denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))

	escrowStartingBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
	s.Require().NoError(err)
	startBalanceEthereum, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
	s.Require().NoError(err)
	startBalanceCosmosResp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
		Address: cosmosUserAddress,
		Denom:   denomOnCosmos.IBCDenom(),
	})
	s.Require().NoError(err)
	startBalanceCosmos := startBalanceCosmosResp.Balance.Amount.BigInt()

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

	var sendTxHashes [][]byte
	s.Require().True(s.Run(fmt.Sprintf("Send %d transfers on Ethereum", numOfTransfers), func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		msgSendTransfer := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			SourceClient:     testvalues.CustomClientID,
			DestPort:         transfertypes.PortID,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			Memo:             "",
		}

		for range numOfTransfers {
			tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendTransfer)
			s.Require().NoError(err)

			receipt, err := eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

			sendTxHashes = append(sendTxHashes, tx.Hash().Bytes())
		}

		s.True(s.Run("Verify balances on Ethereum", func() {
			// User balance on Ethereum
			userBalance, err := s.erc20Contract.BalanceOf(nil, ethereumUserAddress)
			s.Require().NoError(err)
			s.Require().Equal(new(big.Int).Sub(startBalanceEthereum, totalTransferAmount), userBalance)

			// Get the escrow address
			escrowAddress, err = s.ics20Contract.GetEscrow(nil, testvalues.CustomClientID)
			s.Require().NoError(err)

			// ICS20 contract balance on Ethereum
			escrowBalance, err := s.erc20Contract.BalanceOf(nil, escrowAddress)
			s.Require().NoError(err)
			s.Require().Equal(new(big.Int).Add(escrowStartingBalance, totalTransferAmount), escrowBalance)
		}))
	}))

	s.Require().True(s.Run("Receive packets on Cosmos chain", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           eth.ChainID.String(),
				DstChain:           simd.Config().ChainID,
				SourceTxIds:        sendTxHashes,
				SrcClientId:        testvalues.CustomClientID,
				DstClientId:        testvalues.FirstWasmClientID,
				SrcPacketSequences: recvFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx

			s.wasmFixtureGenerator.AddFixtureStep("receive_packets", ethereumtypes.RelayerMessages{
				RelayerTxBody: hex.EncodeToString(relayTxBodyBz),
			})
		}))

		var ackTxHash []byte
		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 5_000_000, relayTxBodyBz)

			var err error
			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			// User balance on Cosmos chain
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(new(big.Int).Add(startBalanceCosmos, relayedAmount), resp.Balance.Amount.BigInt())
			s.Require().Equal(denomOnCosmos.IBCDenom(), resp.Balance.Denom)
		}))
	}))
}

func (s *ProofAPITestSuite) Test_10_BatchedAckPacketToCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenBatchedFilteredAckToCosmosTest(ctx, proofType, 10, nil)
}

func (s *ProofAPITestSuite) Test_10_BatchedFilteredAckPacketToCosmos() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ICS20TransferERC20TokenBatchedFilteredAckToCosmosTest(ctx, proofType, 10, []uint64{1, 2, 8})
}

// Note that the relayer still only relays one tx, the batching is done
// on the cosmos transaction itself. So that it emits multiple IBC events.
// This test also tests the filtering of the acks by packet sequence
func (s *ProofAPITestSuite) ICS20TransferERC20TokenBatchedFilteredAckToCosmosTest(
	ctx context.Context, proofType types.SupportedProofType, numOfTransfers int, ackFilter []uint64,
) {
	s.Require().GreaterOrEqual(numOfTransfers, len(ackFilter))
	s.Require().Greater(numOfTransfers, 0)

	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	transferAmount := big.NewInt(testvalues.TransferAmount)
	totalTransferAmount := big.NewInt(testvalues.TransferAmount * int64(numOfTransfers))
	if totalTransferAmount.Int64() > testvalues.InitialBalance {
		s.FailNow("Total transfer amount exceeds the initial balance")
	}
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()
	sendMemo := "batched ack to cosmos test memo"

	var (
		transferCoin sdk.Coin
		sendTxHashes [][]byte
	)
	s.Require().True(s.Run("Send transfers on Cosmos chain", func() {
		for range numOfTransfers {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
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
				SourceClient:     testvalues.FirstWasmClientID,
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

	var ackTxHash []byte
	s.Require().True(s.Run("Receive packets on Ethereum", func() {
		var multicallTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    s.Eth.Chains[0].ChainID.String(),
				SourceTxIds: sendTxHashes,
				SrcClientId: testvalues.FirstWasmClientID,
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(resp.Address, ics26Address.String())

			multicallTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, multicallTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Tx failed: %+v", receipt))

			ackTxHash = receipt.TxHash.Bytes()
		}))
	}))

	s.Require().True(s.Run("Acknowledge packets on Cosmos", func() {
		s.Require().True(s.Run("Verify commitments exists", func() {
			for i := range numOfTransfers {
				resp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
					ClientId: testvalues.FirstWasmClientID,
					Sequence: uint64(i) + 1,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Commitment)
			}
		}))

		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:           s.Eth.Chains[0].ChainID.String(),
				DstChain:           simd.Config().ChainID,
				SourceTxIds:        [][]byte{ackTxHash},
				SrcClientId:        testvalues.CustomClientID,
				DstClientId:        testvalues.FirstWasmClientID,
				DstPacketSequences: ackFilter,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, relayTxBodyBz)
		}))

		s.Require().True(s.Run("Verify commitments removed", func() {
			for i := range numOfTransfers {
				seq := uint64(i) + 1
				resp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
					ClientId: testvalues.FirstWasmClientID,
					Sequence: seq,
				})
				if len(ackFilter) == 0 || slices.Contains(ackFilter, seq) {
					// If the sequence is in the filter, we expect the commitment to be removed
					s.Require().ErrorContains(err, "packet commitment hash not found")
				} else {
					// Otherwise, we expect the commitment to still exist
					s.Require().NoError(err)
					s.Require().NotEmpty(resp.Commitment)
				}

			}
		}))
	}))
}

func (s *ProofAPITestSuite) Test_UpdateClientToCosmos() {
	if os.Getenv(testvalues.EnvKeyEthTestnetType) != testvalues.EthTestnetTypePoS {
		s.T().Skip("Test is only relevant for PoS networks")
	}

	ctx := context.Background()
	proofType := types.GetEnvProofType()

	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)

	var initialHeight uint64
	s.Require().True(s.Run("Get the initial height", func() {
		resp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simd, &clienttypes.QueryClientStateRequest{
			ClientId: testvalues.FirstWasmClientID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.ClientState)

		var wasmClientState ibcwasmtypes.ClientState
		err = proto.Unmarshal(resp.ClientState.Value, &wasmClientState)
		s.Require().NoError(err)
		s.Require().NotZero(wasmClientState.LatestHeight.RevisionHeight)

		initialHeight = wasmClientState.LatestHeight.RevisionHeight
	}))

	s.Require().True(s.Run("Wait for finality", func() {
		err := testutil.WaitForCondition(30*time.Minute, 5*time.Second, func() (bool, error) {
			resp, err := eth.BeaconAPIClient.GetFinalityUpdate()
			if err != nil {
				return false, err
			}

			// resp.Data.Message.Slot is a string, so we need to convert it to a uint64
			finalizedSlot, err := strconv.ParseUint(resp.Data.FinalizedHeader.Beacon.Slot, 10, 64)
			if err != nil {
				return false, err
			}

			finalizedBlock, err := strconv.ParseInt(resp.Data.FinalizedHeader.Execution.BlockNumber, 10, 64)
			if err != nil {
				return false, err
			}

			code, err := eth.RPCClient.CodeAt(ctx, ics26Address, big.NewInt(finalizedBlock))
			if err != nil || len(code) == 0 {
				// Code not found at the finalized block number
				return false, nil
			}

			return finalizedSlot > initialHeight, nil
		})
		s.Require().NoError(err)
	}))

	s.Require().NoError(testutil.WaitForBlocks(ctx, 1, simd))

	s.Require().True(s.Run("Update the client on Cosmos", func() {
		var updateTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.UpdateClient(context.Background(), &proofapitypes.UpdateClientRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				DstClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			updateTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, updateTxBodyBz)
		}))

		s.Require().True(s.Run("Verify the client state is updated", func() {
			resp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simd, &clienttypes.QueryClientStateRequest{
				ClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.ClientState)

			var wasmClientState ibcwasmtypes.ClientState
			err = proto.Unmarshal(resp.ClientState.Value, &wasmClientState)
			s.Require().NoError(err)

			newHeight := wasmClientState.LatestHeight.RevisionHeight
			s.Require().Greater(newHeight, initialHeight)
		}))
	}))
}

// Test_HistoricalUpdateClientToCosmos tests updating the eth light client with updates that are not the latest
// To do this, we will:
// 1. Send a transfer on Ethereum
// 2. Retrieve the relay tx from the relayer
// 3. Wait until we have a finalized block that is past the update in the relay tx
// 4. Update the client with the finalized block that is past the update in the relay tx
// 5. Finally, we will relay the transfer tx, together with the historical update client
func (s *ProofAPITestSuite) Test_HistoricalUpdateClientToCosmos() {
	if os.Getenv(testvalues.EnvKeyEthTestnetType) != testvalues.EthTestnetTypePoS {
		s.T().Skip("Test is only relevant for PoS networks")
	}

	ctx := context.Background()
	proofType := types.GetEnvProofType()

	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)

	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

	transferAmount := big.NewInt(testvalues.TransferAmount)

	s.Require().True(s.Run("Approve the ICS20Transfer.sol contract to spend the erc20 tokens", func() {
		tx, err := s.erc20Contract.Approve(s.GetTransactOpts(s.key, eth), ics20Address, transferAmount)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	var sendTxHash []byte
	s.Require().True(s.Run("Send transfers on Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		msgSendTransfer := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20Address,
			SourceClient:     testvalues.CustomClientID,
			DestPort:         transfertypes.PortID,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			Memo:             "",
		}

		tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendTransfer)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		sendTxHash = tx.Hash().Bytes()
	}))

	var relayTxBodyBz []byte
	var proofApiUpdateSlot uint64

	s.Require().True(s.Run("Retrieve relay tx", func() {
		// We need to make sure that the update slot for the relay tx is a period change, because then the update client will have to include it when it updates the client. We want the update slot for the proof API to _not_ be included in the update client message, so we wait for the non-period change update slot.
		_, ethClientState := s.GetEthereumClientState(ctx, simd, testvalues.FirstWasmClientID)
		err := testutil.WaitForCondition(30*time.Minute, 15*time.Second, func() (bool, error) {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{sendTxHash},
				SrcClientId: testvalues.CustomClientID,
				DstClientId: testvalues.FirstWasmClientID,
			})
			if err != nil {
				return false, err
			}
			if resp.Tx == nil {
				return false, fmt.Errorf("no relay tx found")
			}

			relayTxBodyBz = resp.Tx

			proofApiUpdateSlot, err = proofapi.GetRelayUpdateSlotForWasmClient(resp.Tx)
			if err != nil {
				return false, fmt.Errorf("failed to get proof API update slot: %w", err)
			}
			isPeriodChange := proofApiUpdateSlot%(ethClientState.EpochsPerSyncCommitteePeriod*ethClientState.SlotsPerEpoch) == 0

			if isPeriodChange {
				s.T().Logf("Proof API update slot %d is a period change, waiting to update past it", proofApiUpdateSlot)
				return false, nil
			}

			return true, nil
		})
		s.Require().NoError(err)

		s.wasmFixtureGenerator.AddFixtureStep("receive_packets", ethereumtypes.RelayerMessages{
			RelayerTxBody: hex.EncodeToString(relayTxBodyBz),
		})
	}))

	// Instead of relaying the tx, we will wait until we have a finalized block that is past the update in the relay tx
	// and then we will update the client on the Cosmos chain, before finally relaying the tx (where the update client will be historical)

	s.Require().True(s.Run("Wait for finality to be past update slot", func() {
		err := testutil.WaitForCondition(30*time.Minute, 5*time.Second, func() (bool, error) {
			resp, err := eth.BeaconAPIClient.GetFinalityUpdate()
			if err != nil {
				return false, err
			}

			// resp.Data.Message.Slot is a string, so we need to convert it to a uint64
			finalizedSlot, err := strconv.ParseUint(resp.Data.FinalizedHeader.Beacon.Slot, 10, 64)
			if err != nil {
				return false, err
			}

			return finalizedSlot > proofApiUpdateSlot, nil
		})
		s.Require().NoError(err)
	}))

	s.Require().NoError(testutil.WaitForBlocks(ctx, 1, simd))

	// Now we update the client on the Cosmos chain past the relay update
	var latestHeight uint64
	s.Require().True(s.Run("Update the client on Cosmos", func() {
		var updateTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.UpdateClient(context.Background(), &proofapitypes.UpdateClientRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    simd.Config().ChainID,
				DstClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			updateTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			_ = s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, updateTxBodyBz)
		}))

		s.Require().True(s.Run("Verify the client state is updated", func() {
			resp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simd, &clienttypes.QueryClientStateRequest{
				ClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.ClientState)

			var wasmClientState ibcwasmtypes.ClientState
			err = proto.Unmarshal(resp.ClientState.Value, &wasmClientState)
			s.Require().NoError(err)

			latestHeight = wasmClientState.LatestHeight.RevisionHeight
			s.Require().Greater(latestHeight, proofApiUpdateSlot)
		}))
	}))

	// And now we can relay the tx with the historical update
	s.Require().True(s.Run("Receive packets on Cosmos chain", func() {
		s.Require().True(s.Run("Verify consensus state does not exist before relaying", func() {
			_, err := e2esuite.GRPCQuery[clienttypes.QueryConsensusStateResponse](ctx, simd, &clienttypes.QueryConsensusStateRequest{
				ClientId:       testvalues.FirstWasmClientID,
				RevisionNumber: 0,
				RevisionHeight: proofApiUpdateSlot,
				LatestHeight:   false,
			})
			s.Require().ErrorContains(err, "consensus state not found")
		}))

		var ackTxHash []byte
		s.Require().True(s.Run("Broadcast relay tx", func() {
			resp := s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, relayTxBodyBz)

			var err error
			ackTxHash, err = hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			s.Require().NotEmpty(ackTxHash)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
			denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))

			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   denomOnCosmos.IBCDenom(),
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(transferAmount, resp.Balance.Amount.BigInt())
			s.Require().Equal(denomOnCosmos.IBCDenom(), resp.Balance.Denom)
		}))

		s.Require().True(s.Run("Verify consensus state exist after relaying", func() {
			_, err := e2esuite.GRPCQuery[clienttypes.QueryConsensusStateResponse](ctx, simd, &clienttypes.QueryConsensusStateRequest{
				ClientId:       testvalues.FirstWasmClientID,
				RevisionNumber: 0,
				RevisionHeight: proofApiUpdateSlot,
				LatestHeight:   false,
			})
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Verify latest client state has not changed", func() {
			resp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, simd, &clienttypes.QueryClientStateRequest{
				ClientId: testvalues.FirstWasmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.ClientState)

			var wasmClientState ibcwasmtypes.ClientState
			err = proto.Unmarshal(resp.ClientState.Value, &wasmClientState)
			s.Require().NoError(err)

			heightBefore := latestHeight

			latestHeight = wasmClientState.LatestHeight.RevisionHeight
			s.Require().Equal(heightBefore, latestHeight, "The latest height should not change after relaying the tx with historical update")
		}))
	}))
}

func (s *ProofAPITestSuite) Test_UpdateClientToEth() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.UpdateClientToEthTest(ctx, proofType)
}

func (s *ProofAPITestSuite) UpdateClientToEthTest(ctx context.Context, proofType types.SupportedProofType) {
	s.SetupSuite(ctx, proofType) // Doesn't matter, since we won't relay to eth in this test

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)

	var initialHeight uint64
	s.Require().True(s.Run("Get the initial height", func() {
		clientState, err := s.sp1Ics07Contract.ClientState(nil)
		s.Require().NoError(err)
		s.Require().NotZero(clientState.LatestHeight.RevisionHeight)

		initialHeight = clientState.LatestHeight.RevisionHeight
	}))

	s.Require().True(s.Run("Update the client on Ethereum", func() {
		var updateTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.UpdateClient(context.Background(), &proofapitypes.UpdateClientRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				DstClientId: testvalues.CustomClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Equal(ics26Address.String(), resp.Address)

			updateTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx", func() {
			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 5_000_000, &ics26Address, updateTxBodyBz)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		}))

		s.Require().True(s.Run("Verify the client state is updated", func() {
			clientState, err := s.sp1Ics07Contract.ClientState(nil)
			s.Require().NoError(err)
			s.Require().NotZero(clientState.LatestHeight.RevisionHeight)

			newHeight := clientState.LatestHeight.RevisionHeight
			s.Require().Greater(newHeight, initialHeight)
		}))
	}))
}

// Test_50_concurrent_RecvPacketToCosmosTest tests the concurrent relaying of 50 packets from Ethereum to Cosmos
func (s *ProofAPITestSuite) Test_50_concurrent_RecvPacketToCosmosTest() {
	ctx := context.Background()
	proofType := types.GetEnvProofType()
	s.ConcurrentRecvPacketToCosmos(ctx, proofType, 50)
}

func (s *ProofAPITestSuite) ConcurrentRecvPacketToCosmos(
	ctx context.Context, proofType types.SupportedProofType, numConcurrentTransfers int,
) {
	s.Require().Greater(numConcurrentTransfers, 0)

	s.SetupSuite(ctx, proofType)

	eth, simd := s.Eth.Chains[0], s.Cosmos.Chains[0]

	ics20Address := ethcommon.HexToAddress(s.contractAddresses.Ics20Transfer)
	erc20Address := ethcommon.HexToAddress(s.contractAddresses.Erc20)
	transferAmount := big.NewInt(testvalues.TransferAmount)
	totalTransferAmount := big.NewInt(testvalues.TransferAmount * int64(numConcurrentTransfers))
	if totalTransferAmount.Int64() > testvalues.InitialBalance {
		s.FailNow("Total transfer amount exceeds the initial balance")
	}
	ethereumUserAddress := crypto.PubkeyToAddress(s.key.PublicKey)
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()

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
		sendTxHashes  [][]byte
		escrowAddress ethcommon.Address
	)
	s.Require().True(s.Run(fmt.Sprintf("Send %d transfers on Ethereum", numConcurrentTransfers), func() {
		// Setting the timeout high to avoid timeouts on mainnet preset
		timeout := uint64(time.Now().Add(120 * time.Minute).Unix())

		msgSendPacket := ics20transfer.IICS20TransferMsgsSendTransferMsg{
			SourceClient:     testvalues.CustomClientID,
			Denom:            erc20Address,
			Amount:           transferAmount,
			Receiver:         cosmosUserAddress,
			TimeoutTimestamp: timeout,
			Memo:             "",
		}

		for range numConcurrentTransfers {
			tx, err := s.ics20Contract.SendTransfer(s.GetTransactOpts(s.key, eth), msgSendPacket)
			s.Require().NoError(err)

			receipt, err := eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
			sendTxHashes = append(sendTxHashes, tx.Hash().Bytes())
		}

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

	s.Require().True(s.Run("Relay the last packet", func() {
		relayResp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    eth.ChainID.String(),
			DstChain:    simd.Config().ChainID,
			SourceTxIds: [][]byte{sendTxHashes[len(sendTxHashes)-1]},
			SrcClientId: testvalues.CustomClientID,
			DstClientId: testvalues.FirstWasmClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(relayResp.Tx)
		s.Require().Empty(relayResp.Address)

		relayTxBodyBz := relayResp.Tx

		_ = s.MustBroadcastSdkTxBody(ctx, simd, s.SimdRelayerSubmitter, 2_000_000, relayTxBodyBz)

		// Remove the last txHash from the list
		sendTxHashes = sendTxHashes[:len(sendTxHashes)-1]
	}))

	var eg errgroup.Group
	s.Require().True(s.Run("Relay all the remaining requests concurrently", func() {
		// to avoid the submitter from getting account sequence mismatch, we need to lock when submitting
		time.Sleep(10 * time.Second) // Just to make sure we are up to date

		var (
			relayTxHashes [][]byte
			mu            sync.Mutex
		)
		// loop over the txHashes and send them concurrently
		for _, txHash := range sendTxHashes {
			txHash := txHash
			eg.Go(func() error {
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    eth.ChainID.String(),
					DstChain:    simd.Config().ChainID,
					SourceTxIds: [][]byte{txHash},
					SrcClientId: testvalues.CustomClientID,
					DstClientId: testvalues.FirstWasmClientID,
				})
				if err != nil {
					return err
				}

				relayTxBodyBz := resp.Tx
				mu.Lock()
				relayTxHashes = append(relayTxHashes, relayTxBodyBz)
				mu.Unlock()
				return nil
			})
		}

		s.Require().NoError(eg.Wait())

		// relay 10 packets at a time
		for i := 0; i < len(relayTxHashes); i += 10 {
			txHashes := relayTxHashes[i:min(i+10, len(relayTxHashes))]

			var msgs []sdk.Msg
			for _, txHash := range txHashes {
				var txBody txtypes.TxBody
				err := proto.Unmarshal(txHash, &txBody)
				s.Require().NoError(err)

				for _, msg := range txBody.Messages {
					// Make sure there are no update client messages
					s.Require().NotEqual("ibc.core.client.v1.MsgUpdateClient", msg.TypeUrl)

					var sdkMsg sdk.Msg
					err = simd.Config().EncodingConfig.InterfaceRegistry.UnpackAny(msg, &sdkMsg)
					s.Require().NoError(err)

					msgs = append(msgs, sdkMsg)
				}
			}
			s.Require().NotZero(len(msgs))

			_, err := s.BroadcastMessages(ctx, simd, s.SimdRelayerSubmitter, 5_000_000, msgs...)
			s.Require().NoError(err)
		}
	}))

	s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
		denomOnCosmos := transfertypes.NewDenom(s.contractAddresses.Erc20, transfertypes.NewHop(transfertypes.PortID, testvalues.FirstWasmClientID))

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
}
