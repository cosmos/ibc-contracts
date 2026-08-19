// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"
	"crypto/ecdsa"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"strings"
	"testing"
	"time"
	"unicode"

	"github.com/cosmos/gogoproto/proto"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	ethtypes "github.com/ethereum/go-ethereum/core/types"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	govtypesv1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"

	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cometproto "github.com/cometbft/cometbft/proto/tendermint/types"
	comettypes "github.com/cometbft/cometbft/types"
	comettime "github.com/cometbft/cometbft/types/time"

	ibcwasmtypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/v11/types"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"
	tmclient "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"

	"github.com/cosmos/interchaintest/v11"
	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"
	"github.com/cosmos/interchaintest/v11/testutil"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	ibctesting "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ibctesting"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	ethereumtypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/ethereum"
)

// broadcastMessages is the shared broadcast path. Pass waitBlocks=0 to skip the post-inclusion wait.
func (s *TestSuite) broadcastMessages(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, gas uint64, waitBlocks int, msgs ...sdk.Msg) (*sdk.TxResponse, error) {
	sdk.GetConfig().SetBech32PrefixForAccount(chain.Config().Bech32Prefix, chain.Config().Bech32Prefix+sdk.PrefixPublic)
	sdk.GetConfig().SetBech32PrefixForValidator(
		chain.Config().Bech32Prefix+sdk.PrefixValidator+sdk.PrefixOperator,
		chain.Config().Bech32Prefix+sdk.PrefixValidator+sdk.PrefixOperator+sdk.PrefixPublic,
	)
	sdk.GetConfig().SetBech32PrefixForConsensusNode(chain.Config().Bech32Prefix+sdk.PrefixValidator+sdk.PrefixConsensus, chain.Config().Bech32Prefix+sdk.PrefixValidator+sdk.PrefixConsensus+sdk.PrefixPublic)

	broadcaster := cosmos.NewBroadcaster(s.T(), chain)

	broadcaster.ConfigureClientContextOptions(func(clientContext client.Context) client.Context {
		return clientContext.
			WithCodec(chain.Config().EncodingConfig.Codec).
			WithChainID(chain.Config().ChainID).
			WithTxConfig(chain.Config().EncodingConfig.TxConfig)
	})

	broadcaster.ConfigureFactoryOptions(func(factory tx.Factory) tx.Factory {
		return factory.WithGas(gas)
	})

	resp, err := cosmos.BroadcastTx(ctx, broadcaster, user, msgs...)
	if err != nil {
		return nil, err
	}

	if waitBlocks > 0 {
		s.Require().NoError(testutil.WaitForBlocks(ctx, waitBlocks, chain))
	}

	if resp.Code != 0 {
		return nil, fmt.Errorf("tx failed with code %d: %s", resp.Code, resp.RawLog)
	}

	return &resp, nil
}

// BroadcastMessages broadcasts the provided messages to the given chain and signs them on behalf of the provided user.
// Once the broadcast response is returned, we wait for two blocks to be created on chain.
func (s *TestSuite) BroadcastMessages(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, gas uint64, msgs ...sdk.Msg) (*sdk.TxResponse, error) {
	return s.broadcastMessages(ctx, chain, user, gas, 2, msgs...)
}

// BroadcastMessagesNoWait is like BroadcastMessages but skips the 2-block wait after inclusion.
func (s *TestSuite) BroadcastMessagesNoWait(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, gas uint64, msgs ...sdk.Msg) (*sdk.TxResponse, error) {
	return s.broadcastMessages(ctx, chain, user, gas, 0, msgs...)
}

// CreateAndFundCosmosUser returns a new cosmos user with the initial balance and funds it with the native chain denom.
func (s *TestSuite) CreateAndFundCosmosUser(ctx context.Context, chain *cosmos.CosmosChain) ibc.Wallet {
	return s.CreateAndFundCosmosUserWithBalance(ctx, chain, testvalues.InitialBalance)
}

// CreateAndFundCosmosUserWithBalance returns a new cosmos user with the given balance and funds it with the native chain denom.
func (s *TestSuite) CreateAndFundCosmosUserWithBalance(ctx context.Context, chain *cosmos.CosmosChain, balance int64) ibc.Wallet {
	cosmosUserFunds := sdkmath.NewInt(balance)
	cosmosUsers := interchaintest.GetAndFundTestUsers(s.T(), ctx, s.T().Name(), cosmosUserFunds, chain)

	return cosmosUsers[0]
}

// GetEvmEvent parses the logs in the given receipt and returns the first event that can be parsed
func GetEvmEvent[T any](receipt *ethtypes.Receipt, parseFn func(log ethtypes.Log) (*T, error)) (event *T, err error) {
	for _, l := range receipt.Logs {
		event, err = parseFn(*l)
		if err == nil && event != nil {
			break
		}
	}

	if event == nil {
		err = fmt.Errorf("event not found")
	}

	return event, err
}

func (s *TestSuite) GetTransactOpts(key *ecdsa.PrivateKey, chain *ethereum.Ethereum) *bind.TransactOpts {
	opts, err := chain.GetTransactOpts(key)
	s.Require().NoError(err)
	return opts
}

// PushNewWasmClientProposal submits a new wasm client governance proposal to the chain.
func (s *TestSuite) PushNewWasmClientProposal(ctx context.Context, chain *cosmos.CosmosChain, wallet ibc.Wallet, proposalContentReader io.Reader) string {
	zippedContent, err := io.ReadAll(proposalContentReader)
	s.Require().NoError(err)

	computedChecksum := s.extractChecksumFromGzippedContent(zippedContent)

	moduleAddr, err := chain.AuthQueryModuleAddress(ctx, govtypes.ModuleName)
	s.Require().NoError(err)
	message := ibcwasmtypes.MsgStoreCode{
		Signer:       moduleAddr,
		WasmByteCode: zippedContent,
	}

	err = s.ExecuteGovV1Proposal(ctx, &message, chain, wallet)
	s.Require().NoError(err)

	codeResp, err := GRPCQuery[ibcwasmtypes.QueryCodeResponse](ctx, chain, &ibcwasmtypes.QueryCodeRequest{Checksum: computedChecksum})
	s.Require().NoError(err)

	checksumBz := codeResp.Data
	checksum32 := sha256.Sum256(checksumBz)
	actualChecksum := hex.EncodeToString(checksum32[:])
	s.Require().Equal(computedChecksum, actualChecksum, "checksum returned from query did not match the computed checksum")

	return actualChecksum
}

// PushMigrateWasmClientProposal submits a new wasm client governance proposal to the chain.
func (s *TestSuite) PushMigrateWasmClientProposal(ctx context.Context, chain *cosmos.CosmosChain, wallet ibc.Wallet, clientId string, checksum string, migrateMsg []byte) {
	checksumBz, err := hex.DecodeString(checksum)
	s.Require().NoError(err)

	message := ibcwasmtypes.MsgMigrateContract{
		Signer:   authtypes.NewModuleAddress(govtypes.ModuleName).String(),
		ClientId: clientId,
		Checksum: checksumBz,
		Msg:      migrateMsg,
	}

	err = s.ExecuteGovV1Proposal(ctx, &message, chain, wallet)
	s.Require().NoError(err)
}

// extractChecksumFromGzippedContent takes a gzipped wasm contract and returns the checksum.
func (s *TestSuite) extractChecksumFromGzippedContent(zippedContent []byte) string {
	content, err := ibcwasmtypes.Uncompress(zippedContent, ibcwasmtypes.MaxWasmSize)
	s.Require().NoError(err)

	checksum32 := sha256.Sum256(content)
	return hex.EncodeToString(checksum32[:])
}

// ExecuteGovV1Proposal submits a v1 governance proposal using the provided user and message and uses all validators
// to vote yes on the proposal.
func (s *TestSuite) ExecuteGovV1Proposal(ctx context.Context, msg sdk.Msg, cosmosChain *cosmos.CosmosChain, user ibc.Wallet) error {
	proposalID := s.Cosmos.proposalIDs[cosmosChain.Config().ChainID]
	defer func() {
		s.Cosmos.proposalIDs[cosmosChain.Config().ChainID] = proposalID + 1
	}()

	msgs := []sdk.Msg{msg}

	msgSubmitProposal, err := govtypesv1.NewMsgSubmitProposal(
		msgs,
		sdk.NewCoins(sdk.NewCoin(cosmosChain.Config().Denom, govtypesv1.DefaultMinDepositTokens)),
		user.FormattedAddress(),
		"",
		fmt.Sprintf("e2e gov proposal: %d", proposalID),
		fmt.Sprintf("executing gov proposal %d", proposalID),
		false,
	)
	s.Require().NoError(err)

	_, err = s.BroadcastMessages(ctx, cosmosChain, user, 50_000_000, msgSubmitProposal)
	s.Require().NoError(err)

	s.Require().NoError(cosmosChain.VoteOnProposalAllValidators(ctx, proposalID, cosmos.ProposalVoteYes))

	return s.waitForGovV1ProposalToPass(ctx, cosmosChain, proposalID)
}

// waitForGovV1ProposalToPass polls for the entire voting period to see if the proposal has passed.
// if the proposal has not passed within the duration of the voting period, an error is returned.
func (*TestSuite) waitForGovV1ProposalToPass(ctx context.Context, chain *cosmos.CosmosChain, proposalID uint64) error {
	var govProposal *govtypesv1.Proposal
	// poll for the query for the entire voting period to see if the proposal has passed.
	err := testutil.WaitForCondition(testvalues.VotingPeriod, 10*time.Second, func() (bool, error) {
		proposalResp, err := GRPCQuery[govtypesv1.QueryProposalResponse](ctx, chain, &govtypesv1.QueryProposalRequest{
			ProposalId: proposalID,
		})
		if err != nil {
			return false, err
		}

		govProposal = proposalResp.Proposal
		return govProposal.Status == govtypesv1.StatusPassed, nil
	})

	// in the case of a failed proposal, we wrap the polling error with additional information about why the proposal failed.
	if err != nil && govProposal.FailedReason != "" {
		err = errorsmod.Wrap(err, govProposal.FailedReason)
	}
	return err
}

func IsLowercase(s string) bool {
	for _, r := range s {
		if !unicode.IsLower(r) && unicode.IsLetter(r) {
			return false
		}
	}
	return true
}

func (s *TestSuite) GetEthereumClientState(ctx context.Context, cosmosChain *cosmos.CosmosChain, clientID string) (*ibcwasmtypes.ClientState, ethereumtypes.ClientState) {
	clientStateResp, err := GRPCQuery[clienttypes.QueryClientStateResponse](ctx, cosmosChain, &clienttypes.QueryClientStateRequest{
		ClientId: clientID,
	})
	s.Require().NoError(err)

	var clientState ibcexported.ClientState
	err = cosmosChain.Config().EncodingConfig.InterfaceRegistry.UnpackAny(clientStateResp.ClientState, &clientState)
	s.Require().NoError(err)

	wasmClientState, ok := clientState.(*ibcwasmtypes.ClientState)
	s.Require().True(ok)
	s.Require().NotEmpty(wasmClientState.Data)

	var ethClientState ethereumtypes.ClientState
	err = json.Unmarshal(wasmClientState.Data, &ethClientState)
	s.Require().NoError(err, "failed to unmarshal ethereum client state: %s", string(wasmClientState.Data))

	return wasmClientState, ethClientState
}

func (s *TestSuite) CreateTMClientHeader(
	ctx context.Context,
	chain *cosmos.CosmosChain,
	blockHeight int64,
	timestamp time.Time,
	oldHeader tmclient.Header,
) tmclient.Header {
	var privVals []comettypes.PrivValidator
	var validators []*comettypes.Validator
	for _, chainVal := range chain.Validators {
		keyBz, err := chainVal.ReadFile(ctx, "config/priv_validator_key.json")
		s.Require().NoError(err)
		var privValidatorKeyFile cosmos.PrivValidatorKeyFile
		err = json.Unmarshal(keyBz, &privValidatorKeyFile)
		s.Require().NoError(err)
		decodedKeyBz, err := base64.StdEncoding.DecodeString(privValidatorKeyFile.PrivKey.Value)
		s.Require().NoError(err)

		privKey := ed25519.PrivKey(decodedKeyBz)
		privVal := comettypes.NewMockPVWithParams(privKey, false, false)
		privVals = append(privVals, privVal)

		pubKey, err := privVal.GetPubKey()
		s.Require().NoError(err)

		val := comettypes.NewValidator(pubKey, oldHeader.ValidatorSet.Proposer.VotingPower)
		validators = append(validators, val)

	}

	valSet := comettypes.NewValidatorSet(validators)
	vsetHash := valSet.Hash()

	// Make sure all the signers are in the correct order as expected by the validator set
	signers := make([]comettypes.PrivValidator, valSet.Size())
	for i := range signers {
		_, val := valSet.GetByIndex(int32(i))

		for _, pv := range privVals {
			pk, err := pv.GetPubKey()
			s.Require().NoError(err)

			if pk.Equals(val.PubKey) {
				signers[i] = pv
				break
			}
		}

		if signers[i] == nil {
			s.Require().FailNow("could not find signer for validator")
		}
	}

	tmHeader := comettypes.Header{
		Version:            oldHeader.Header.Version,
		ChainID:            oldHeader.Header.ChainID,
		Height:             blockHeight,
		Time:               timestamp,
		LastBlockID:        ibctesting.MakeBlockID(make([]byte, tmhash.Size), 10_000, make([]byte, tmhash.Size)),
		LastCommitHash:     oldHeader.Header.LastCommitHash,
		DataHash:           tmhash.Sum([]byte("data_hash")),
		ValidatorsHash:     vsetHash,
		NextValidatorsHash: vsetHash,
		ConsensusHash:      tmhash.Sum([]byte("consensus_hash")),
		AppHash:            tmhash.Sum([]byte("app_hash")),
		LastResultsHash:    tmhash.Sum([]byte("last_results_hash")),
		EvidenceHash:       tmhash.Sum([]byte("evidence_hash")),
		ProposerAddress:    valSet.Proposer.Address,
	}

	hhash := tmHeader.Hash()
	blockID := ibctesting.MakeBlockID(hhash, oldHeader.Commit.BlockID.PartSetHeader.Total, tmhash.Sum([]byte("part_set")))
	voteSet := comettypes.NewVoteSet(oldHeader.Header.ChainID, blockHeight, 1, cometproto.PrecommitType, valSet)

	voteProto := &comettypes.Vote{
		ValidatorAddress: nil,
		ValidatorIndex:   -1,
		Height:           blockHeight,
		Round:            1,
		Timestamp:        comettime.Now(),
		Type:             cometproto.PrecommitType,
		BlockID:          blockID,
	}

	for i, sign := range signers {
		pv, err := sign.GetPubKey()
		s.Require().NoError(err)
		addr := pv.Address()
		vote := voteProto.Copy()
		vote.ValidatorAddress = addr
		vote.ValidatorIndex = int32(i)
		_, err = comettypes.SignAndCheckVote(vote, sign, oldHeader.Header.ChainID, false)
		s.Require().NoError(err)
		added, err := voteSet.AddVote(vote)
		s.Require().NoError(err)
		s.Require().True(added)
	}
	extCommit := voteSet.MakeExtendedCommit(comettypes.DefaultABCIParams())
	commit := extCommit.ToCommit()

	signedHeader := &cometproto.SignedHeader{
		Header: tmHeader.ToProto(),
		Commit: commit.ToProto(),
	}

	valSetProto, err := valSet.ToProto()
	s.Require().NoError(err)

	return tmclient.Header{
		SignedHeader:      signedHeader,
		ValidatorSet:      valSetProto,
		TrustedHeight:     oldHeader.TrustedHeight,
		TrustedValidators: oldHeader.TrustedValidators,
	}
}

func (s *TestSuite) GetTopLevelTestName() string {
	parts := strings.Split(s.T().Name(), "/")
	if len(parts) >= 2 {
		return parts[1]
	}

	return s.T().Name()
}

func (s *TestSuite) MustBroadcastSdkTxBody(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, gas uint64, txBodyBz []byte) *sdk.TxResponse {
	resp, err := s.BroadcastSdkTxBody(ctx, chain, user, gas, txBodyBz)
	s.Require().NoError(err)

	return resp
}

func (s *TestSuite) BroadcastSdkTxBody(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, gas uint64, txBodyBz []byte) (*sdk.TxResponse, error) {
	var txBody txtypes.TxBody
	err := proto.Unmarshal(txBodyBz, &txBody)
	s.Require().NoError(err)

	var msgs []sdk.Msg
	for _, msg := range txBody.Messages {
		var sdkMsg sdk.Msg
		err = chain.Config().EncodingConfig.InterfaceRegistry.UnpackAny(msg, &sdkMsg)
		s.Require().NoError(err)

		msgs = append(msgs, sdkMsg)
	}

	s.Require().NotZero(len(msgs))

	return s.BroadcastMessages(ctx, chain, user, gas, msgs...)
}

func (s *TestSuite) BroadcastSdkTxBodyNoWait(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, gas uint64, txBodyBz []byte) (*sdk.TxResponse, error) {
	var txBody txtypes.TxBody
	err := proto.Unmarshal(txBodyBz, &txBody)
	s.Require().NoError(err)

	var msgs []sdk.Msg
	for _, msg := range txBody.Messages {
		var sdkMsg sdk.Msg
		err = chain.Config().EncodingConfig.InterfaceRegistry.UnpackAny(msg, &sdkMsg)
		s.Require().NoError(err)

		msgs = append(msgs, sdkMsg)
	}

	s.Require().NotZero(len(msgs))

	return s.BroadcastMessagesNoWait(ctx, chain, user, gas, msgs...)
}

func (s *TestSuite) MustBroadcastSdkTxBodyNoWait(ctx context.Context, chain *cosmos.CosmosChain, user ibc.Wallet, gas uint64, txBodyBz []byte) *sdk.TxResponse {
	resp, err := s.BroadcastSdkTxBodyNoWait(ctx, chain, user, gas, txBodyBz)
	s.Require().NoError(err)

	return resp
}

// BlockTimeFetcher abstracts reading the latest block timestamp from any chain.
type BlockTimeFetcher interface {
	GetBlockTime(ctx context.Context) (int64, error)
}

// ProofStaleness is the extra time to wait after a timeout is reached on-chain
// before relaying the timeout proof. The relayer builds proofs against recent
// consensus states that may lag behind the actual chain clock by ~12s (attestor
// update interval). Without this buffer the destination chain rejects the proof
// because its counterparty timestamp is still below the timeout.
const ProofStaleness = 15 * time.Second

// WaitForBlockTime polls chain's block time every second until it exceeds
// timeoutTimestamp, returning an error if 2 minutes elapse first.
func WaitForBlockTime(ctx context.Context, t *testing.T, chain BlockTimeFetcher, timeoutTimestamp uint64) error {
	t.Helper()
	start := time.Now()
	return testutil.WaitForCondition(2*time.Minute, time.Second, func() (bool, error) {
		blockTime, err := chain.GetBlockTime(ctx)
		if err != nil {
			return false, nil
		}
		if uint64(blockTime) > timeoutTimestamp {
			t.Logf("Timeout reached after %s (block time: %d > timeout: %d)", time.Since(start).Round(time.Second), blockTime, timeoutTimestamp)
			return true, nil
		}
		return false, nil
	})
}

// StripHTTPPrefix removes http:// or https:// prefix from an endpoint.
func StripHTTPPrefix(endpoint string) string {
	endpoint = strings.TrimPrefix(endpoint, "https://")
	endpoint = strings.TrimPrefix(endpoint, "http://")
	return endpoint
}
