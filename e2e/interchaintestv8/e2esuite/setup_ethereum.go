// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"
	"fmt"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/crypto"

	interchaintest "github.com/cosmos/interchaintest/v11"
	icfoundry "github.com/cosmos/interchaintest/v11/chain/ethereum/foundry"
	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

const (
	baseAnvilChainID   = 31337
	anvilBlockGasLimit = "50000000"
)

func (s *TestSuite) buildChainSpecs(cfg setupConfig) []*interchaintest.ChainSpec {
	if !cfg.ethereum.isAnvilBased() {
		return chainconfig.DefaultChainSpecs
	}

	// Multi-anvil: only Anvil chains, no Cosmos
	if cfg.ethereum.anvilCount > 1 {
		return s.buildAnvilSpecs(cfg.ethereum.anvilCount)
	}

	// Single anvil: Cosmos chains + one Anvil
	specs := chainconfig.DefaultChainSpecs
	chainConfig := icfoundry.DefaultEthereumAnvilChainConfig("ethereum")
	chainConfig.AdditionalStartArgs = append(chainConfig.AdditionalStartArgs, "--gas-limit", anvilBlockGasLimit)
	specs = append(specs, &interchaintest.ChainSpec{ChainConfig: chainConfig})
	return specs
}

func (s *TestSuite) buildAnvilSpecs(count int) []*interchaintest.ChainSpec {
	specs := make([]*interchaintest.ChainSpec, 0, count)
	for i := 0; i < count; i++ {
		chainID := strconv.Itoa(baseAnvilChainID + i)
		name := fmt.Sprintf("ethereum-%d", i)
		chainConfig := icfoundry.DefaultEthereumAnvilChainConfig(name)
		chainConfig.ChainID = chainID
		chainConfig.AdditionalStartArgs = append(chainConfig.AdditionalStartArgs, "--chain-id", chainID, "--gas-limit", anvilBlockGasLimit)
		specs = append(specs, &interchaintest.ChainSpec{ChainConfig: chainConfig})
	}
	return specs
}

func (s *TestSuite) setupEthereumPoSAsync(ctx context.Context, resultCh chan<- ethPosSetupResult) {
	var result ethPosSetupResult
	defer func() { resultCh <- result }()

	chain, err := chainconfig.SpinUpKurtosisPoS(ctx)
	if err != nil {
		result.err = err
		return
	}
	result.chain = &chain
}

func (s *TestSuite) processEthereumPoSResult(ctx context.Context, chain *chainconfig.EthKurtosisChain) {
	if chain == nil {
		return
	}

	s.T().Cleanup(func() {
		cleanupCtx := context.Background()
		if s.T().Failed() {
			_ = chain.DumpLogs(cleanupCtx)
		}
		chain.Destroy(cleanupCtx)
	})

	ethChain, err := ethereum.NewEthereum(ctx, chain.RPC, &chain.BeaconApiClient, chain.Faucet)
	s.Require().NoError(err, "Failed to create Ethereum client")
	s.Eth.Chains = append(s.Eth.Chains, &ethChain)
}

func (s *TestSuite) setupEthereumBesuQBFTAsync(ctx context.Context, networkID string, resultCh chan<- ethBesuQBFTSetupResult) {
	var result ethBesuQBFTSetupResult
	defer func() { resultCh <- result }()

	chain, err := chainconfig.SpinUpBesuQBFT(ctx, chainconfig.BesuQBFTParams{
		ChainID:             1337,
		Subnet:              "10.42.0.0/16",
		Gateway:             "10.42.0.1",
		ValidatorIPs:        [4]string{"10.42.0.2", "10.42.0.3", "10.42.0.4", "10.42.0.5"},
		DockerRPCAlias:      "besu-qbft-rpc",
		InterchainNetworkID: networkID,
	})
	if err != nil {
		result.err = err
		return
	}
	result.chain = &chain
}

func (s *TestSuite) processEthereumBesuQBFTResult(ctx context.Context, chain *chainconfig.BesuQBFTChain) {
	if chain == nil {
		return
	}

	s.T().Cleanup(func() {
		cleanupCtx := context.Background()
		if s.T().Failed() {
			_ = chain.DumpLogs(cleanupCtx)
		}
		chain.Destroy(cleanupCtx)
	})

	ethChain, err := ethereum.NewEthereum(ctx, chain.RPC, nil, chain.Faucet)
	s.Require().NoError(err, "Failed to create Ethereum client")
	ethChain.DockerRPC = chain.DockerRPC
	s.Eth.Chains = append(s.Eth.Chains, &ethChain)
}

// setupAnvilChains detects and sets up any Anvil chains in the slice.
func (s *TestSuite) setupAnvilChains(ctx context.Context, chains []ibc.Chain) {
	faucet, err := crypto.HexToECDSA(testvalues.E2EDeployerPrivateKeyHex)
	s.Require().NoError(err)

	for _, chain := range chains {
		if anvil, ok := chain.(*icfoundry.AnvilChain); ok {
			rpcAddr := strings.Replace(anvil.GetHostRPCAddress(), "0.0.0.0", "127.0.0.1", 1)
			ethChain, err := ethereum.NewEthereum(ctx, rpcAddr, nil, faucet)
			s.Require().NoError(err)
			ethChain.DockerRPC = anvil.GetRPCAddress()
			s.Eth.Chains = append(s.Eth.Chains, &ethChain)
		}
	}
}
