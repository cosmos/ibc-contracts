// SPDX-License-Identifier: Apache-2.0

package types

import (
	"context"
	"os"
	"path/filepath"

	"github.com/stretchr/testify/suite"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/tendermint_light_client_fixtures"
)

type KeyPath = tendermint_light_client_fixtures.KeyPath

type TendermintLightClientFixtureGenerator struct {
	Enabled    bool
	FixtureDir string
	suite      *suite.Suite

	updateClientGenerator *tendermint_light_client_fixtures.UpdateClientFixtureGenerator
	membershipGenerator   *tendermint_light_client_fixtures.MembershipFixtureGenerator
}

func NewTendermintLightClientFixtureGenerator(s *suite.Suite) *TendermintLightClientFixtureGenerator {
	generator := &TendermintLightClientFixtureGenerator{
		Enabled: isFixtureGenerationEnabled(),
		suite:   s,
	}

	if generator.Enabled {
		generator.initializeFixtureDirectory(s)
	}

	generator.initializeSubmodules(s)

	return generator
}

func isFixtureGenerationEnabled() bool {
	return os.Getenv(testvalues.EnvKeyGenerateTendermintLightClientFixtures) == testvalues.EnvValueGenerateFixtures_True
}

func (g *TendermintLightClientFixtureGenerator) initializeFixtureDirectory(s *suite.Suite) {
	absPath, err := filepath.Abs(filepath.Join("../..", testvalues.TendermintLightClientFixturesDir))
	s.Require().NoErrorf(err, "failed to get absolute path for fixtures")

	g.FixtureDir = absPath

	// Does nothing if already exists
	err = os.MkdirAll(g.FixtureDir, 0o755) // does nothing if already exists
	s.Require().NoErrorf(err, "failed to create Tendermint light client fixture directory")

	s.T().Logf("📁 Tendermint light client fixtures will be saved to: %s", g.FixtureDir)
}

func (g *TendermintLightClientFixtureGenerator) initializeSubmodules(s *suite.Suite) {
	g.updateClientGenerator = tendermint_light_client_fixtures.NewUpdateClientFixtureGenerator(g.Enabled, g.FixtureDir, s)
	g.membershipGenerator = tendermint_light_client_fixtures.NewMembershipFixtureGenerator(g.Enabled, g.FixtureDir, s)
}

// Public API - Main Fixture Generation Methods

func (g *TendermintLightClientFixtureGenerator) GenerateUpdateClientHappyPath(
	ctx context.Context,
	chainA *cosmos.CosmosChain,
	updateTxBodyBz []byte,
) {
	g.updateClientGenerator.GenerateUpdateClientHappyPath(ctx, chainA, updateTxBodyBz)
}

func (g *TendermintLightClientFixtureGenerator) GenerateMembershipVerificationScenarios(
	ctx context.Context,
	chainA *cosmos.CosmosChain,
	chainB *cosmos.CosmosChain,
	keyPaths []KeyPath,
	clientId string,
) {
	g.membershipGenerator.GenerateMembershipVerificationScenarios(ctx, chainA, chainB, keyPaths, clientId)
}
