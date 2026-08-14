// SPDX-License-Identifier: Apache-2.0

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
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec/types"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"

	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	ibctmtypes "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
)

type UpdateClientFixtureGenerator struct {
	enabled    bool
	fixtureDir string
	suite      *suite.Suite
}

func NewUpdateClientFixtureGenerator(enabled bool, fixtureDir string, s *suite.Suite) *UpdateClientFixtureGenerator {
	return &UpdateClientFixtureGenerator{
		enabled:    enabled,
		fixtureDir: fixtureDir,
		suite:      s,
	}
}

func (g *UpdateClientFixtureGenerator) GenerateUpdateClientHappyPath(
	ctx context.Context,
	chain *cosmos.CosmosChain,
	updateTxBodyBz []byte,
) {
	if !g.enabled {
		return
	}

	g.suite.T().Log("🔧 Generating update client happy path fixture")

	msgUpdateClient := g.extractSingleUpdateClientMessageFromTransaction(updateTxBodyBz)
	clientId := msgUpdateClient.ClientId
	g.suite.T().Logf("📊 Found MsgUpdateClient for client: %s", clientId)

	g.generateHappyPathScenarioFromRealTransaction(ctx, chain, msgUpdateClient.ClientMessage, clientId)

	g.suite.T().Log("✅ Update client happy path fixture generated successfully")
}

func (g *UpdateClientFixtureGenerator) extractSingleUpdateClientMessageFromTransaction(txBodyBz []byte) *clienttypes.MsgUpdateClient {
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

func (g *UpdateClientFixtureGenerator) generateHappyPathScenarioFromRealTransaction(
	ctx context.Context,
	chain *cosmos.CosmosChain,
	clientMessage *types.Any,
	clientId string,
) {
	g.suite.T().Log("🔧 Generating happy path scenario")

	clientState := g.fetchAndFormatClientState(ctx, chain, clientId)
	consensusState := g.fetchAndFormatConsensusState(ctx, chain, clientId)
	updateMessage := g.formatClientMessageForFixture(clientMessage)

	fixture := g.createUpdateClientFixture(
		"happy_path",
		clientState,
		consensusState,
		updateMessage,
	)

	g.saveFixtureToFile(fixture, "update_client_happy_path.json")
}

func (g *UpdateClientFixtureGenerator) fetchAndFormatClientState(
	ctx context.Context,
	chain *cosmos.CosmosChain,
	clientId string,
) string {
	tmClientState := g.queryTendermintClientState(ctx, chain, clientId)
	return g.convertClientStateToFixtureFormat(tmClientState)
}

func (g *UpdateClientFixtureGenerator) fetchAndFormatConsensusState(
	ctx context.Context,
	chain *cosmos.CosmosChain,
	clientId string,
) string {
	tmConsensusState := g.queryTendermintLatestConsensusState(ctx, chain, clientId)
	return g.convertConsensusStateToFixtureFormat(tmConsensusState)
}

func (g *UpdateClientFixtureGenerator) formatClientMessageForFixture(clientMessage *types.Any) map[string]interface{} {
	headerBytes := clientMessage.Value
	tmHeader := g.parseAndValidateTendermintHeader(headerBytes)

	return map[string]interface{}{
		"client_message_hex": hex.EncodeToString(headerBytes),
		"type_url":           clientMessage.TypeUrl,
		"trusted_height":     tmHeader.TrustedHeight.RevisionHeight,
		"new_height":         tmHeader.Header.Height,
		"metadata":           g.createMetadata("Protobuf-encoded Tendermint header for update client"),
	}
}

func (g *UpdateClientFixtureGenerator) parseAndValidateTendermintHeader(headerBytes []byte) *ibctmtypes.Header {
	var tmHeader ibctmtypes.Header
	err := proto.Unmarshal(headerBytes, &tmHeader)
	g.suite.Require().NoError(err, "Failed to parse header for height extraction - fixture generation cannot continue")

	trustedHeight := tmHeader.TrustedHeight.RevisionHeight
	newHeight := tmHeader.Header.Height

	g.suite.Require().Greater(newHeight, int64(0), "New height must be greater than 0")
	g.suite.Require().Greater(trustedHeight, uint64(0), "Trusted height must be greater than 0")
	g.suite.Require().Greater(newHeight, int64(trustedHeight), "New height must be greater than trusted height")

	return &tmHeader
}

func (g *UpdateClientFixtureGenerator) createUpdateClientFixture(
	scenario string,
	clientStateHex string,
	consensusStateHex string,
	updateMessage map[string]interface{},
) map[string]interface{} {
	return map[string]interface{}{
		"client_state_hex":      clientStateHex,
		"consensus_state_hex":   consensusStateHex,
		"update_client_message": updateMessage,
		"metadata":              g.createMetadata(fmt.Sprintf("Tendermint light client fixture for scenario: %s", scenario)),
	}
}

func (g *UpdateClientFixtureGenerator) saveFixtureToFile(fixture map[string]interface{}, filename string) {
	fullPath := filepath.Join(g.fixtureDir, filename)
	g.saveJsonFixture(fullPath, fixture)
	g.suite.T().Logf("💾 Fixture saved: %s", fullPath)
}

func (g *UpdateClientFixtureGenerator) saveJsonFixture(filename string, data interface{}) {
	jsonData, err := json.MarshalIndent(data, "", "  ")
	g.suite.Require().NoError(err)

	err = os.WriteFile(filename, jsonData, 0o600)
	g.suite.Require().NoError(err)
}

func (g *UpdateClientFixtureGenerator) queryTendermintClientState(ctx context.Context, chainA *cosmos.CosmosChain, clientId string) *ibctmtypes.ClientState {
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

func (g *UpdateClientFixtureGenerator) queryTendermintLatestConsensusState(ctx context.Context, chainA *cosmos.CosmosChain, clientId string) *ibctmtypes.ConsensusState {
	resp, err := e2esuite.GRPCQuery[clienttypes.QueryConsensusStateResponse](ctx, chainA, &clienttypes.QueryConsensusStateRequest{
		ClientId:     clientId,
		LatestHeight: true,
	})
	g.suite.Require().NoError(err)
	g.suite.Require().NotNil(resp.ConsensusState)

	var tmConsensusState ibctmtypes.ConsensusState
	err = proto.Unmarshal(resp.ConsensusState.Value, &tmConsensusState)
	g.suite.Require().NoError(err)

	return &tmConsensusState
}

func (g *UpdateClientFixtureGenerator) convertClientStateToFixtureFormat(tmClientState *ibctmtypes.ClientState) string {
	clientStateBytes, err := proto.Marshal(tmClientState)
	g.suite.Require().NoError(err)

	return hex.EncodeToString(clientStateBytes)
}

func (g *UpdateClientFixtureGenerator) convertConsensusStateToFixtureFormat(tmConsensusState *ibctmtypes.ConsensusState) string {
	consensusStateBytes, err := proto.Marshal(tmConsensusState)
	g.suite.Require().NoError(err)

	return hex.EncodeToString(consensusStateBytes)
}

func (g *UpdateClientFixtureGenerator) createMetadata(description string) map[string]interface{} {
	return map[string]interface{}{
		"generated_at": time.Now().UTC().Format(time.RFC3339),
		"source":       "local_cosmos_chain",
		"description":  description,
	}
}
