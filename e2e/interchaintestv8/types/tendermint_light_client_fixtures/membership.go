// SPDX-License-Identifier: Apache-2.0

// Package tendermint_light_client_fixtures generates test fixtures for IBC Tendermint light client verification.
//
// IBC Cross-Chain Architecture:
// - ChainA runs an IBC client that tracks ChainB's consensus states
// - ChainB is the counterparty chain whose state we want to verify
// - The client on ChainA stores ChainB's consensus states at various heights
// - To verify data exists on ChainB, we need:
//  1. A consensus state from ChainB (stored on ChainA's client)
//  2. A Merkle proof from ChainB showing the data's existence/non-existence
//  3. Cryptographic verification that the proof matches the consensus state's app hash
//
// This package creates fixtures containing all the components for testing
// membership and non-membership verification scenarios.
package tendermint_light_client_fixtures

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/cosmos/gogoproto/proto"
	ics23 "github.com/cosmos/ics23/go"
	"github.com/stretchr/testify/suite"

	abci "github.com/cometbft/cometbft/abci/types"
	cmtcrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"

	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v11/modules/core/23-commitment/types"
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"
	ibctmtypes "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
)

type KeyPath struct {
	Key        string
	Membership bool
}

type ProofContext struct {
	KeyPath          string
	ExpectMembership bool
	ProofHeight      uint64
	BlockHeight      uint64
	ABCIResponse     *abci.ResponseQuery
	ConsensusState   *ibctmtypes.ConsensusState
}

type MembershipFixtureGenerator struct {
	enabled    bool
	fixtureDir string
	suite      *suite.Suite
}

func NewMembershipFixtureGenerator(enabled bool, fixtureDir string, s *suite.Suite) *MembershipFixtureGenerator {
	return &MembershipFixtureGenerator{
		enabled:    enabled,
		fixtureDir: fixtureDir,
		suite:      s,
	}
}

// GenerateMembershipVerificationScenarios creates IBC light client verification fixtures.
// ChainA: The chain running the IBC client (where consensus states are stored)
// ChainB: The counterparty chain being tracked (where state proofs are queried from)
// clientId: The IBC client ID on ChainA that tracks ChainB's consensus states
func (g *MembershipFixtureGenerator) GenerateMembershipVerificationScenarios(
	ctx context.Context,
	chainA *cosmos.CosmosChain, // IBC client chain - stores consensus states of ChainB
	chainB *cosmos.CosmosChain, // Counterparty chain - source of state proofs
	keyPaths []KeyPath,
	clientId string, // IBC client ID on ChainA tracking ChainB
) {
	if !g.enabled {
		return
	}

	g.suite.T().Log("🔧 Generating membership verification scenarios with predefined keys")

	for i, keySpec := range keyPaths {
		proofType := g.proofTypeNameFor(keySpec.Membership)
		g.suite.T().Logf("🔍 Processing predefined key path: %s (%s)", keySpec.Key, proofType)
		g.generateFixtureForKeyPath(ctx, chainA, chainB, keySpec.Key, i, keySpec.Membership, clientId)
	}

	g.suite.T().Log("✅ Predefined key membership scenarios generated successfully")
}

func (g *MembershipFixtureGenerator) generateFixtureForKeyPath(
	ctx context.Context,
	chainA *cosmos.CosmosChain, // IBC client chain
	chainB *cosmos.CosmosChain, // Counterparty chain
	keyPath string,
	index int,
	expectMembership bool,

	clientId string,
) {
	proofType := g.proofTypeNameFor(expectMembership)
	g.suite.T().Logf("🔧 Generating %s fixture for key: %s", proofType, keyPath)

	// Step 1: Get the latest consensus state of ChainB stored on ChainA's IBC client
	consensusStateHeight, latestConsensusState := g.getLatestConsensusStateHeight(ctx, chainA, clientId)
	g.suite.T().Logf("📊 Using latest consensus state at height %d for proof generation", consensusStateHeight)

	tmConsensusState := g.unmarshalConsensusState(latestConsensusState)

	// In Tendermint, the AppHash in block H+1 reflects the state after executing transactions from block H.
	// Since IBC consensus states are stored at their committed block height, to verify proofs against a
	// consensus state at height H, we must query the blockchain state at height H-1 (before the transactions
	// that led to that consensus state were executed). This ensures the proof's app hash matches the
	// consensus state's root hash for cryptographic verification.
	proofHeight := consensusStateHeight - 1
	g.suite.T().Logf("📍 Querying proof from chainB at height %d", proofHeight)

	// Step 2: Query ChainB for a Merkle proof of the key's existence/non-existence at proofHeight
	abciResp := g.queryStateProofForKey(ctx, chainB, keyPath, proofHeight)
	g.ensureProofMatchesExpectation(abciResp, keyPath, expectMembership)

	merkleProofBytes := g.convertToIBCMerkleProof(abciResp.ProofOps)

	proofCtx := &ProofContext{
		KeyPath:          keyPath,
		ExpectMembership: expectMembership,
		ProofHeight:      proofHeight,
		BlockHeight:      consensusStateHeight,
		ABCIResponse:     abciResp,
		ConsensusState:   tmConsensusState,
	}

	// Step 3: Save the complete fixture with ChainA's client state, ChainB's consensus state, and the proof
	g.saveFixture(ctx, chainA, proofCtx, merkleProofBytes, index, clientId)
}

func (g *MembershipFixtureGenerator) proofTypeNameFor(isMembership bool) string {
	if isMembership {
		return "membership"
	}
	return "non-membership"
}

func (g *MembershipFixtureGenerator) getLatestConsensusStateHeight(
	ctx context.Context,
	chainA *cosmos.CosmosChain, // ChainA - where consensus states are stored
	clientId string,
) (uint64, *clienttypes.ConsensusStateWithHeight) {
	allStatesResp, err := e2esuite.GRPCQuery[clienttypes.QueryConsensusStatesResponse](
		ctx,
		chainA,
		&clienttypes.QueryConsensusStatesRequest{
			ClientId: clientId,
		},
	)
	g.suite.Require().NoError(err)
	g.suite.Require().NotNil(allStatesResp.ConsensusStates, "No consensus states found for client")
	g.suite.Require().Greater(len(allStatesResp.ConsensusStates), 0, "No consensus states found for client")

	latest := g.findHighestRevisionHeight(allStatesResp.ConsensusStates)
	return latest.Height.RevisionHeight, latest
}

func (g *MembershipFixtureGenerator) findHighestRevisionHeight(
	states []clienttypes.ConsensusStateWithHeight,
) *clienttypes.ConsensusStateWithHeight {
	var highest *clienttypes.ConsensusStateWithHeight
	for _, state := range states {
		if highest == nil || state.Height.RevisionHeight > highest.Height.RevisionHeight {
			stateCopy := state
			highest = &stateCopy
		}
	}
	return highest
}

func (g *MembershipFixtureGenerator) queryStateProofForKey(
	ctx context.Context,
	chainB *cosmos.CosmosChain, // ChainB - counterparty chain being proven
	keyPath string,
	proofHeight uint64,
) *abci.ResponseQuery {
	storePath := fmt.Sprintf("store/%s/key", ibcexported.StoreKey)

	abciReq := &abci.RequestQuery{
		Path:   storePath,
		Data:   []byte(keyPath),
		Height: int64(proofHeight),
		Prove:  true,
	}

	g.suite.T().Logf("📡 ABCI Query: path=%s, data=%s, height=%d, prove=true",
		storePath, keyPath, abciReq.Height)

	abciResp, err := e2esuite.ABCIQuery(ctx, chainB, abciReq)
	g.suite.Require().NoError(err)

	return abciResp
}

func (g *MembershipFixtureGenerator) ensureProofMatchesExpectation(
	abciResp *abci.ResponseQuery,
	keyPath string,
	expectMembership bool,
) {
	hasValue := len(abciResp.Value) > 0

	if expectMembership && !hasValue {
		g.suite.T().Fatalf("❌ Expected membership proof for key %s but value is empty", keyPath)
	}
	if !expectMembership && hasValue {
		g.suite.T().Fatalf("❌ Expected non-membership proof for key %s but value exists (length: %d)",
			keyPath, len(abciResp.Value))
	}

	proofType := g.proofTypeNameFor(expectMembership)
	if hasValue {
		g.suite.T().Logf("✅ ABCI query successful - %s case: value length: %d, proof ops: %d",
			proofType, len(abciResp.Value), len(abciResp.ProofOps.Ops))
	} else {
		g.suite.T().Logf("✅ ABCI query successful - %s case: empty value, proof ops: %d",
			proofType, len(abciResp.ProofOps.Ops))
	}

	if len(abciResp.ProofOps.Ops) == 0 {
		g.suite.T().Fatalf("❌ ABCI proof is empty for key: %s", keyPath)
	}
}

func (g *MembershipFixtureGenerator) unmarshalConsensusState(
	consensusStateWithHeight *clienttypes.ConsensusStateWithHeight,
) *ibctmtypes.ConsensusState {
	var tmConsensusState ibctmtypes.ConsensusState
	err := proto.Unmarshal(consensusStateWithHeight.ConsensusState.Value, &tmConsensusState)
	g.suite.Require().NoError(err)

	g.suite.T().Logf("📊 Consensus state root (app hash): %s", hex.EncodeToString(tmConsensusState.Root.GetHash()))

	return &tmConsensusState
}

func (g *MembershipFixtureGenerator) convertToIBCMerkleProof(proofOps *cmtcrypto.ProofOps) []byte {
	commitmentProofs := g.extractCommitmentProofs(proofOps)
	merkleProof := &commitmenttypes.MerkleProof{Proofs: commitmentProofs}

	proofBytes, err := proto.Marshal(merkleProof)
	g.suite.Require().NoError(err)

	g.suite.T().Logf("🔄 Converted %d ABCI ProofOps to IBC MerkleProof (%d bytes)",
		len(proofOps.Ops), len(proofBytes))

	return proofBytes
}

func (g *MembershipFixtureGenerator) extractCommitmentProofs(proofOps *cmtcrypto.ProofOps) []*ics23.CommitmentProof {
	commitmentProofs := make([]*ics23.CommitmentProof, len(proofOps.Ops))

	for i, op := range proofOps.Ops {
		var commitmentProof ics23.CommitmentProof
		err := proto.Unmarshal(op.Data, &commitmentProof)
		if err != nil {
			g.suite.T().Fatalf("Failed to unmarshal CommitmentProof from ProofOp[%d]: %v", i, err)
		}
		commitmentProofs[i] = &commitmentProof
	}

	return commitmentProofs
}

func (g *MembershipFixtureGenerator) saveFixture(
	ctx context.Context,
	chainA *cosmos.CosmosChain, // ChainA - where the IBC client is running
	proofCtx *ProofContext,
	proofBytes []byte,
	index int,
	clientId string,
) {
	proofType := g.proofTypeNameFor(proofCtx.ExpectMembership)

	// Build the membership message with proof data from ChainB
	membershipMsg := g.buildMembershipMessage(proofCtx, proofBytes)
	// Get the IBC client state from ChainA (tracks ChainB)
	clientState := g.buildClientState(ctx, chainA, clientId)
	// Use the consensus state from ChainB (stored in the proof context)
	consensusState := g.buildConsensusState(proofCtx)

	scenarioName := fmt.Sprintf("%s_key_%d", proofType, index)
	fixture := g.assembleFixture(
		scenarioName,
		clientState,
		consensusState,
		membershipMsg,
	)

	filename := filepath.Join(g.fixtureDir,
		fmt.Sprintf("verify_%s_key_%d.json", proofType, index))
	g.saveJsonFixture(filename, fixture)
	g.suite.T().Logf("💾 %s fixture saved: %s", proofType, filename)
}

func (g *MembershipFixtureGenerator) buildMembershipMessage(
	proofCtx *ProofContext,
	proofBytes []byte,
) map[string]interface{} {
	return map[string]interface{}{
		"height":             proofCtx.ProofHeight,
		"delay_time_period":  0,
		"delay_block_period": 0,
		"proof":              hex.EncodeToString(proofBytes),
		"path":               []string{string(ibcexported.StoreKey), proofCtx.KeyPath},
		"value":              hex.EncodeToString(proofCtx.ABCIResponse.Value),
	}
}

func (g *MembershipFixtureGenerator) buildClientState(
	ctx context.Context,
	chainA *cosmos.CosmosChain, // ChainA - where the IBC client is running
	clientId string,
) string {
	tmClientState := g.queryTendermintClientState(ctx, chainA, clientId)
	return g.convertClientStateToFixtureFormat(tmClientState)
}

func (g *MembershipFixtureGenerator) buildConsensusState(proofCtx *ProofContext) string {
	consensusStateBytes, err := proto.Marshal(proofCtx.ConsensusState)
	g.suite.Require().NoError(err)

	return hex.EncodeToString(consensusStateBytes)
}

func (g *MembershipFixtureGenerator) assembleFixture(
	scenarioName string,
	clientStateHex string,
	consensusStateHex string,
	membershipMsg map[string]interface{},
) map[string]interface{} {
	return map[string]interface{}{
		"client_state_hex":    clientStateHex,
		"consensus_state_hex": consensusStateHex,
		"membership_msg":      membershipMsg,
		"metadata":            g.createMetadata(fmt.Sprintf("Tendermint light client fixture for scenario: %s", scenarioName)),
	}
}

func (g *MembershipFixtureGenerator) saveJsonFixture(filename string, data interface{}) {
	jsonData, err := json.MarshalIndent(data, "", "  ")
	g.suite.Require().NoError(err)

	err = os.WriteFile(filename, jsonData, 0o600)
	g.suite.Require().NoError(err)
}

func (g *MembershipFixtureGenerator) queryTendermintClientState(ctx context.Context, chainA *cosmos.CosmosChain, clientId string) *ibctmtypes.ClientState {
	resp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, chainA, &clienttypes.QueryClientStateRequest{
		ClientId: clientId,
	})
	g.suite.Require().NoError(err)
	g.suite.Require().NotNil(resp.ClientState)

	var tmClientState ibctmtypes.ClientState
	err = proto.Unmarshal(resp.ClientState.Value, &tmClientState)
	g.suite.Require().NoError(err)

	return &tmClientState
}

func (g *MembershipFixtureGenerator) convertClientStateToFixtureFormat(tmClientState *ibctmtypes.ClientState) string {
	clientStateBytes, err := proto.Marshal(tmClientState)
	g.suite.Require().NoError(err)

	return hex.EncodeToString(clientStateBytes)
}

func (g *MembershipFixtureGenerator) createMetadata(description string) map[string]interface{} {
	return map[string]interface{}{
		"generated_at": time.Now().UTC().Format(time.RFC3339),
		"source":       "local_cosmos_chain",
		"description":  description,
	}
}
