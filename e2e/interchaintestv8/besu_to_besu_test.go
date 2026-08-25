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

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"

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
	besuToBesuChainAID = 1337
	besuToBesuChainBID = 1338

	besuToBesuClientOnA = "besu-chain-b"
	besuToBesuClientOnB = "besu-chain-a"

	besuToBesuConsensusTypeQBFT = "qbft"
)

var (
	besuToBesuChainAIPs = [4]string{"10.42.0.2", "10.42.0.3", "10.42.0.4", "10.42.0.5"}
	besuToBesuChainBIPs = [4]string{"10.43.0.2", "10.43.0.3", "10.43.0.4", "10.43.0.5"}
)

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

	s.chainA = s.spinUpChain(ctx, chainconfig.BesuQBFTParams{
		ChainID:      besuToBesuChainAID,
		Subnet:       "10.42.0.0/16",
		Gateway:      "10.42.0.1",
		ValidatorIPs: besuToBesuChainAIPs,
	})

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

	s.createAndRegisterBesuClient(&s.chainB, &s.chainA, besuToBesuClientOnA, besuToBesuClientOnB)
	s.createAndRegisterBesuClient(&s.chainA, &s.chainB, besuToBesuClientOnB, besuToBesuClientOnA)
}

func (s *BesuToBesuTestSuite) Test_Deploy() {
	transferOnA, err := s.chainA.ics26.GetIBCApp(nil, transfertypes.PortID)
	s.Require().NoError(err)
	s.Require().Equal(strings.ToLower(s.chainA.contractAddresses.Ics20Transfer), strings.ToLower(transferOnA.Hex()))

	transferOnB, err := s.chainB.ics26.GetIBCApp(nil, transfertypes.PortID)
	s.Require().NoError(err)
	s.Require().Equal(strings.ToLower(s.chainB.contractAddresses.Ics20Transfer), strings.ToLower(transferOnB.Hex()))

	clientOnA, err := s.chainA.ics26.GetClient(nil, besuToBesuClientOnA)
	s.Require().NoError(err)
	s.Require().Equal(s.chainA.clientAddress, clientOnA)
	clientOnB, err := s.chainB.ics26.GetClient(nil, besuToBesuClientOnB)
	s.Require().NoError(err)
	s.Require().Equal(s.chainB.clientAddress, clientOnB)
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

	fundTx, err := s.chainA.erc20.Transfer(s.mustTransactOpts(&s.chainA, s.chainA.eth.Faucet), userAddressA, testvalues.StartingERC20Balance)
	s.Require().NoError(err)
	fundReceipt, err := s.chainA.eth.GetTxReciept(ctx, fundTx.Hash())
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, fundReceipt.Status)

	approveTx, err := s.chainA.erc20.Approve(s.mustTransactOpts(&s.chainA, s.chainA.user), ics20AddressA, transferAmount)
	s.Require().NoError(err)
	approveReceipt, err := s.chainA.eth.GetTxReciept(ctx, approveTx.Hash())
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, approveReceipt.Status)

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
	sendReceipt, err := s.chainA.eth.GetTxReciept(ctx, sendTx.Hash())
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, sendReceipt.Status)

	sendEvent, err := e2esuite.GetEvmEvent(sendReceipt, s.chainA.ics26.ParseSendPacket)
	s.Require().NoError(err)

	escrowAddress, err := s.chainA.ics20.GetEscrow(nil, besuToBesuClientOnA)
	s.Require().NoError(err)
	escrowBalance, err := s.chainA.erc20.BalanceOf(nil, escrowAddress)
	s.Require().NoError(err)
	s.Require().Equal(0, transferAmount.Cmp(escrowBalance))

	userBalanceA, err := s.chainA.erc20.BalanceOf(nil, userAddressA)
	s.Require().NoError(err)
	expectedBalanceA := new(big.Int).Sub(new(big.Int).Set(testvalues.StartingERC20Balance), transferAmount)
	s.Require().Equal(0, expectedBalanceA.Cmp(userBalanceA))

	relayAB, err := s.relayerClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
		SrcChain:    s.chainA.eth.ChainID.String(),
		DstChain:    s.chainB.eth.ChainID.String(),
		SourceTxIds: [][]byte{sendTx.Hash().Bytes()},
		SrcClientId: besuToBesuClientOnA,
		DstClientId: besuToBesuClientOnB,
	})
	s.Require().NoError(err)
	s.Require().NotEmpty(relayAB.Tx)
	s.Require().Equal(strings.ToLower(s.chainB.contractAddresses.Ics26Router), strings.ToLower(relayAB.Address))

	recvReceipt, err := s.chainB.eth.BroadcastTx(ctx, s.chainB.relayerSubmitter, 15_000_000, &ics26AddressB, relayAB.Tx)
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
	ibcERC20OnB, err := ibcerc20.NewContract(ibcERC20AddressOnB, s.chainB.eth.RPCClient)
	s.Require().NoError(err)

	userBalanceB, err := ibcERC20OnB.BalanceOf(nil, userAddressB)
	s.Require().NoError(err)
	s.Require().Equal(0, transferAmount.Cmp(userBalanceB))

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

	ackReceipt, err := s.chainA.eth.BroadcastTx(ctx, s.chainA.relayerSubmitter, 15_000_000, &ics26AddressA, ackRelay.Tx)
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, ackReceipt.Status)

	_, err = e2esuite.GetEvmEvent(ackReceipt, s.chainA.ics26.ParseAckPacket)
	s.Require().NoError(err)

	if s.besuFixtureGenerator.Enabled {
		sendHeight := sendReceipt.BlockNumber.Uint64()
		s.Require().Greater(sendHeight, uint64(2))
		s.Require().Greater(ackReceipt.BlockNumber.Uint64(), sendHeight)
		s.Require().NoError(s.besuFixtureGenerator.GenerateAndSaveQBFTFixture(ctx, e2etypes.GenerateQBFTFixtureParams{
			SourceChain:             &s.chainA.eth,
			RouterAddress:           ics26AddressA,
			Packet:                  sendEvent.Packet,
			InitialTrustedHeight:    sendHeight - 2,
			AdjacentUpdateHeight:    sendHeight - 1,
			NonAdjacentUpdateHeight: sendHeight,
			SyntheticSourceHeight:   ackReceipt.BlockNumber.Uint64(),
			TrustingPeriod:          uint64(testvalues.DefaultTrustPeriod),
			MaxClockDrift:           uint64(testvalues.DefaultMaxClockDrift),
		}))
	}
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
		BesuToBesu(proofapi.BesuToBesuParams{
			SrcChainID:    s.chainA.eth.ChainID.String(),
			DstChainID:    s.chainB.eth.ChainID.String(),
			SrcRPC:        s.chainA.eth.RPC,
			DstRPC:        s.chainB.eth.RPC,
			SrcICS26:      s.chainA.contractAddresses.Ics26Router,
			DstICS26:      s.chainB.contractAddresses.Ics26Router,
			ConsensusType: besuToBesuConsensusTypeQBFT,
		}).
		BesuToBesu(proofapi.BesuToBesuParams{
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
) {
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

	dstChain.clientAddress = createClientReceipt.ContractAddress
}

func (s *BesuToBesuTestSuite) mustTransactOpts(chain *besuToBesuChainState, key *ecdsa.PrivateKey) *bind.TransactOpts {
	txOpts, err := chain.eth.GetTransactOpts(key)
	s.Require().NoError(err)
	return txOpts
}
