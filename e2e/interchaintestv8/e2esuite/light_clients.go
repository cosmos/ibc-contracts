// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"
	"os"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/wasm"
)

// StoreLightClient stores the light client on the given Cosmos chain and returns the hex-encoded checksum of the light client.
// For native attestor (attestor-native), returns empty string as no wasm binary is needed.
func (s *TestSuite) StoreLightClient(ctx context.Context, cosmosChain *cosmos.CosmosChain, simdRelayerUser ibc.Wallet) string {
	// Native attestor doesn't need a wasm binary
	if s.config.cosmos.lightClientType == testvalues.EthLCOnCosmosTypeAttestorNative {
		s.T().Log("Using native attestor - no wasm storage needed")
		return ""
	}

	wasmBinary := s.getWasmLightClientBinary()
	if wasmBinary == nil {
		return ""
	}
	checksum := s.PushNewWasmClientProposal(ctx, cosmosChain, simdRelayerUser, wasmBinary)
	s.Require().NotEmpty(checksum, "checksum was empty but should not have been")

	s.T().Logf("Stored wasm light client with checksum %s", checksum)

	return checksum
}

// StoreSolanaLightClient stores the Solana light client on the given Cosmos chain and returns the hex-encoded checksum of the light client.
func (s *TestSuite) StoreSolanaLightClient(ctx context.Context, cosmosChain *cosmos.CosmosChain, simdRelayerUser ibc.Wallet) string {
	// For Solana verification, we use the dummy light client for testing
	s.T().Log("Using dummy Wasm light client for Solana verification")
	wasmBinary, err := wasm.GetWasmDummyLightClient()
	s.Require().NoError(err, "Failed to get dummy Wasm light client binary")

	checksum := s.PushNewWasmClientProposal(ctx, cosmosChain, simdRelayerUser, wasmBinary)
	s.Require().NotEmpty(checksum, "checksum was empty but should not have been")

	s.T().Logf("Stored Solana light client with checksum %s", checksum)

	return checksum
}

func (s *TestSuite) getWasmLightClientBinary() *os.File {
	lightClientType := s.config.cosmos.lightClientType
	wasmTag := s.config.cosmos.wasmLightClientTag

	// Native attestor doesn't need a wasm binary
	if lightClientType == testvalues.EthLCOnCosmosTypeAttestorNative {
		s.T().Log("Using native attestor - no wasm binary needed")
		return nil
	}

	// Dummy light client (only valid for Anvil testnets)
	if lightClientType == testvalues.EthLCOnCosmosTypeDummyWasm {
		s.T().Log("Using dummy Wasm light client")
		file, err := wasm.GetWasmDummyLightClient()
		s.Require().NoError(err, "Failed to get dummy Wasm light client binary")
		return file
	}

	// Full Ethereum light client (only valid for PoS testnets)
	s.Require().Equal(testvalues.EthLCOnCosmosTypeFullWasm, lightClientType, "unexpected EthWasmType: %s", lightClientType)
	s.Require().Equal(testvalues.EthTestnetTypePoS, s.config.ethereum.testnetType, "full light client requires PoS testnet")

	if wasmTag == "" || wasmTag == testvalues.EnvValueWasmLightClientTag_Local {
		s.T().Log("Using local Wasm Ethereum light client binary")
		file, err := wasm.GetLocalWasmEthLightClient()
		s.Require().NoError(err, "Failed to get local Wasm Ethereum light client binary")
		return file
	}

	s.T().Logf("Downloading Wasm light client binary for tag %s", wasmTag)
	file, err := wasm.DownloadWasmLightClientRelease(wasm.Release{
		TagName: wasmTag,
	})
	s.Require().NoError(err, "Failed to download Wasm light client binary from release")
	return file
}
