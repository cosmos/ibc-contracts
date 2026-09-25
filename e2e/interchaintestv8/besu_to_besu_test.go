// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"crypto/ecdsa"
	"fmt"
	"math/big"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	goethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	ethcommon "github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/rpc"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"

	"github.com/cosmos/interchaintest/v11/testutil"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/besumsgs"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/besuqbft"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ibcerc20"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics20transfer"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	e2etypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/erc20"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

const (
	besuToBesuChainBID = 1338

	besuToBesuClientOnA = "besu-chain-b"
	besuToBesuClientOnB = "besu-chain-a"

	besuToBesuConsensusTypeQBFT = "qbft"
)

var besuToBesuChainBIPs = [4]string{"10.43.0.2", "10.43.0.3", "10.43.0.4", "10.43.0.5"}

type besuToBesuChainState struct {
	network           chainconfig.BesuQBFTChain
	eth               ethereum.Ethereum
	contractAddresses ethereum.DeployedContracts
	ics26             *ics26router.Contract
	ics20             *ics20transfer.Contract
	erc20             *erc20.Contract
	deployer          *ecdsa.PrivateKey
	user              *ecdsa.PrivateKey
	relayerSubmitter  *ecdsa.PrivateKey
	clientAddress     ethcommon.Address
}

type BesuToBesuTestSuite struct {
	suite.Suite

	cwd                  string
	relayerConfigPath    string
	relayerProcess       *os.Process
	relayerClient        proofapitypes.ProofApiServiceClient
	besuFixtureGenerator *e2etypes.BesuFixtureGenerator

	chainA besuToBesuChainState
	chainB besuToBesuChainState
}

func TestWithBesuToBesuTestSuite(t *testing.T) {
	suite.Run(t, new(BesuToBesuTestSuite))
}

func (s *BesuToBesuTestSuite) SetupSuite() {
	s.T().Cleanup(func() {
		ctx := context.Background()

		if s.T() != nil && s.T().Failed() {
			_ = s.chainA.network.DumpLogs(ctx)
			_ = s.chainB.network.DumpLogs(ctx)
		}

		if s.relayerProcess != nil {
			_ = s.relayerProcess.Kill()
		}
		if s.relayerConfigPath != "" {
			_ = os.Remove(s.relayerConfigPath)
		}

		s.chainB.network.Destroy(ctx)
		s.chainA.network.Destroy(ctx)

		if s.cwd != "" {
			_ = os.Chdir(s.cwd)
		}
	})
	ctx := context.Background()

	if os.Getenv(testvalues.EnvKeyRustLog) == "" {
		os.Setenv(testvalues.EnvKeyRustLog, testvalues.EnvValueRustLog_Info)
	}

	var err error
	s.cwd, err = os.Getwd()
	s.Require().NoError(err)
	s.Require().NoError(os.Chdir("../.."))

	s.chainA = s.spinUpChain(ctx, chainconfig.DefaultBesuQBFTParams())

	s.chainB = s.spinUpChain(ctx, chainconfig.BesuQBFTParams{
		ChainID:      besuToBesuChainBID,
		Subnet:       "10.43.0.0/16",
		Gateway:      "10.43.0.1",
		ValidatorIPs: besuToBesuChainBIPs,
	})
	s.Require().NoError(s.chainA.network.WaitForTransactionHandling(ctx))

	s.besuFixtureGenerator = e2etypes.NewBesuFixtureGenerator()

	s.createUsers(&s.chainA)
	s.createUsers(&s.chainB)

	s.deployContracts(&s.chainA)
	s.deployContracts(&s.chainB)

	s.startRelayer()
	s.connectRelayer()

	s.chainA.clientAddress = s.createAndRegisterBesuClient(&s.chainB, &s.chainA, besuToBesuClientOnA, besuToBesuClientOnB)
	s.chainB.clientAddress = s.createAndRegisterBesuClient(&s.chainA, &s.chainB, besuToBesuClientOnB, besuToBesuClientOnA)
}

func (s *BesuToBesuTestSuite) Test_Deploy() {
	s.Require().True(s.Run("Verify ICS26 on Chain A", func() {
		transferOnA, err := s.chainA.ics26.GetIBCApp(nil, transfertypes.PortID)
		s.Require().NoError(err)
		s.Require().Equal(strings.ToLower(s.chainA.contractAddresses.Ics20Transfer), strings.ToLower(transferOnA.Hex()))
	}))

	s.Require().True(s.Run("Verify ICS26 on Chain B", func() {
		transferOnB, err := s.chainB.ics26.GetIBCApp(nil, transfertypes.PortID)
		s.Require().NoError(err)
		s.Require().Equal(strings.ToLower(s.chainB.contractAddresses.Ics20Transfer), strings.ToLower(transferOnB.Hex()))
	}))

	s.Require().True(s.Run("Verify Besu client on Chain A", func() {
		clientOnA, err := s.chainA.ics26.GetClient(nil, besuToBesuClientOnA)
		s.Require().NoError(err)
		s.Require().Equal(s.chainA.clientAddress, clientOnA)

		counterparty, err := s.chainA.ics26.GetCounterparty(nil, besuToBesuClientOnA)
		s.Require().NoError(err)
		s.Require().Equal(besuToBesuClientOnB, counterparty.ClientId)
	}))

	s.Require().True(s.Run("Verify Besu client on Chain B", func() {
		clientOnB, err := s.chainB.ics26.GetClient(nil, besuToBesuClientOnB)
		s.Require().NoError(err)
		s.Require().Equal(s.chainB.clientAddress, clientOnB)

		counterparty, err := s.chainB.ics26.GetCounterparty(nil, besuToBesuClientOnB)
		s.Require().NoError(err)
		s.Require().Equal(besuToBesuClientOnA, counterparty.ClientId)
	}))

	s.Require().True(s.Run("Verify Proof API Info A->B", func() {
		info := s.waitForRelayerInfo(s.chainA.eth.ChainID.String(), s.chainB.eth.ChainID.String())
		s.Require().Equal(s.chainA.eth.ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(s.chainB.eth.ChainID.String(), info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Proof API Info B->A", func() {
		info := s.waitForRelayerInfo(s.chainB.eth.ChainID.String(), s.chainA.eth.ChainID.String())
		s.Require().Equal(s.chainB.eth.ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(s.chainA.eth.ChainID.String(), info.TargetChain.ChainId)
	}))
}

func (s *BesuToBesuTestSuite) Test_ICS20TransferERC20FromChainAToChainB() {
	ctx := context.Background()
	transferAmount := big.NewInt(testvalues.TransferAmount)
	userAddressA := crypto.PubkeyToAddress(s.chainA.user.PublicKey)
	userAddressB := crypto.PubkeyToAddress(s.chainB.user.PublicKey)
	ics20AddressA := ethcommon.HexToAddress(s.chainA.contractAddresses.Ics20Transfer)
	ics26AddressA := ethcommon.HexToAddress(s.chainA.contractAddresses.Ics26Router)
	ics26AddressB := ethcommon.HexToAddress(s.chainB.contractAddresses.Ics26Router)
	erc20AddressA := ethcommon.HexToAddress(s.chainA.contractAddresses.Erc20)

	var (
		sendTxHash  []byte
		sendReceipt *ethtypes.Receipt
		sendPacket  ics26router.IICS26RouterMsgsPacket
		recvReceipt *ethtypes.Receipt
		ibcERC20OnB *ibcerc20.Contract
		ackReceipt  *ethtypes.Receipt
	)

	s.Require().True(s.Run("Fund user on Chain A", func() {
		fundTx, err := s.chainA.erc20.Transfer(s.mustTransactOpts(&s.chainA, s.chainA.eth.Faucet), userAddressA, testvalues.StartingERC20Balance)
		s.Require().NoError(err)

		fundReceipt, err := s.chainA.eth.GetTxReciept(ctx, fundTx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, fundReceipt.Status)
	}))

	s.Require().True(s.Run("Approve ICS20 on Chain A", func() {
		approveTx, err := s.chainA.erc20.Approve(s.mustTransactOpts(&s.chainA, s.chainA.user), ics20AddressA, transferAmount)
		s.Require().NoError(err)

		approveReceipt, err := s.chainA.eth.GetTxReciept(ctx, approveTx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, approveReceipt.Status)
	}))

	s.Require().True(s.Run("Send transfer from Chain A to Chain B", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		sendTx, err := s.chainA.ics20.SendTransfer(s.mustTransactOpts(&s.chainA, s.chainA.user), ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20AddressA,
			Amount:           transferAmount,
			Receiver:         strings.ToLower(userAddressB.Hex()),
			TimeoutTimestamp: timeout,
			SourceClient:     besuToBesuClientOnA,
			DestPort:         transfertypes.PortID,
			Memo:             "",
		})
		s.Require().NoError(err)

		sendReceipt, err = s.chainA.eth.GetTxReciept(ctx, sendTx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, sendReceipt.Status)
		sendTxHash = sendTx.Hash().Bytes()

		sendEvent, err := e2esuite.GetEvmEvent(sendReceipt, s.chainA.ics26.ParseSendPacket)
		s.Require().NoError(err)
		sendPacket = sendEvent.Packet
	}))

	s.Require().True(s.Run("Verify balances on Chain A after send", func() {
		escrowAddress, err := s.chainA.ics20.GetEscrow(nil, besuToBesuClientOnA)
		s.Require().NoError(err)

		escrowBalance, err := s.chainA.erc20.BalanceOf(nil, escrowAddress)
		s.Require().NoError(err)
		s.Require().Equal(0, transferAmount.Cmp(escrowBalance))

		userBalanceA, err := s.chainA.erc20.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		expectedBalanceA := new(big.Int).Sub(new(big.Int).Set(testvalues.StartingERC20Balance), transferAmount)
		s.Require().Equal(0, expectedBalanceA.Cmp(userBalanceA))
	}))

	s.Require().True(s.Run("Relay packet to Chain B", func() {
		var relayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			relayAB, err := s.relayerClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.chainA.eth.ChainID.String(),
				DstChain:    s.chainB.eth.ChainID.String(),
				SourceTxIds: [][]byte{sendTxHash},
				SrcClientId: besuToBesuClientOnA,
				DstClientId: besuToBesuClientOnB,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(relayAB.Tx)
			s.Require().Equal(strings.ToLower(s.chainB.contractAddresses.Ics26Router), strings.ToLower(relayAB.Address))
			relayTx = relayAB.Tx
		}))

		s.Require().True(s.Run("Broadcast relay tx on Chain B", func() {
			var err error
			recvReceipt, err = s.chainB.eth.BroadcastTx(ctx, s.chainB.relayerSubmitter, 15_000_000, &ics26AddressB, relayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, recvReceipt.Status)

			writeAckEvent, err := e2esuite.GetEvmEvent(recvReceipt, s.chainB.ics26.ParseWriteAcknowledgement)
			s.Require().NoError(err)

			ibcDenomOnB := fmt.Sprintf(
				"%s/%s/%s",
				writeAckEvent.Packet.Payloads[0].DestPort,
				writeAckEvent.Packet.DestClient,
				strings.ToLower(erc20AddressA.Hex()),
			)
			ibcERC20AddressOnB, err := s.chainB.ics20.IbcERC20Contract(nil, ibcDenomOnB)
			s.Require().NoError(err)
			ibcERC20OnB, err = ibcerc20.NewContract(ibcERC20AddressOnB, s.chainB.eth.RPCClient)
			s.Require().NoError(err)
		}))
	}))

	s.Require().True(s.Run("Verify balances on Chain B after receive", func() {
		userBalanceB, err := ibcERC20OnB.BalanceOf(nil, userAddressB)
		s.Require().NoError(err)
		s.Require().Equal(0, transferAmount.Cmp(userBalanceB))
	}))

	s.Require().True(s.Run("Relay acknowledgement to Chain A", func() {
		var relayTx []byte
		s.Require().True(s.Run("Retrieve acknowledgement relay tx", func() {
			ackRelay, err := s.relayerClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    s.chainB.eth.ChainID.String(),
				DstChain:    s.chainA.eth.ChainID.String(),
				SourceTxIds: [][]byte{recvReceipt.TxHash.Bytes()},
				SrcClientId: besuToBesuClientOnB,
				DstClientId: besuToBesuClientOnA,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(ackRelay.Tx)
			s.Require().Equal(strings.ToLower(s.chainA.contractAddresses.Ics26Router), strings.ToLower(ackRelay.Address))
			relayTx = ackRelay.Tx
		}))

		s.Require().True(s.Run("Broadcast acknowledgement relay tx on Chain A", func() {
			var err error
			ackReceipt, err = s.chainA.eth.BroadcastTx(ctx, s.chainA.relayerSubmitter, 15_000_000, &ics26AddressA, relayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, ackReceipt.Status)

			_, err = e2esuite.GetEvmEvent(ackReceipt, s.chainA.ics26.ParseAckPacket)
			s.Require().NoError(err)
		}))
	}))

	if s.besuFixtureGenerator.Enabled {
		s.Require().True(s.Run("Generate QBFT fixture", func() {
			sendHeight := sendReceipt.BlockNumber.Uint64()
			s.Require().Greater(sendHeight, uint64(2))
			s.Require().Greater(ackReceipt.BlockNumber.Uint64(), sendHeight)
			s.Require().NoError(s.besuFixtureGenerator.GenerateAndSaveQBFTFixture(ctx, e2etypes.GenerateQBFTFixtureParams{
				SourceChain:             &s.chainA.eth,
				RouterAddress:           ics26AddressA,
				Packet:                  sendPacket,
				InitialTrustedHeight:    sendHeight - 2,
				AdjacentUpdateHeight:    sendHeight - 1,
				NonAdjacentUpdateHeight: sendHeight,
				SyntheticSourceHeight:   ackReceipt.BlockNumber.Uint64(),
				TrustingPeriod:          uint64(testvalues.DefaultTrustPeriod),
				MaxClockDrift:           uint64(testvalues.DefaultMaxClockDrift),
			}))
		}))
	}
}

func (s *BesuToBesuTestSuite) Test_TimeoutICS20TransferERC20FromChainAToChainB() {
	ctx := context.Background()
	transferAmount := big.NewInt(testvalues.TransferAmount)
	userAddressA := crypto.PubkeyToAddress(s.chainA.user.PublicKey)
	userAddressB := crypto.PubkeyToAddress(s.chainB.user.PublicKey)
	ics20AddressA := ethcommon.HexToAddress(s.chainA.contractAddresses.Ics20Transfer)
	ics26AddressA := ethcommon.HexToAddress(s.chainA.contractAddresses.Ics26Router)
	erc20AddressA := ethcommon.HexToAddress(s.chainA.contractAddresses.Erc20)

	var (
		initialUserBalanceA   *big.Int
		initialEscrowBalanceA *big.Int
		escrowAddressA        ethcommon.Address
		sendTxHash            []byte
		sendPacket            ics26router.IICS26RouterMsgsPacket
		packetTimeout         uint64
	)

	s.Require().True(s.Run("Fund user on Chain A", func() {
		fundTx, err := s.chainA.erc20.Transfer(s.mustTransactOpts(&s.chainA, s.chainA.eth.Faucet), userAddressA, transferAmount)
		s.Require().NoError(err)

		fundReceipt, err := s.chainA.eth.GetTxReciept(ctx, fundTx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, fundReceipt.Status)

		initialUserBalanceA, err = s.chainA.erc20.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		escrowAddressA, err = s.chainA.ics20.GetEscrow(nil, besuToBesuClientOnA)
		s.Require().NoError(err)
		initialEscrowBalanceA, err = s.chainA.erc20.BalanceOf(nil, escrowAddressA)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Approve ICS20 on Chain A", func() {
		approveTx, err := s.chainA.erc20.Approve(s.mustTransactOpts(&s.chainA, s.chainA.user), ics20AddressA, transferAmount)
		s.Require().NoError(err)

		approveReceipt, err := s.chainA.eth.GetTxReciept(ctx, approveTx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, approveReceipt.Status)
	}))

	s.Require().True(s.Run("Send transfer with short timeout from Chain A", func() {
		chainATime, err := s.chainA.eth.GetBlockTime(ctx)
		s.Require().NoError(err)
		chainBTime, err := s.chainB.eth.GetBlockTime(ctx)
		s.Require().NoError(err)
		packetTimeout = uint64(max(chainATime, chainBTime)) + 15

		sendTx, err := s.chainA.ics20.SendTransfer(s.mustTransactOpts(&s.chainA, s.chainA.user), ics20transfer.IICS20TransferMsgsSendTransferMsg{
			Denom:            erc20AddressA,
			Amount:           transferAmount,
			Receiver:         strings.ToLower(userAddressB.Hex()),
			TimeoutTimestamp: packetTimeout,
			SourceClient:     besuToBesuClientOnA,
			DestPort:         transfertypes.PortID,
			Memo:             "",
		})
		s.Require().NoError(err)

		sendReceipt, err := s.chainA.eth.GetTxReciept(ctx, sendTx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, sendReceipt.Status)
		sendTxHash = sendTx.Hash().Bytes()

		sendEvent, err := e2esuite.GetEvmEvent(sendReceipt, s.chainA.ics26.ParseSendPacket)
		s.Require().NoError(err)
		sendPacket = sendEvent.Packet
	}))

	s.Require().True(s.Run("Verify balances on Chain A after send", func() {
		userBalanceA, err := s.chainA.erc20.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		expectedUserBalanceA := new(big.Int).Sub(new(big.Int).Set(initialUserBalanceA), transferAmount)
		s.Require().Equal(0, expectedUserBalanceA.Cmp(userBalanceA))

		escrowAddressA, err = s.chainA.ics20.GetEscrow(nil, besuToBesuClientOnA)
		s.Require().NoError(err)
		s.Require().NotEqual(ethcommon.Address{}, escrowAddressA)
		escrowBalanceA, err := s.chainA.erc20.BalanceOf(nil, escrowAddressA)
		s.Require().NoError(err)
		expectedEscrowBalanceA := new(big.Int).Add(new(big.Int).Set(initialEscrowBalanceA), transferAmount)
		s.Require().Equal(0, expectedEscrowBalanceA.Cmp(escrowBalanceA))
	}))

	s.Require().True(s.Run("Wait for timeout on Chain B", func() {
		s.Require().NoError(e2esuite.WaitForBlockTime(ctx, s.T(), &s.chainB.eth, packetTimeout))
	}))

	s.Require().True(s.Run("Relay timeout to Chain A", func() {
		var timeoutRelayTx []byte
		s.Require().True(s.Run("Retrieve timeout relay tx", func() {
			timeoutRelay, err := s.relayerClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:     s.chainB.eth.ChainID.String(),
				DstChain:     s.chainA.eth.ChainID.String(),
				TimeoutTxIds: [][]byte{sendTxHash},
				SrcClientId:  besuToBesuClientOnB,
				DstClientId:  besuToBesuClientOnA,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(timeoutRelay.Tx)
			s.Require().Equal(strings.ToLower(s.chainA.contractAddresses.Ics26Router), strings.ToLower(timeoutRelay.Address))
			timeoutRelayTx = timeoutRelay.Tx
		}))

		s.Require().True(s.Run("Broadcast timeout relay tx on Chain A", func() {
			timeoutReceipt, err := s.chainA.eth.BroadcastTx(ctx, s.chainA.relayerSubmitter, 15_000_000, &ics26AddressA, timeoutRelayTx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, timeoutReceipt.Status)

			timeoutEvent, err := e2esuite.GetEvmEvent(timeoutReceipt, s.chainA.ics26.ParseTimeoutPacket)
			s.Require().NoError(err)
			s.Require().Equal(sendPacket, timeoutEvent.Packet)
		}))
	}))

	s.Require().True(s.Run("Verify tokens refunded on Chain A", func() {
		userBalanceA, err := s.chainA.erc20.BalanceOf(nil, userAddressA)
		s.Require().NoError(err)
		s.Require().Equal(0, initialUserBalanceA.Cmp(userBalanceA))

		escrowBalanceA, err := s.chainA.erc20.BalanceOf(nil, escrowAddressA)
		s.Require().NoError(err)
		s.Require().Equal(0, initialEscrowBalanceA.Cmp(escrowBalanceA))
	}))
}

// Test_DoubleSignFreezesClient freezes a dedicated client on Chain B so the shared clients stay usable.
func (s *BesuToBesuTestSuite) Test_DoubleSignFreezesClient() {
	ctx := context.Background()

	const doubleSignClientID = "besu-double-sign"
	var (
		client        *besuqbft.Contract
		trustedHeight uint64
		height        uint64
		trustedHash   [32]byte
		updateMsg     []byte
	)

	s.Require().True(s.Run("Create dedicated client on Chain B", func() {
		client = s.createDedicatedBesuClient(doubleSignClientID)
		trustedHeight = s.besuClientState(client).LatestHeight.RevisionHeight
		height = trustedHeight + 1
	}))

	s.Require().True(s.Run("Update the client to the next height", func() {
		s.waitForBlock(ctx, &s.chainA, height)
		honestUpdate, err := e2etypes.BuildQBFTUpdate(ctx, &s.chainA.eth, trustedHeight, height)
		s.Require().NoError(err)
		s.requireUpdateResult(s.updateClient(ctx, doubleSignClientID, honestUpdate), 0) // Update

		trustedHash, err = client.GetConsensusStateHash(nil, height)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Build conflicting header at the stored height", func() {
		var err error
		updateMsg, err = e2etypes.BuildQBFTDoubleSignUpdate(ctx, &s.chainA.eth, trustedHeight, height)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Submit double sign and freeze the client", func() {
		receipt := s.updateClient(ctx, doubleSignClientID, updateMsg)
		s.requireUpdateResult(receipt, 1) // Misbehaviour

		doubleSignEvent, err := e2esuite.GetEvmEvent(receipt, client.ParseDoubleSign)
		s.Require().NoError(err)
		s.Require().Equal(height, doubleSignEvent.RevisionHeight)
		s.Require().Equal(trustedHash, doubleSignEvent.TrustedConsensusStateHash)
		s.Require().NotEqual(trustedHash, doubleSignEvent.ConflictingConsensusStateHash)

		s.Require().True(s.besuClientState(client).Frozen)
		storedHash, err := client.GetConsensusStateHash(nil, height)
		s.Require().NoError(err)
		s.Require().Equal(trustedHash, storedHash)
	}))

	s.Require().True(s.Run("Frozen client rejects further updates", func() {
		s.requireFrozenClientRevert(ctx, "updateClient", doubleSignClientID, updateMsg)
	}))
}

// Test_TimeNonMonotonicityFreezesClient stores two consensus states whose timestamps do not increase with height on
// a dedicated client on Chain B, then submits them as misbehaviour. Each update is only checked against its own
// trusted height, so both updates are accepted.
func (s *BesuToBesuTestSuite) Test_TimeNonMonotonicityFreezesClient() {
	ctx := context.Background()

	const timeMisbehaviourClientID = "besu-time-non-monotonicity"
	var (
		client          *besuqbft.Contract
		trustedHeight   uint64
		height1         uint64
		height2         uint64
		timestamp       uint64
		misbehaviourMsg []byte
	)

	s.Require().True(s.Run("Create dedicated client on Chain B", func() {
		client = s.createDedicatedBesuClient(timeMisbehaviourClientID)
		trustedHeight = s.besuClientState(client).LatestHeight.RevisionHeight
		height1, height2 = trustedHeight+1, trustedHeight+2
	}))

	s.Require().True(s.Run("Store consensus states with non-monotonic timestamps", func() {
		s.waitForBlock(ctx, &s.chainA, height2)

		state2, err := e2etypes.FetchQBFTConsensusState(ctx, &s.chainA.eth, height2)
		s.Require().NoError(err)
		timestamp = state2.Timestamp

		honestUpdate, err := e2etypes.BuildQBFTUpdate(ctx, &s.chainA.eth, trustedHeight, height2)
		s.Require().NoError(err)
		s.requireUpdateResult(s.updateClient(ctx, timeMisbehaviourClientID, honestUpdate), 0) // Update

		// Height1 is re-sealed with the timestamp of height2.
		forgedUpdate, err := e2etypes.BuildQBFTTimestampUpdate(ctx, &s.chainA.eth, trustedHeight, height1, timestamp)
		s.Require().NoError(err)
		s.requireUpdateResult(s.updateClient(ctx, timeMisbehaviourClientID, forgedUpdate), 0) // Update

		state1, err := e2etypes.FetchQBFTConsensusState(ctx, &s.chainA.eth, height1)
		s.Require().NoError(err)
		state1.Timestamp = timestamp

		// The light client expects abi.encode(MsgTimeNonMonotonicityMisbehaviour), without the function selector.
		misbehaviourMsg = besumsgs.NewBindings().PackTimeNonMonotonicityMisbehaviour(
			besumsgs.IBesuLightClientMsgsMsgTimeNonMonotonicityMisbehaviour{
				Height1:                 besumsgs.IICS02ClientMsgsHeight{RevisionHeight: height1},
				Height2:                 besumsgs.IICS02ClientMsgsHeight{RevisionHeight: height2},
				ConsensusStatePreimage1: state1,
				ConsensusStatePreimage2: state2,
			},
		)[4:]
		s.Require().False(s.besuClientState(client).Frozen)
	}))

	s.Require().True(s.Run("Submit misbehaviour and freeze the client", func() {
		tx, err := s.chainB.ics26.SubmitMisbehaviour(
			s.mustTransactOpts(&s.chainB, s.chainB.relayerSubmitter), timeMisbehaviourClientID, misbehaviourMsg,
		)
		s.Require().NoError(err)
		receipt, err := s.chainB.eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		submittedEvent, err := e2esuite.GetEvmEvent(receipt, s.chainB.ics26.ParseICS02MisbehaviourSubmitted)
		s.Require().NoError(err)
		s.Require().Equal(timeMisbehaviourClientID, submittedEvent.ClientId)

		timeEvent, err := e2esuite.GetEvmEvent(receipt, client.ParseTimeNonMonotonicity)
		s.Require().NoError(err)
		s.Require().Equal(height2, timeEvent.Height2)
		s.Require().Equal(height1, timeEvent.Height1)
		s.Require().Equal(timestamp, timeEvent.Timestamp2)
		s.Require().Equal(timestamp, timeEvent.Timestamp1)

		s.Require().True(s.besuClientState(client).Frozen)
	}))

	s.Require().True(s.Run("Frozen client rejects further misbehaviour", func() {
		s.requireFrozenClientRevert(ctx, "submitMisbehaviour", timeMisbehaviourClientID, misbehaviourMsg)
	}))
}

// createDedicatedBesuClient registers a Chain A client on Chain B that the relayer does not use.
func (s *BesuToBesuTestSuite) createDedicatedBesuClient(clientID string) *besuqbft.Contract {
	clientAddress := s.createAndRegisterBesuClient(&s.chainA, &s.chainB, clientID, besuToBesuClientOnA)
	client, err := besuqbft.NewContract(clientAddress, s.chainB.eth.RPCClient)
	s.Require().NoError(err)
	s.Require().False(s.besuClientState(client).Frozen)
	return client
}

func (s *BesuToBesuTestSuite) waitForBlock(ctx context.Context, chain *besuToBesuChainState, height uint64) {
	s.Require().NoError(testutil.WaitForCondition(time.Minute, time.Second, func() (bool, error) {
		latest, err := chain.eth.RPCClient.BlockNumber(ctx)
		return latest >= height, err
	}))
}

// updateClient submits updateMsg to clientID on Chain B through ICS26 and returns the successful receipt.
func (s *BesuToBesuTestSuite) updateClient(ctx context.Context, clientID string, updateMsg []byte) *ethtypes.Receipt {
	tx, err := s.chainB.ics26.UpdateClient(s.mustTransactOpts(&s.chainB, s.chainB.relayerSubmitter), clientID, updateMsg)
	s.Require().NoError(err)
	receipt, err := s.chainB.eth.GetTxReciept(ctx, tx.Hash())
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	return receipt
}

// requireUpdateResult asserts the ILightClientMsgs.UpdateResult emitted in receipt.
func (s *BesuToBesuTestSuite) requireUpdateResult(receipt *ethtypes.Receipt, result uint8) {
	updatedEvent, err := e2esuite.GetEvmEvent(receipt, s.chainB.ics26.ParseICS02ClientUpdated)
	s.Require().NoError(err)
	s.Require().Equal(result, updatedEvent.Result)
}

// requireFrozenClientRevert asserts that calling the ICS26 method on Chain B reverts with FrozenClientState.
func (s *BesuToBesuTestSuite) requireFrozenClientRevert(ctx context.Context, method string, args ...any) {
	ics26ABI, err := ics26router.ContractMetaData.GetAbi()
	s.Require().NoError(err)
	calldata, err := ics26ABI.Pack(method, args...)
	s.Require().NoError(err)

	ics26Address := ethcommon.HexToAddress(s.chainB.contractAddresses.Ics26Router)
	_, err = s.chainB.eth.RPCClient.CallContract(ctx, goethereum.CallMsg{
		From: crypto.PubkeyToAddress(s.chainB.relayerSubmitter.PublicKey),
		To:   &ics26Address,
		Data: calldata,
	}, nil)
	var dataErr rpc.DataError
	s.Require().ErrorAs(err, &dataErr)
	s.Require().Equal(hexutil.Encode(crypto.Keccak256([]byte("FrozenClientState()"))[:4]), dataErr.ErrorData())
}

func (s *BesuToBesuTestSuite) besuClientState(client *besuqbft.Contract) besumsgs.IBesuLightClientMsgsClientState {
	clientStateBz, err := client.GetClientState(nil)
	s.Require().NoError(err)
	clientState, err := besumsgs.NewBindings().UnpackClientState(clientStateBz)
	s.Require().NoError(err)
	return clientState
}

func (s *BesuToBesuTestSuite) spinUpChain(ctx context.Context, params chainconfig.BesuQBFTParams) besuToBesuChainState {
	network, err := chainconfig.SpinUpBesuQBFT(ctx, params)
	s.Require().NoError(err)

	ethChain, err := ethereum.NewEthereum(ctx, network.RPC, nil, network.Faucet)
	s.Require().NoError(err)

	return besuToBesuChainState{
		network: network,
		eth:     ethChain,
	}
}

func (s *BesuToBesuTestSuite) createUsers(chain *besuToBesuChainState) {
	var err error
	chain.deployer, err = chain.eth.CreateAndFundUser()
	s.Require().NoError(err)
	chain.user, err = chain.eth.CreateAndFundUser()
	s.Require().NoError(err)
	chain.relayerSubmitter, err = chain.eth.CreateAndFundUser()
	s.Require().NoError(err)
}

func (s *BesuToBesuTestSuite) deployContracts(chain *besuToBesuChainState) {
	os.Setenv(testvalues.EnvKeyEthRPC, chain.eth.RPC)

	stdout, err := chain.eth.ForgeScript(chain.deployer, testvalues.E2EDeployScriptPath, "--slow")
	s.Require().NoError(err)

	chain.contractAddresses, err = ethereum.GetEthContractsFromDeployOutput(string(stdout))
	s.Require().NoError(err)

	chain.ics26, err = ics26router.NewContract(ethcommon.HexToAddress(chain.contractAddresses.Ics26Router), chain.eth.RPCClient)
	s.Require().NoError(err)
	chain.ics20, err = ics20transfer.NewContract(ethcommon.HexToAddress(chain.contractAddresses.Ics20Transfer), chain.eth.RPCClient)
	s.Require().NoError(err)
	chain.erc20, err = erc20.NewContract(ethcommon.HexToAddress(chain.contractAddresses.Erc20), chain.eth.RPCClient)
	s.Require().NoError(err)
}

func (s *BesuToBesuTestSuite) startRelayer() {
	config := proofapi.NewConfigBuilder().
		BesuToEth(proofapi.BesuToEthParams{
			SrcChainID:    s.chainA.eth.ChainID.String(),
			DstChainID:    s.chainB.eth.ChainID.String(),
			SrcRPC:        s.chainA.eth.RPC,
			DstRPC:        s.chainB.eth.RPC,
			SrcICS26:      s.chainA.contractAddresses.Ics26Router,
			DstICS26:      s.chainB.contractAddresses.Ics26Router,
			ConsensusType: besuToBesuConsensusTypeQBFT,
		}).
		BesuToEth(proofapi.BesuToEthParams{
			SrcChainID:    s.chainB.eth.ChainID.String(),
			DstChainID:    s.chainA.eth.ChainID.String(),
			SrcRPC:        s.chainB.eth.RPC,
			DstRPC:        s.chainA.eth.RPC,
			SrcICS26:      s.chainB.contractAddresses.Ics26Router,
			DstICS26:      s.chainA.contractAddresses.Ics26Router,
			ConsensusType: besuToBesuConsensusTypeQBFT,
		}).
		Build()

	s.relayerConfigPath = filepath.Join(os.TempDir(), fmt.Sprintf("besu-to-besu-relayer-%d.json", time.Now().UnixNano()))
	s.Require().NoError(config.GenerateConfigFile(s.relayerConfigPath))

	var err error
	s.relayerProcess, err = proofapi.StartProofAPI(s.relayerConfigPath)
	s.Require().NoError(err)
}

func (s *BesuToBesuTestSuite) connectRelayer() {
	var err error
	s.relayerClient, err = proofapi.GetGRPCClient(proofapi.DefaultProofAPIGRPCAddress())
	s.Require().NoError(err)

	infoAB := s.waitForRelayerInfo(s.chainA.eth.ChainID.String(), s.chainB.eth.ChainID.String())
	s.Require().Equal(s.chainA.eth.ChainID.String(), infoAB.SourceChain.ChainId)
	s.Require().Equal(s.chainB.eth.ChainID.String(), infoAB.TargetChain.ChainId)

	infoBA := s.waitForRelayerInfo(s.chainB.eth.ChainID.String(), s.chainA.eth.ChainID.String())
	s.Require().Equal(s.chainB.eth.ChainID.String(), infoBA.SourceChain.ChainId)
	s.Require().Equal(s.chainA.eth.ChainID.String(), infoBA.TargetChain.ChainId)
}

func (s *BesuToBesuTestSuite) waitForRelayerInfo(srcChainID, dstChainID string) *proofapitypes.InfoResponse {
	var (
		info *proofapitypes.InfoResponse
		err  error
	)

	for range 20 {
		info, err = s.relayerClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: srcChainID,
			DstChain: dstChainID,
		})
		if err == nil {
			return info
		}
		time.Sleep(time.Second)
	}

	s.Require().NoError(err)
	return info
}

func (s *BesuToBesuTestSuite) createAndRegisterBesuClient(
	srcChain *besuToBesuChainState,
	dstChain *besuToBesuChainState,
	dstClientID string,
	counterpartyClientID string,
) ethcommon.Address {
	resp, err := s.relayerClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
		SrcChain: srcChain.eth.ChainID.String(),
		DstChain: dstChain.eth.ChainID.String(),
		Parameters: map[string]string{
			testvalues.ParameterKey_TrustingPeriod: strconv.Itoa(testvalues.DefaultTrustPeriod),
			testvalues.ParameterKey_MaxClockDrift:  strconv.Itoa(testvalues.DefaultMaxClockDrift),
			testvalues.ParameterKey_RoleManager:    dstChain.contractAddresses.Ics26Router,
		},
	})
	s.Require().NoError(err)
	s.Require().NotEmpty(resp.Tx)

	createClientReceipt, err := dstChain.eth.BroadcastTx(context.Background(), dstChain.relayerSubmitter, 15_000_000, nil, resp.Tx)
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, createClientReceipt.Status)
	s.Require().NotEqual(ethcommon.Address{}, createClientReceipt.ContractAddress)

	counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
		ClientId:     counterpartyClientID,
		MerklePrefix: [][]byte{[]byte("")},
	}
	addClientTx, err := dstChain.ics26.AddClient(
		s.mustTransactOpts(dstChain, dstChain.deployer),
		dstClientID,
		counterpartyInfo,
		createClientReceipt.ContractAddress,
	)
	s.Require().NoError(err)

	addClientReceipt, err := dstChain.eth.GetTxReciept(context.Background(), addClientTx.Hash())
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, addClientReceipt.Status)

	addClientEvent, err := e2esuite.GetEvmEvent(addClientReceipt, dstChain.ics26.ParseICS02ClientAdded)
	s.Require().NoError(err)
	s.Require().Equal(dstClientID, addClientEvent.ClientId)
	s.Require().Equal(createClientReceipt.ContractAddress, addClientEvent.Client)

	registeredClient, err := dstChain.ics26.GetClient(nil, dstClientID)
	s.Require().NoError(err)
	s.Require().Equal(createClientReceipt.ContractAddress, registeredClient)

	return createClientReceipt.ContractAddress
}

func (s *BesuToBesuTestSuite) mustTransactOpts(chain *besuToBesuChainState, key *ecdsa.PrivateKey) *bind.TransactOpts {
	txOpts, err := chain.eth.GetTransactOpts(key)
	s.Require().NoError(err)
	return txOpts
}
