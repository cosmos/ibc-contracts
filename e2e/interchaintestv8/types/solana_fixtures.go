// SPDX-License-Identifier: Apache-2.0

package types

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec/types"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"

	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	ibctmtypes "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v11/testing"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

type SolanaFixtureGenerator struct {
	Enabled    bool
	FixtureDir string
	suite      *suite.Suite
}

func NewSolanaFixtureGenerator(s *suite.Suite) *SolanaFixtureGenerator {
	generator := &SolanaFixtureGenerator{
		Enabled: os.Getenv(testvalues.EnvKeyGenerateTendermintLightClientFixtures) == testvalues.EnvValueGenerateFixtures_True,
		suite:   s,
	}

	if generator.Enabled {
		absPath, err := filepath.Abs(filepath.Join("../..", testvalues.TendermintLightClientFixturesDir))
		if err != nil {
			s.T().Fatalf("Failed to get absolute path for fixtures: %v", err)
		}
		generator.FixtureDir = absPath

		if err := os.MkdirAll(generator.FixtureDir, 0o755); err != nil {
			s.T().Fatalf("Failed to create Solana fixture directory: %v", err)
		}
		s.T().Logf("📁 Solana fixtures will be saved to: %s", generator.FixtureDir)
	}

	return generator
}

// GenerateMultipleUpdateClientScenarios generates multiple test scenarios
func (g *SolanaFixtureGenerator) GenerateMultipleUpdateClientScenarios(ctx context.Context, chainA *cosmos.CosmosChain, updateTxBodyBz []byte) {
	if !g.Enabled {
		return
	}

	g.suite.T().Log("🔧 Generating multiple update client scenarios")

	// Extract the real update client message from the transaction
	g.suite.T().Log("🔍 Parsing update client transaction")
	msgUpdateClient := g.extractUpdateClientMessage(updateTxBodyBz)
	g.suite.T().Logf("📊 Found MsgUpdateClient for client: %s", msgUpdateClient.ClientId)

	// Generate the happy path scenario using real transaction data
	g.generateHappyPathScenario(ctx, chainA, msgUpdateClient.ClientMessage)

	// Generate malformed client message scenario based on the real data
	g.generateMalformedClientMessageScenario(ctx, chainA)

	// Generate additional edge case scenarios
	g.generateExpiredHeaderScenario(ctx, chainA)
	g.generateFutureTimestampScenario(ctx, chainA)
	g.generateWrongTrustedHeightScenario(ctx, chainA)
	g.generateInvalidProtobufScenario()
	// Note: Conflicting consensus state scenario is complex to generate with valid signatures

	g.suite.T().Log("✅ Multiple Solana scenarios generated successfully")
}

func (g *SolanaFixtureGenerator) extractUpdateClientMessage(txBodyBz []byte) *clienttypes.MsgUpdateClient {
	var txBody txtypes.TxBody
	err := proto.Unmarshal(txBodyBz, &txBody)
	g.suite.Require().NoError(err)
	g.suite.Require().Len(txBody.Messages, 1, "Expected exactly one message in update client tx")

	var msgUpdateClient clienttypes.MsgUpdateClient
	err = proto.Unmarshal(txBody.Messages[0].Value, &msgUpdateClient)
	g.suite.Require().NoError(err)
	g.suite.Require().NotNil(msgUpdateClient.ClientMessage)

	return &msgUpdateClient
}

func (g *SolanaFixtureGenerator) generateHappyPathScenario(ctx context.Context, chainA *cosmos.CosmosChain, clientMessage *types.Any) {
	g.suite.T().Log("🔧 Generating happy path scenario")

	// Get the client state
	tmClientState := g.queryTendermintClientState(ctx, chainA)
	solanaClientState := g.convertClientStateToSolanaFormat(tmClientState, chainA.Config().ChainID)

	// Get the consensus state (this would be the trusted state)
	tmConsensusState := g.queryTendermintConsensusState(ctx, chainA)
	solanaConsensusState := g.convertConsensusStateToSolanaFormat(tmConsensusState, chainA.Config().ChainID)

	// Process the real update client message from the transaction
	realUpdateMessage := g.convertUpdateClientMessageToSolanaFormat(clientMessage)

	// Create the unified fixture
	unifiedFixture := map[string]interface{}{
		"scenario":                "happy_path",
		"client_state":            solanaClientState,
		"trusted_consensus_state": solanaConsensusState,
		"update_client_message":   realUpdateMessage,
		"metadata":                g.createUnifiedMetadata("happy_path", tmClientState.ChainId),
	}

	filename := filepath.Join(g.FixtureDir, "update_client_happy_path.json")
	g.saveJsonFixture(filename, unifiedFixture)
	g.suite.T().Logf("💾 Happy path scenario fixture saved: %s", filename)
}

func (g *SolanaFixtureGenerator) generateMalformedClientMessageScenario(ctx context.Context, chainA *cosmos.CosmosChain) {
	g.suite.T().Log("🔧 Generating malformed client message scenario")

	// Get valid client state and consensus state (same as happy path)
	tmClientState := g.queryTendermintClientState(ctx, chainA)
	solanaClientState := g.convertClientStateToSolanaFormat(tmClientState, chainA.Config().ChainID)

	tmConsensusState := g.queryTendermintConsensusState(ctx, chainA)
	solanaConsensusState := g.convertConsensusStateToSolanaFormat(tmConsensusState, chainA.Config().ChainID)

	// Load the happy path fixture to base the malformed one on
	happyPathFile := filepath.Join(g.FixtureDir, "update_client_happy_path.json")
	g.suite.Require().FileExists(happyPathFile, "Happy path fixture must exist before generating malformed fixture")

	g.suite.T().Log("📖 Loading happy path fixture to create malformed version")
	validHex := g.extractHexFromHappyPathFixture(happyPathFile)

	malformedHex := g.corruptSignatureInValidHeader(validHex)

	// Create a malformed update message by corrupting signature bytes from a valid message
	malformedUpdateMessage := map[string]interface{}{
		"client_message_hex": malformedHex,
		"type_url":           "/ibc.lightclients.tendermint.v1.Header",
		"trusted_height":     tmClientState.LatestHeight.RevisionHeight,
		"new_height":         tmClientState.LatestHeight.RevisionHeight + 1,
		"metadata":           g.createMetadata("Intentionally malformed Tendermint header for unhappy path testing (signature corruption in valid protobuf structure)"),
	}

	// Create the unified fixture
	unifiedFixture := map[string]interface{}{
		"scenario":                "malformed_client_message",
		"client_state":            solanaClientState,
		"trusted_consensus_state": solanaConsensusState,
		"update_client_message":   malformedUpdateMessage,
		"metadata":                g.createUnifiedMetadata("malformed_client_message", tmClientState.ChainId),
	}

	filename := filepath.Join(g.FixtureDir, "update_client_malformed_client_message.json")
	g.saveJsonFixture(filename, unifiedFixture)
	g.suite.T().Logf("💾 Malformed client message scenario fixture saved: %s", filename)
}

func (g *SolanaFixtureGenerator) convertUpdateClientMessageToSolanaFormat(clientMessage *types.Any) map[string]interface{} {
	headerBytes := clientMessage.Value

	// Parse the header to extract the new height information
	var tmHeader ibctmtypes.Header
	err := proto.Unmarshal(headerBytes, &tmHeader)
	g.suite.Require().NoError(err, "Failed to parse header for height extraction - fixture generation cannot continue")

	// Validate that we have valid height information
	trustedHeight := tmHeader.TrustedHeight.RevisionHeight
	newHeight := tmHeader.Header.Height

	g.suite.Require().Greater(newHeight, int64(0), "New height must be greater than 0")
	g.suite.Require().Greater(trustedHeight, uint64(0), "Trusted height must be greater than 0")
	g.suite.Require().Greater(newHeight, int64(trustedHeight), "New height must be greater than trusted height")

	return map[string]interface{}{
		"client_message_hex": hex.EncodeToString(headerBytes),
		"type_url":           clientMessage.TypeUrl,
		"trusted_height":     trustedHeight,
		"new_height":         newHeight,
		"metadata":           g.createMetadata("Protobuf-encoded Tendermint header for update client"),
	}
}

// extractHexFromHappyPathFixture loads the happy path fixture and extracts the client_message_hex
func (g *SolanaFixtureGenerator) extractHexFromHappyPathFixture(filePath string) string {
	data, err := os.ReadFile(filePath)
	g.suite.Require().NoError(err, "Failed to read happy path fixture")

	var fixture map[string]interface{}
	err = json.Unmarshal(data, &fixture)
	g.suite.Require().NoError(err, "Failed to parse happy path fixture JSON")

	updateMessage, ok := fixture["update_client_message"].(map[string]interface{})
	g.suite.Require().True(ok, "update_client_message not found in happy path fixture")

	hex, ok := updateMessage["client_message_hex"].(string)
	g.suite.Require().True(ok, "client_message_hex not found in happy path fixture")

	return hex
}

// corruptSignatureInValidHeader takes a valid header hex and corrupts signature bytes
// This creates a valid protobuf structure that will deserialize correctly but fail cryptographic verification
func (g *SolanaFixtureGenerator) corruptSignatureInValidHeader(validHex string) string {
	// Decode the hex string to bytes
	headerBytes, err := hex.DecodeString(validHex)
	if err != nil {
		g.suite.T().Fatalf("Failed to decode valid header hex: %v", err)
	}

	// Parse the header first to understand its structure
	var tmHeader ibctmtypes.Header
	err = proto.Unmarshal(headerBytes, &tmHeader)
	if err != nil {
		g.suite.T().Fatalf("Failed to parse header for corruption: %v", err)
	}

	// Make a copy to avoid modifying the original
	corruptedHeader := tmHeader

	// Corrupt signature data in the commit while preserving the protobuf structure
	if corruptedHeader.SignedHeader != nil && corruptedHeader.Commit != nil {
		commit := corruptedHeader.Commit

		// Corrupt block signature if it exists
		if len(commit.Signatures) > 0 {
			// Corrupt the first signature by flipping one byte
			if len(commit.Signatures[0].Signature) > 10 {
				// Flip a byte in the middle of the signature
				sigPos := len(commit.Signatures[0].Signature) / 2
				commit.Signatures[0].Signature[sigPos] ^= 0xFF
				g.suite.T().Logf("🔧 Corrupted signature byte at position %d in first commit signature", sigPos)
			}
		}

		// Also corrupt the block ID hash if present
		if len(commit.BlockID.Hash) > 0 {
			// Flip one byte in the block hash
			hashPos := len(commit.BlockID.Hash) / 2
			commit.BlockID.Hash[hashPos] ^= 0xFF
			g.suite.T().Logf("🔧 Corrupted block hash byte at position %d", hashPos)
		}
	}

	// Re-marshal the corrupted header
	corruptedBytes, err := proto.Marshal(&corruptedHeader)
	if err != nil {
		g.suite.T().Fatalf("Failed to marshal corrupted header: %v", err)
	}

	// Verify it can still be parsed (should succeed)
	var testHeader ibctmtypes.Header
	err = proto.Unmarshal(corruptedBytes, &testHeader)
	if err != nil {
		g.suite.T().Fatalf("Corrupted header failed to parse - corruption was too aggressive: %v", err)
	}

	g.suite.T().Log("🔧 Header corrupted successfully - still deserializable but signatures are invalid")
	return hex.EncodeToString(corruptedBytes)
}

func (g *SolanaFixtureGenerator) queryTendermintClientState(ctx context.Context, chainA *cosmos.CosmosChain) *ibctmtypes.ClientState {
	resp, err := e2esuite.GRPCQuery[clienttypes.QueryClientStateResponse](ctx, chainA, &clienttypes.QueryClientStateRequest{
		ClientId: ibctesting.FirstClientID,
	})
	g.suite.Require().NoError(err)
	g.suite.Require().NotNil(resp.ClientState)

	var tmClientState ibctmtypes.ClientState
	err = proto.Unmarshal(resp.ClientState.Value, &tmClientState)
	g.suite.Require().NoError(err)

	return &tmClientState
}

func (g *SolanaFixtureGenerator) convertClientStateToSolanaFormat(tmClientState *ibctmtypes.ClientState, chainID string) map[string]interface{} {
	return map[string]interface{}{
		"chain_id":                tmClientState.ChainId,
		"trust_level_numerator":   tmClientState.TrustLevel.Numerator,
		"trust_level_denominator": tmClientState.TrustLevel.Denominator,
		"trusting_period":         tmClientState.TrustingPeriod.Seconds(),
		"unbonding_period":        tmClientState.UnbondingPeriod.Seconds(),
		"max_clock_drift":         tmClientState.MaxClockDrift.Seconds(),
		"frozen_height":           tmClientState.FrozenHeight.RevisionHeight,
		"latest_height":           tmClientState.LatestHeight.RevisionHeight,
		"metadata":                g.createMetadata(fmt.Sprintf("Client state for %s captured from %s", tmClientState.ChainId, chainID)),
	}
}

func (g *SolanaFixtureGenerator) queryTendermintConsensusState(ctx context.Context, chainA *cosmos.CosmosChain) *ibctmtypes.ConsensusState {
	resp, err := e2esuite.GRPCQuery[clienttypes.QueryConsensusStateResponse](ctx, chainA, &clienttypes.QueryConsensusStateRequest{
		ClientId:       ibctesting.FirstClientID,
		RevisionNumber: 1,
		RevisionHeight: 1,
		LatestHeight:   true,
	})
	g.suite.Require().NoError(err)
	g.suite.Require().NotNil(resp.ConsensusState)

	var tmConsensusState ibctmtypes.ConsensusState
	err = proto.Unmarshal(resp.ConsensusState.Value, &tmConsensusState)
	g.suite.Require().NoError(err)

	return &tmConsensusState
}

func (g *SolanaFixtureGenerator) convertConsensusStateToSolanaFormat(tmConsensusState *ibctmtypes.ConsensusState, chainID string) map[string]interface{} {
	return map[string]interface{}{
		"timestamp":            tmConsensusState.Timestamp.UnixNano(),
		"root":                 hex.EncodeToString(tmConsensusState.Root.GetHash()),
		"next_validators_hash": hex.EncodeToString(tmConsensusState.NextValidatorsHash),
		"metadata":             g.createMetadata(fmt.Sprintf("Consensus state captured from %s", chainID)),
	}
}

func (g *SolanaFixtureGenerator) saveJsonFixture(filename string, data interface{}) {
	jsonData, err := json.MarshalIndent(data, "", "  ")
	g.suite.Require().NoError(err)

	err = os.WriteFile(filename, jsonData, 0o600)
	g.suite.Require().NoError(err)
}

func (g *SolanaFixtureGenerator) createMetadata(description string) map[string]interface{} {
	return map[string]interface{}{
		"generated_at": time.Now().UTC().Format(time.RFC3339),
		"source":       "real_cosmos_chain",
		"description":  description,
	}
}

func (g *SolanaFixtureGenerator) createUnifiedMetadata(scenarioName, chainID string) map[string]interface{} {
	return map[string]interface{}{
		"generated_at": time.Now().UTC().Format(time.RFC3339),
		"source":       "real_cosmos_chain",
		"description":  fmt.Sprintf("Unified update client fixture for scenario: %s", scenarioName),
		"scenario":     scenarioName,
		"chain_id":     chainID,
	}
}

// generateExpiredHeaderScenario creates a fixture with an expired header (beyond trusting period)
func (g *SolanaFixtureGenerator) generateExpiredHeaderScenario(ctx context.Context, chainA *cosmos.CosmosChain) {
	g.suite.T().Log("🔧 Generating expired header scenario")

	// Get valid client state and consensus state
	tmClientState := g.queryTendermintClientState(ctx, chainA)
	solanaClientState := g.convertClientStateToSolanaFormat(tmClientState, chainA.Config().ChainID)

	tmConsensusState := g.queryTendermintConsensusState(ctx, chainA)
	solanaConsensusState := g.convertConsensusStateToSolanaFormat(tmConsensusState, chainA.Config().ChainID)

	// Load the happy path fixture to base the expired one on
	happyPathFile := filepath.Join(g.FixtureDir, "update_client_happy_path.json")
	g.suite.Require().FileExists(happyPathFile)

	validHex := g.extractHexFromHappyPathFixture(happyPathFile)

	// Create an expired header by modifying the timestamp
	expiredHex := g.createExpiredHeader(validHex, int64(tmClientState.TrustingPeriod.Seconds()))

	expiredUpdateMessage := map[string]interface{}{
		"client_message_hex": expiredHex,
		"type_url":           "/ibc.lightclients.tendermint.v1.Header",
		"trusted_height":     tmClientState.LatestHeight.RevisionHeight,
		"new_height":         tmClientState.LatestHeight.RevisionHeight + 1,
		"metadata":           g.createMetadata("Expired header - timestamp beyond trusting period"),
	}

	unifiedFixture := map[string]interface{}{
		"scenario":                "expired_header",
		"client_state":            solanaClientState,
		"trusted_consensus_state": solanaConsensusState,
		"update_client_message":   expiredUpdateMessage,
		"metadata":                g.createUnifiedMetadata("expired_header", tmClientState.ChainId),
	}

	filename := filepath.Join(g.FixtureDir, "update_client_expired_header.json")
	g.saveJsonFixture(filename, unifiedFixture)
	g.suite.T().Logf("💾 Expired header scenario fixture saved: %s", filename)
}

// generateFutureTimestampScenario creates a fixture with a future timestamp
func (g *SolanaFixtureGenerator) generateFutureTimestampScenario(ctx context.Context, chainA *cosmos.CosmosChain) {
	g.suite.T().Log("🔧 Generating future timestamp scenario")

	tmClientState := g.queryTendermintClientState(ctx, chainA)
	solanaClientState := g.convertClientStateToSolanaFormat(tmClientState, chainA.Config().ChainID)

	tmConsensusState := g.queryTendermintConsensusState(ctx, chainA)
	solanaConsensusState := g.convertConsensusStateToSolanaFormat(tmConsensusState, chainA.Config().ChainID)

	happyPathFile := filepath.Join(g.FixtureDir, "update_client_happy_path.json")
	g.suite.Require().FileExists(happyPathFile)

	validHex := g.extractHexFromHappyPathFixture(happyPathFile)

	// Create a header with future timestamp (beyond max clock drift)
	futureHex := g.createFutureTimestampHeader(validHex, int64(tmClientState.MaxClockDrift.Seconds()))

	futureUpdateMessage := map[string]interface{}{
		"client_message_hex": futureHex,
		"type_url":           "/ibc.lightclients.tendermint.v1.Header",
		"trusted_height":     tmClientState.LatestHeight.RevisionHeight,
		"new_height":         tmClientState.LatestHeight.RevisionHeight + 1,
		"metadata":           g.createMetadata("Future timestamp - beyond max clock drift"),
	}

	unifiedFixture := map[string]interface{}{
		"scenario":                "future_timestamp",
		"client_state":            solanaClientState,
		"trusted_consensus_state": solanaConsensusState,
		"update_client_message":   futureUpdateMessage,
		"metadata":                g.createUnifiedMetadata("future_timestamp", tmClientState.ChainId),
	}

	filename := filepath.Join(g.FixtureDir, "update_client_future_timestamp.json")
	g.saveJsonFixture(filename, unifiedFixture)
	g.suite.T().Logf("💾 Future timestamp scenario fixture saved: %s", filename)
}

// generateWrongTrustedHeightScenario creates a fixture referencing wrong trusted height
func (g *SolanaFixtureGenerator) generateWrongTrustedHeightScenario(ctx context.Context, chainA *cosmos.CosmosChain) {
	g.suite.T().Log("🔧 Generating wrong trusted height scenario")

	tmClientState := g.queryTendermintClientState(ctx, chainA)
	solanaClientState := g.convertClientStateToSolanaFormat(tmClientState, chainA.Config().ChainID)

	tmConsensusState := g.queryTendermintConsensusState(ctx, chainA)
	solanaConsensusState := g.convertConsensusStateToSolanaFormat(tmConsensusState, chainA.Config().ChainID)

	happyPathFile := filepath.Join(g.FixtureDir, "update_client_happy_path.json")
	g.suite.Require().FileExists(happyPathFile)

	validHex := g.extractHexFromHappyPathFixture(happyPathFile)

	// Use the valid header but with wrong trusted height in metadata
	wrongHeightUpdateMessage := map[string]interface{}{
		"client_message_hex": validHex,
		"type_url":           "/ibc.lightclients.tendermint.v1.Header",
		"trusted_height":     tmClientState.LatestHeight.RevisionHeight + 100, // Wrong height
		"new_height":         tmClientState.LatestHeight.RevisionHeight + 1,
		"metadata":           g.createMetadata("Wrong trusted height - references non-existent consensus state"),
	}

	unifiedFixture := map[string]interface{}{
		"scenario":                "wrong_trusted_height",
		"client_state":            solanaClientState,
		"trusted_consensus_state": solanaConsensusState,
		"update_client_message":   wrongHeightUpdateMessage,
		"metadata":                g.createUnifiedMetadata("wrong_trusted_height", tmClientState.ChainId),
	}

	filename := filepath.Join(g.FixtureDir, "update_client_wrong_trusted_height.json")
	g.saveJsonFixture(filename, unifiedFixture)
	g.suite.T().Logf("💾 Wrong trusted height scenario fixture saved: %s", filename)
}

// generateInvalidProtobufScenario creates a fixture with invalid protobuf bytes
func (g *SolanaFixtureGenerator) generateInvalidProtobufScenario() {
	g.suite.T().Log("🔧 Generating invalid protobuf scenario")

	// Create completely invalid protobuf bytes
	invalidProtobuf := "FFFFFFFF" // Invalid protobuf that can't be decoded

	invalidUpdateMessage := map[string]interface{}{
		"client_message_hex": invalidProtobuf,
		"type_url":           "/ibc.lightclients.tendermint.v1.Header",
		"trusted_height":     19,
		"new_height":         20,
		"metadata":           g.createMetadata("Invalid protobuf bytes - cannot be deserialized"),
	}

	// Use dummy client and consensus states
	dummyClientState := map[string]interface{}{
		"chain_id":                "test-chain",
		"trust_level_numerator":   1,
		"trust_level_denominator": 3,
		"trusting_period":         1209600,
		"unbonding_period":        1814400,
		"max_clock_drift":         10,
		"frozen_height":           0,
		"latest_height":           19,
		"metadata":                g.createMetadata("Dummy client state for invalid protobuf test"),
	}

	dummyConsensusState := map[string]interface{}{
		"timestamp":            uint64(time.Now().Unix()),
		"root":                 hex.EncodeToString(make([]byte, 32)),
		"next_validators_hash": hex.EncodeToString(make([]byte, 32)),
		"metadata":             g.createMetadata("Dummy consensus state for invalid protobuf test"),
	}

	unifiedFixture := map[string]interface{}{
		"scenario":                "invalid_protobuf",
		"client_state":            dummyClientState,
		"trusted_consensus_state": dummyConsensusState,
		"update_client_message":   invalidUpdateMessage,
		"metadata":                g.createUnifiedMetadata("invalid_protobuf", "test-chain"),
	}

	filename := filepath.Join(g.FixtureDir, "update_client_invalid_protobuf.json")
	g.saveJsonFixture(filename, unifiedFixture)
	g.suite.T().Logf("💾 Invalid protobuf scenario fixture saved: %s", filename)
}

// Helper functions for modifying headers

func (g *SolanaFixtureGenerator) createExpiredHeader(validHex string, trustingPeriodSeconds int64) string {
	headerBytes, _ := hex.DecodeString(validHex)
	var header ibctmtypes.Header
	if err := proto.Unmarshal(headerBytes, &header); err != nil {
		g.suite.T().Fatalf("Failed to unmarshal header: %v", err)
	}

	// Set timestamp to be older than trusting period
	expiredTime := time.Now().Add(-time.Duration(trustingPeriodSeconds+3600) * time.Second) // Add 1 hour buffer
	header.Header.Time = expiredTime

	modifiedBytes, _ := proto.Marshal(&header)
	return hex.EncodeToString(modifiedBytes)
}

func (g *SolanaFixtureGenerator) createFutureTimestampHeader(validHex string, maxClockDriftSeconds int64) string {
	headerBytes, _ := hex.DecodeString(validHex)
	var header ibctmtypes.Header
	if err := proto.Unmarshal(headerBytes, &header); err != nil {
		g.suite.T().Fatalf("Failed to unmarshal header: %v", err)
	}

	// Set timestamp to be in the future beyond max clock drift
	futureTime := time.Now().Add(time.Duration(maxClockDriftSeconds+3600) * time.Second) // Add 1 hour buffer
	header.Header.Time = futureTime

	modifiedBytes, _ := proto.Marshal(&header)
	return hex.EncodeToString(modifiedBytes)
}
