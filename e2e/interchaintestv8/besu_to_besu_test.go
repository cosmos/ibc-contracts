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
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ibcflow"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	e2etypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
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
	*ibcflow.EVMEndpoint

	network       chainconfig.BesuQBFTChain
	deployer      *ecdsa.PrivateKey
	clientAddress ethcommon.Address
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

	s.deployContracts(&s.chainA, besuToBesuClientOnA)
	s.deployContracts(&s.chainB, besuToBesuClientOnB)

	s.startRelayer()
	s.connectRelayer()

	s.chainA.clientAddress = s.createAndRegisterBesuClient(&s.chainB, &s.chainA, besuToBesuClientOnA, besuToBesuClientOnB)
	s.chainB.clientAddress = s.createAndRegisterBesuClient(&s.chainA, &s.chainB, besuToBesuClientOnB, besuToBesuClientOnA)
}

func (s *BesuToBesuTestSuite) Test_Deploy() {
	s.Require().True(s.Run("Verify ICS26 on Chain A", func() {
		transferOnA, err := s.chainA.ICS26.GetIBCApp(nil, transfertypes.PortID)
		s.Require().NoError(err)
		s.Require().Equal(strings.ToLower(s.chainA.Contracts.Ics20Transfer), strings.ToLower(transferOnA.Hex()))
	}))

	s.Require().True(s.Run("Verify ICS26 on Chain B", func() {
		transferOnB, err := s.chainB.ICS26.GetIBCApp(nil, transfertypes.PortID)
		s.Require().NoError(err)
		s.Require().Equal(strings.ToLower(s.chainB.Contracts.Ics20Transfer), strings.ToLower(transferOnB.Hex()))
	}))

	s.Require().True(s.Run("Verify Besu client on Chain A", func() {
		clientOnA, err := s.chainA.ICS26.GetClient(nil, besuToBesuClientOnA)
		s.Require().NoError(err)
		s.Require().Equal(s.chainA.clientAddress, clientOnA)

		counterparty, err := s.chainA.ICS26.GetCounterparty(nil, besuToBesuClientOnA)
		s.Require().NoError(err)
		s.Require().Equal(besuToBesuClientOnB, counterparty.ClientId)
	}))

	s.Require().True(s.Run("Verify Besu client on Chain B", func() {
		clientOnB, err := s.chainB.ICS26.GetClient(nil, besuToBesuClientOnB)
		s.Require().NoError(err)
		s.Require().Equal(s.chainB.clientAddress, clientOnB)

		counterparty, err := s.chainB.ICS26.GetCounterparty(nil, besuToBesuClientOnB)
		s.Require().NoError(err)
		s.Require().Equal(besuToBesuClientOnA, counterparty.ClientId)
	}))

	s.Require().True(s.Run("Verify Proof API Info A->B", func() {
		info := s.waitForRelayerInfo(s.chainA.Chain.ChainID.String(), s.chainB.Chain.ChainID.String())
		s.Require().Equal(s.chainA.Chain.ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(s.chainB.Chain.ChainID.String(), info.TargetChain.ChainId)
	}))

	s.Require().True(s.Run("Verify Proof API Info B->A", func() {
		info := s.waitForRelayerInfo(s.chainB.Chain.ChainID.String(), s.chainA.Chain.ChainID.String())
		s.Require().Equal(s.chainB.Chain.ChainID.String(), info.SourceChain.ChainId)
		s.Require().Equal(s.chainA.Chain.ChainID.String(), info.TargetChain.ChainId)
	}))
}

func (s *BesuToBesuTestSuite) Test_ICS20TransferERC20FromChainAToChainB() {
	ctx := context.Background()

	s.Require().True(s.Run("Fund user on Chain A", func() {
		s.fundUser(ctx, &s.chainA, testvalues.StartingERC20Balance)
	}))

	res := ibcflow.TransferWithAck(ctx, s.T(), s.relayerClient, s.chainA.EVMEndpoint, s.chainB.EVMEndpoint, big.NewInt(testvalues.TransferAmount), 1)

	if s.besuFixtureGenerator.Enabled {
		s.Require().True(s.Run("Generate QBFT fixture", func() {
			sendHeight := res.SendReceipts[0].BlockNumber.Uint64()
			s.Require().Greater(sendHeight, uint64(2))
			s.Require().Greater(res.AckReceipt.BlockNumber.Uint64(), sendHeight)
			s.Require().NoError(s.besuFixtureGenerator.GenerateAndSaveQBFTFixture(ctx, e2etypes.GenerateQBFTFixtureParams{
				SourceChain:             s.chainA.Chain,
				RouterAddress:           ethcommon.HexToAddress(s.chainA.Contracts.Ics26Router),
				Packet:                  res.Packets[0],
				InitialTrustedHeight:    sendHeight - 2,
				AdjacentUpdateHeight:    sendHeight - 1,
				NonAdjacentUpdateHeight: sendHeight,
				SyntheticSourceHeight:   res.AckReceipt.BlockNumber.Uint64(),
				TrustingPeriod:          uint64(testvalues.DefaultTrustPeriod),
				MaxClockDrift:           uint64(testvalues.DefaultMaxClockDrift),
			}))
		}))
	}
}

func (s *BesuToBesuTestSuite) Test_TimeoutICS20TransferERC20FromChainAToChainB() {
	ctx := context.Background()
	transferAmount := big.NewInt(testvalues.TransferAmount)

	s.Require().True(s.Run("Fund user on Chain A", func() {
		s.fundUser(ctx, &s.chainA, transferAmount)
	}))

	ibcflow.TimeoutFromA(ctx, s.T(), s.relayerClient, s.chainA.EVMEndpoint, s.chainB.EVMEndpoint, transferAmount)
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
		honestUpdate, err := e2etypes.BuildQBFTUpdate(ctx, s.chainA.Chain, trustedHeight, height)
		s.Require().NoError(err)
		s.requireUpdateResult(s.updateClient(ctx, doubleSignClientID, honestUpdate), 0) // Update

		trustedHash, err = client.GetConsensusStateHash(nil, height)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Build conflicting header at the stored height", func() {
		var err error
		updateMsg, err = e2etypes.BuildQBFTDoubleSignUpdate(ctx, s.chainA.Chain, trustedHeight, height)
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

		s.Require().True(s.besuClientState(client).IsFrozen)
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

		state2, err := e2etypes.FetchQBFTConsensusState(ctx, s.chainA.Chain, height2)
		s.Require().NoError(err)
		timestamp = state2.Timestamp

		honestUpdate, err := e2etypes.BuildQBFTUpdate(ctx, s.chainA.Chain, trustedHeight, height2)
		s.Require().NoError(err)
		s.requireUpdateResult(s.updateClient(ctx, timeMisbehaviourClientID, honestUpdate), 0) // Update

		// Height1 is re-sealed with the timestamp of height2.
		forgedUpdate, err := e2etypes.BuildQBFTTimestampUpdate(ctx, s.chainA.Chain, trustedHeight, height1, timestamp)
		s.Require().NoError(err)
		s.requireUpdateResult(s.updateClient(ctx, timeMisbehaviourClientID, forgedUpdate), 0) // Update

		state1, err := e2etypes.FetchQBFTConsensusState(ctx, s.chainA.Chain, height1)
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
		s.Require().False(s.besuClientState(client).IsFrozen)
	}))

	s.Require().True(s.Run("Submit misbehaviour and freeze the client", func() {
		tx, err := s.chainB.ICS26.SubmitMisbehaviour(
			s.mustTransactOpts(&s.chainB, s.chainB.RelayerSubmitterKey), timeMisbehaviourClientID, misbehaviourMsg,
		)
		s.Require().NoError(err)
		receipt, err := s.chainB.Chain.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		submittedEvent, err := e2esuite.GetEvmEvent(receipt, s.chainB.ICS26.ParseICS02MisbehaviourSubmitted)
		s.Require().NoError(err)
		s.Require().Equal(timeMisbehaviourClientID, submittedEvent.ClientId)

		timeEvent, err := e2esuite.GetEvmEvent(receipt, client.ParseTimeNonMonotonicity)
		s.Require().NoError(err)
		s.Require().Equal(height2, timeEvent.Height2)
		s.Require().Equal(height1, timeEvent.Height1)
		s.Require().Equal(timestamp, timeEvent.Timestamp2)
		s.Require().Equal(timestamp, timeEvent.Timestamp1)

		s.Require().True(s.besuClientState(client).IsFrozen)
	}))

	s.Require().True(s.Run("Frozen client rejects further misbehaviour", func() {
		s.requireFrozenClientRevert(ctx, "submitMisbehaviour", timeMisbehaviourClientID, misbehaviourMsg)
	}))
}

// createDedicatedBesuClient registers a Chain A client on Chain B that the relayer does not use.
func (s *BesuToBesuTestSuite) createDedicatedBesuClient(clientID string) *besuqbft.Contract {
	clientAddress := s.createAndRegisterBesuClient(&s.chainA, &s.chainB, clientID, besuToBesuClientOnA)
	client, err := besuqbft.NewContract(clientAddress, s.chainB.Chain.RPCClient)
	s.Require().NoError(err)
	s.Require().False(s.besuClientState(client).IsFrozen)
	return client
}

func (s *BesuToBesuTestSuite) waitForBlock(ctx context.Context, chain *besuToBesuChainState, height uint64) {
	s.Require().NoError(testutil.WaitForCondition(time.Minute, time.Second, func() (bool, error) {
		latest, err := chain.Chain.RPCClient.BlockNumber(ctx)
		return latest >= height, err
	}))
}

// updateClient submits updateMsg to clientID on Chain B through ICS26 and returns the successful receipt.
func (s *BesuToBesuTestSuite) updateClient(ctx context.Context, clientID string, updateMsg []byte) *ethtypes.Receipt {
	tx, err := s.chainB.ICS26.UpdateClient(s.mustTransactOpts(&s.chainB, s.chainB.RelayerSubmitterKey), clientID, updateMsg)
	s.Require().NoError(err)
	receipt, err := s.chainB.Chain.GetTxReciept(ctx, tx.Hash())
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	return receipt
}

// requireUpdateResult asserts the ILightClientMsgs.UpdateResult emitted in receipt.
func (s *BesuToBesuTestSuite) requireUpdateResult(receipt *ethtypes.Receipt, result uint8) {
	updatedEvent, err := e2esuite.GetEvmEvent(receipt, s.chainB.ICS26.ParseICS02ClientUpdated)
	s.Require().NoError(err)
	s.Require().Equal(result, updatedEvent.Result)
}

// requireFrozenClientRevert asserts that calling the ICS26 method on Chain B reverts with FrozenClientState.
func (s *BesuToBesuTestSuite) requireFrozenClientRevert(ctx context.Context, method string, args ...any) {
	ics26ABI, err := ics26router.ContractMetaData.GetAbi()
	s.Require().NoError(err)
	calldata, err := ics26ABI.Pack(method, args...)
	s.Require().NoError(err)

	ics26Address := ethcommon.HexToAddress(s.chainB.Contracts.Ics26Router)
	_, err = s.chainB.Chain.RPCClient.CallContract(ctx, goethereum.CallMsg{
		From: crypto.PubkeyToAddress(s.chainB.RelayerSubmitterKey.PublicKey),
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
		EVMEndpoint: &ibcflow.EVMEndpoint{Chain: &ethChain},
		network:     network,
	}
}

func (s *BesuToBesuTestSuite) createUsers(chain *besuToBesuChainState) {
	var err error
	chain.deployer, err = chain.Chain.CreateAndFundUser()
	s.Require().NoError(err)
	chain.UserKey, err = chain.Chain.CreateAndFundUser()
	s.Require().NoError(err)
	chain.RelayerSubmitterKey, err = chain.Chain.CreateAndFundUser()
	s.Require().NoError(err)
}

func (s *BesuToBesuTestSuite) deployContracts(chain *besuToBesuChainState, clientID string) {
	os.Setenv(testvalues.EnvKeyEthRPC, chain.Chain.RPC)

	stdout, err := chain.Chain.ForgeScript(chain.deployer, testvalues.E2EDeployScriptPath, "--slow")
	s.Require().NoError(err)

	contracts, err := ethereum.GetEthContractsFromDeployOutput(string(stdout))
	s.Require().NoError(err)

	chain.EVMEndpoint = ibcflow.NewEVMEndpoint(s.T(), chain.Chain, contracts, clientID, chain.UserKey, chain.RelayerSubmitterKey)
}

func (s *BesuToBesuTestSuite) fundUser(ctx context.Context, chain *besuToBesuChainState, amount *big.Int) {
	fundTx, err := chain.ERC20.Transfer(s.mustTransactOpts(chain, chain.Chain.Faucet), chain.UserAddress(), amount)
	s.Require().NoError(err)

	fundReceipt, err := chain.Chain.GetTxReciept(ctx, fundTx.Hash())
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, fundReceipt.Status)
}

func (s *BesuToBesuTestSuite) startRelayer() {
	config := proofapi.NewConfigBuilder().
		BesuToEth(proofapi.BesuToEthParams{
			SrcChainID:    s.chainA.Chain.ChainID.String(),
			DstChainID:    s.chainB.Chain.ChainID.String(),
			SrcRPC:        s.chainA.Chain.RPC,
			DstRPC:        s.chainB.Chain.RPC,
			SrcICS26:      s.chainA.Contracts.Ics26Router,
			DstICS26:      s.chainB.Contracts.Ics26Router,
			ConsensusType: besuToBesuConsensusTypeQBFT,
		}).
		BesuToEth(proofapi.BesuToEthParams{
			SrcChainID:    s.chainB.Chain.ChainID.String(),
			DstChainID:    s.chainA.Chain.ChainID.String(),
			SrcRPC:        s.chainB.Chain.RPC,
			DstRPC:        s.chainA.Chain.RPC,
			SrcICS26:      s.chainB.Contracts.Ics26Router,
			DstICS26:      s.chainA.Contracts.Ics26Router,
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

	infoAB := s.waitForRelayerInfo(s.chainA.Chain.ChainID.String(), s.chainB.Chain.ChainID.String())
	s.Require().Equal(s.chainA.Chain.ChainID.String(), infoAB.SourceChain.ChainId)
	s.Require().Equal(s.chainB.Chain.ChainID.String(), infoAB.TargetChain.ChainId)

	infoBA := s.waitForRelayerInfo(s.chainB.Chain.ChainID.String(), s.chainA.Chain.ChainID.String())
	s.Require().Equal(s.chainB.Chain.ChainID.String(), infoBA.SourceChain.ChainId)
	s.Require().Equal(s.chainA.Chain.ChainID.String(), infoBA.TargetChain.ChainId)
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
		SrcChain: srcChain.Chain.ChainID.String(),
		DstChain: dstChain.Chain.ChainID.String(),
		Parameters: map[string]string{
			testvalues.ParameterKey_TrustingPeriod: strconv.Itoa(testvalues.DefaultTrustPeriod),
			testvalues.ParameterKey_MaxClockDrift:  strconv.Itoa(testvalues.DefaultMaxClockDrift),
			testvalues.ParameterKey_TrustLevel:     "2/3",
			testvalues.ParameterKey_RoleManager:    dstChain.Contracts.Ics26Router,
		},
	})
	s.Require().NoError(err)
	s.Require().NotEmpty(resp.Tx)

	createClientReceipt, err := dstChain.Chain.BroadcastTx(context.Background(), dstChain.RelayerSubmitterKey, 15_000_000, nil, resp.Tx)
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, createClientReceipt.Status)
	s.Require().NotEqual(ethcommon.Address{}, createClientReceipt.ContractAddress)

	counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
		ClientId:     counterpartyClientID,
		MerklePrefix: [][]byte{[]byte("")},
	}
	addClientTx, err := dstChain.ICS26.AddClient(
		s.mustTransactOpts(dstChain, dstChain.deployer),
		dstClientID,
		counterpartyInfo,
		createClientReceipt.ContractAddress,
	)
	s.Require().NoError(err)

	addClientReceipt, err := dstChain.Chain.GetTxReciept(context.Background(), addClientTx.Hash())
	s.Require().NoError(err)
	s.Require().Equal(ethtypes.ReceiptStatusSuccessful, addClientReceipt.Status)

	addClientEvent, err := e2esuite.GetEvmEvent(addClientReceipt, dstChain.ICS26.ParseICS02ClientAdded)
	s.Require().NoError(err)
	s.Require().Equal(dstClientID, addClientEvent.ClientId)
	s.Require().Equal(createClientReceipt.ContractAddress, addClientEvent.Client)

	registeredClient, err := dstChain.ICS26.GetClient(nil, dstClientID)
	s.Require().NoError(err)
	s.Require().Equal(createClientReceipt.ContractAddress, registeredClient)

	return createClientReceipt.ContractAddress
}

func (s *BesuToBesuTestSuite) mustTransactOpts(chain *besuToBesuChainState, key *ecdsa.PrivateKey) *bind.TransactOpts {
	txOpts, err := chain.Chain.GetTransactOpts(key)
	s.Require().NoError(err)
	return txOpts
}
