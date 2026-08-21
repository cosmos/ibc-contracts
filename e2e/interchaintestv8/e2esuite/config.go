// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"fmt"
	"os"
	"strconv"

	dockerclient "github.com/moby/moby/client"
	"go.uber.org/zap"

	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

func envInt(key string, defaultVal int) int {
	if v, err := strconv.Atoi(os.Getenv(key)); err == nil && v > 0 {
		return v
	}
	return defaultVal
}

// ethereumConfig holds configuration for Ethereum chain setup.
type ethereumConfig struct {
	testnetType string
	anvilCount  int
}

// isAnvilBased returns true for testnets using local Anvil chain.
func (c *ethereumConfig) isAnvilBased() bool {
	return c.testnetType == testvalues.EthTestnetTypeAnvil
}

func (c *ethereumConfig) isBesuQBFT() bool {
	return c.testnetType == testvalues.EthTestnetTypeBesuQBFT
}

func (c *ethereumConfig) needsPoS() bool {
	return c.testnetType == testvalues.EthTestnetTypePoS
}

// solanaConfig holds configuration for Solana chain setup.
type solanaConfig struct {
	testnetType string
}

// cosmosConfig holds config for wasm light clients deployed on Cosmos.
type cosmosConfig struct {
	lightClientType    string // dummy, full, or attestor
	wasmLightClientTag string // version tag or "local"
}

// setupConfig holds parsed configuration for all chain types.
type setupConfig struct {
	ethereum ethereumConfig
	solana   solanaConfig
	cosmos   cosmosConfig
}

// validate checks for invalid environment variable combinations and returns an error if found.
func (c *setupConfig) validate() error {
	ethTestnetType := c.ethereum.testnetType
	ethLcOnCosmos := c.cosmos.lightClientType

	switch ethTestnetType {
	case "", testvalues.EthTestnetType_None:
		return nil
	case testvalues.EthTestnetTypeAnvil:
		if ethLcOnCosmos == testvalues.EthLCOnCosmosTypeFullWasm {
			return fmt.Errorf("invalid config: ETH_TESTNET_TYPE=%s cannot use ETH_LC_ON_COSMOS=%s (Anvil doesn't have beacon chain)", ethTestnetType, ethLcOnCosmos)
		}
		return nil
	case testvalues.EthTestnetTypePoS:
		if ethLcOnCosmos == testvalues.EthLCOnCosmosTypeDummyWasm {
			return fmt.Errorf("invalid config: ETH_TESTNET_TYPE=%s cannot use ETH_LC_ON_COSMOS=%s (PoS requires actual verification)", ethTestnetType, ethLcOnCosmos)
		}
		return nil
	case testvalues.EthTestnetTypeBesuQBFT:
		switch ethLcOnCosmos {
		case testvalues.EthLCOnCosmosTypeAttestorNative:
			return nil
		case testvalues.EthLCOnCosmosTypeFullWasm:
			return fmt.Errorf("invalid config: ETH_TESTNET_TYPE=%s cannot use ETH_LC_ON_COSMOS=%s (Besu QBFT doesn't have beacon chain)", ethTestnetType, ethLcOnCosmos)
		default:
			return fmt.Errorf("invalid config: ETH_TESTNET_TYPE=%s does not support ETH_LC_ON_COSMOS=%s", ethTestnetType, ethLcOnCosmos)
		}
	default:
		return fmt.Errorf("invalid config: unsupported ETH_TESTNET_TYPE=%s", ethTestnetType)
	}
}

// Result types for parallel chain setup

type solanaSetupResult struct {
	chain *chainconfig.SolanaLocalnetChain
	err   error
}

type interchainSetupResult struct {
	chains       []ibc.Chain
	dockerClient *dockerclient.Client
	network      string
	logger       *zap.Logger
	err          error
}

type ethPosSetupResult struct {
	chain *chainconfig.EthKurtosisChain
	err   error
}

type ethBesuQBFTSetupResult struct {
	chain *chainconfig.BesuQBFTChain
	err   error
}
