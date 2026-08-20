// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"
	"os"

	dockerclient "github.com/moby/moby/client"
	"github.com/stretchr/testify/suite"
	"go.uber.org/zap"
	"go.uber.org/zap/zaptest"

	interchaintest "github.com/cosmos/interchaintest/v11"
	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"
	"github.com/cosmos/interchaintest/v11/testreporter"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

// TestSuite is a suite of tests that require two chains and a relayer
type TestSuite struct {
	suite.Suite

	// Chain-specific state
	Eth struct {
		Chains []*ethereum.Ethereum
	}
	Cosmos struct {
		Chains      []*cosmos.CosmosChain
		Users       []ibc.Wallet
		proposalIDs map[string]uint64
	}
	Solana struct {
		Chain solana.Solana
	}

	// Parsed configuration
	config setupConfig

	// Infrastructure
	dockerClient *dockerclient.Client
	network      string
	logger       *zap.Logger
}

// SetupSuite sets up the chains, relayer, user accounts, clients, and connections
func (s *TestSuite) SetupSuite(ctx context.Context) {
	s.config = s.parseConfig()
	if err := s.config.validate(); err != nil {
		s.T().Fatalf("Configuration error: %v", err)
	}

	if s.config.cosmos.lightClientType != "" {
		s.T().Logf("wasm type %s", s.config.cosmos.lightClientType)
	}

	chainSpecs := s.buildChainSpecs(s.config)
	chains, dockerClient, network, logger := s.setupChainsInParallel(ctx, s.config, chainSpecs)

	s.logger = logger
	s.dockerClient = dockerClient
	s.network = network

	s.processChains(ctx, s.config, chains)
}

// GetEthLightClientType returns the Ethereum light client type on Cosmos (dummy, full, attestor-native).
func (s *TestSuite) GetEthLightClientType() string {
	return s.config.cosmos.lightClientType
}

// GetDockerClient returns the Docker client used for container management.
// This is primarily used for setting up Docker-based attestors.
func (s *TestSuite) GetDockerClient() *dockerclient.Client {
	return s.dockerClient
}

// GetNetworkID returns the Docker network ID used for container networking.
// This is primarily used for setting up Docker-based attestors.
func (s *TestSuite) GetNetworkID() string {
	return s.network
}

// SetupDocker initializes just the Docker client and network without starting any chains.
// This is useful for lightweight tests that only need Docker resources.
func (s *TestSuite) SetupDocker() {
	if s.dockerClient != nil {
		return // Already initialized
	}
	s.dockerClient, s.network = interchaintest.DockerSetup(s.T())
}

func (s *TestSuite) parseConfig() setupConfig {
	return setupConfig{
		ethereum: ethereumConfig{
			testnetType: os.Getenv(testvalues.EnvKeyEthTestnetType),
			anvilCount:  envInt(testvalues.EnvKeyEthAnvilCount, 1),
		},
		solana: solanaConfig{
			testnetType: os.Getenv(testvalues.EnvKeySolanaTestnetType),
		},
		cosmos: cosmosConfig{
			lightClientType:    os.Getenv(testvalues.EnvKeyEthLcOnCosmos),
			wasmLightClientTag: os.Getenv(testvalues.EnvKeyE2EWasmLightClientTag),
		},
	}
}

func (s *TestSuite) setupChainsInParallel(
	ctx context.Context,
	cfg setupConfig,
	chainSpecs []*interchaintest.ChainSpec,
) ([]ibc.Chain, *dockerclient.Client, string, *zap.Logger) {
	solanaCh := make(chan solanaSetupResult, 1)
	interchainCh := make(chan interchainSetupResult, 1)
	ethPosCh := make(chan ethPosSetupResult, 1)
	ethBesuQBFTCh := make(chan ethBesuQBFTSetupResult, 1)

	go s.setupSolanaAsync(ctx, cfg, solanaCh)
	go s.setupInterchainAsync(ctx, chainSpecs, interchainCh)

	if cfg.ethereum.needsPoS() {
		go s.setupEthereumPoSAsync(ctx, ethPosCh)
	} else {
		ethPosCh <- ethPosSetupResult{}
	}

	interchainRes := <-interchainCh
	s.Require().NoError(interchainRes.err, "Interchain setup failed")

	if cfg.ethereum.isBesuQBFT() {
		go s.setupEthereumBesuQBFTAsync(ctx, interchainRes.network, ethBesuQBFTCh)
	} else {
		ethBesuQBFTCh <- ethBesuQBFTSetupResult{}
	}

	solanaRes := <-solanaCh
	s.Require().NoError(solanaRes.err, "Solana chain setup failed")
	s.processSolanaResult(solanaRes.chain)

	ethPosRes := <-ethPosCh
	s.Require().NoError(ethPosRes.err, "Ethereum PoS chain setup failed")
	s.processEthereumPoSResult(ctx, ethPosRes.chain)

	ethBesuQBFTRes := <-ethBesuQBFTCh
	s.Require().NoError(ethBesuQBFTRes.err, "Ethereum Besu QBFT chain setup failed")
	s.processEthereumBesuQBFTResult(ctx, ethBesuQBFTRes.chain)

	return interchainRes.chains, interchainRes.dockerClient, interchainRes.network, interchainRes.logger
}

func (s *TestSuite) setupInterchainAsync(
	ctx context.Context,
	chainSpecs []*interchaintest.ChainSpec,
	resultCh chan<- interchainSetupResult,
) {
	var result interchainSetupResult
	defer func() { resultCh <- result }()

	testName := s.T().Name()
	logger := zaptest.NewLogger(s.T())
	dockerClient, network := interchaintest.DockerSetup(s.T())

	result.logger = logger
	result.dockerClient = dockerClient
	result.network = network

	cf := interchaintest.NewBuiltinChainFactory(logger, chainSpecs)
	chains, err := cf.Chains(testName)
	if err != nil {
		result.err = err
		return
	}

	ic := interchaintest.NewInterchain()
	for _, chain := range chains {
		ic = ic.AddChain(chain)
	}

	execRep := testreporter.NewNopReporter().RelayerExecReporter(s.T())
	err = ic.Build(ctx, execRep, interchaintest.InterchainBuildOptions{
		TestName:         testName,
		Client:           dockerClient,
		NetworkID:        network,
		SkipPathCreation: true,
	})
	if err != nil {
		result.err = err
		return
	}

	result.chains = chains
}

func (s *TestSuite) processChains(ctx context.Context, cfg setupConfig, chains []ibc.Chain) {
	s.setupAnvilChains(ctx, chains)
	s.setupCosmosChains(ctx, chains)
}
