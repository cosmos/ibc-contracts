// SPDX-License-Identifier: Apache-2.0

package types

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/suite"

	txtypes "github.com/cosmos/cosmos-sdk/types/tx"

	ibcwasmtypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/v11/types"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	ethereumtypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/ethereum"
)

type WasmFixtureGenerator struct {
	shouldGenerateFixture bool
	fixture               ethereumtypes.StepsFixture
}

// NewWasmFixtureGenerator creates a new WasmFixtureGenerator
func NewWasmFixtureGenerator(s *suite.Suite) *WasmFixtureGenerator {
	wasmFixtureGenerator := &WasmFixtureGenerator{
		shouldGenerateFixture: os.Getenv(testvalues.EnvKeyGenerateWasmFixtures) == testvalues.EnvValueGenerateFixtures_True,
	}

	fixtureName := getTopLevelTestName(s)
	if wasmFixtureGenerator.shouldGenerateFixture {
		s.T().Cleanup(func() {
			s.T().Logf("Writing fixtures for %s", fixtureName)
			if err := wasmFixtureGenerator.writeFixtures(fixtureName); err != nil {
				s.T().Logf("Error writing fixtures: %v", err)
			}
		})
	}

	return wasmFixtureGenerator
}

// AddFixtureStep adds a new fixture step that will be written to the fixture file at the end of the test
func (g *WasmFixtureGenerator) AddFixtureStep(stepName string, jsonMarshalble interface{}) {
	if !g.shouldGenerateFixture {
		return
	}

	g.fixture.Steps = append(g.fixture.Steps, ethereumtypes.Step{
		Name: stepName,
		Data: jsonMarshalble,
	})
}

// AddInitialStateStep creates the initial step from the create client tx body (proof API response)
func (g *WasmFixtureGenerator) AddInitialStateStep(createClientTxBodyBz []byte) error {
	if !g.shouldGenerateFixture {
		return nil
	}

	var (
		txBody             txtypes.TxBody
		msgCreateClient    clienttypes.MsgCreateClient
		wasmClientState    ibcwasmtypes.ClientState
		wasmConsensusState ibcwasmtypes.ConsensusState
		clientState        ethereumtypes.ClientState
		consensusState     ethereumtypes.ConsensusState
	)

	if err := proto.Unmarshal(createClientTxBodyBz, &txBody); err != nil {
		return err
	}
	if len(txBody.GetMessages()) != 1 {
		return fmt.Errorf("expected 1 `create_client` message, got %d", len(txBody.GetMessages()))
	}

	if err := proto.Unmarshal(txBody.GetMessages()[0].GetValue(), &msgCreateClient); err != nil {
		return err
	}

	if err := proto.Unmarshal(msgCreateClient.ClientState.GetValue(), &wasmClientState); err != nil {
		return err
	}
	if err := proto.Unmarshal(msgCreateClient.ConsensusState.GetValue(), &wasmConsensusState); err != nil {
		return err
	}

	if err := json.Unmarshal(wasmClientState.Data, &clientState); err != nil {
		return err
	}
	if err := json.Unmarshal(wasmConsensusState.Data, &consensusState); err != nil {
		return err
	}

	g.AddFixtureStep("initial_state", ethereumtypes.InitialState{
		ClientState:    clientState,
		ConsensusState: consensusState,
	})

	return nil
}

func (g *WasmFixtureGenerator) ShouldGenerateFixture() bool {
	return g.shouldGenerateFixture
}

func (g *WasmFixtureGenerator) writeFixtures(fixtureName string) error {
	if !g.shouldGenerateFixture {
		return nil
	}

	filePath := fmt.Sprintf("%s/%s.json", testvalues.WasmFixturesDir, fixtureName)

	fmt.Printf("Writing %d fixtures to %s\n", len(g.fixture.Steps), filePath)
	fixturesBz, err := json.MarshalIndent(g.fixture, "", " ")
	if err != nil {
		return err
	}

	// nolint:gosec
	return os.WriteFile(filePath, fixturesBz, 0o644)
}

func getTopLevelTestName(s *suite.Suite) string {
	parts := strings.Split(s.T().Name(), "/")

	if len(parts) >= 2 {
		return parts[1]
	}

	return s.T().Name()
}
