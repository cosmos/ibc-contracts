package main

import (
	"context"
	"os"
	"testing"

	ethcommon "github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/require"

	interchaintest "github.com/cosmos/interchaintest/v11"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

func TestBesuQBFTChainBringUpAndDeploy(t *testing.T) {
	ctx := context.Background()

	_, networkID := interchaintest.DockerSetup(t)

	chain, err := chainconfig.SpinUpBesuQBFT(ctx, chainconfig.BesuQBFTParams{
		ChainID:             1337,
		Subnet:              "10.42.0.0/16",
		Gateway:             "10.42.0.1",
		ValidatorIPs:        [4]string{"10.42.0.2", "10.42.0.3", "10.42.0.4", "10.42.0.5"},
		DockerRPCAlias:      "besu-qbft-rpc",
		InterchainNetworkID: networkID,
	})
	require.NoError(t, err)
	t.Cleanup(func() {
		cleanupCtx := context.Background()
		if t.Failed() {
			_ = chain.DumpLogs(cleanupCtx)
		}
		chain.Destroy(cleanupCtx)
	})

	cwd, err := os.Getwd()
	require.NoError(t, err)
	require.NoError(t, os.Chdir("../.."))
	t.Cleanup(func() {
		require.NoError(t, os.Chdir(cwd))
	})

	require.Equal(t, "http://besu-qbft-rpc:8545", chain.DockerRPC)

	eth, err := ethereum.NewEthereum(ctx, chain.RPC, nil, chain.Faucet)
	require.NoError(t, err)

	latestBlock, err := eth.RPCClient.BlockNumber(ctx)
	require.NoError(t, err)
	require.Greater(t, latestBlock, uint64(0))

	var validators []ethcommon.Address
	err = eth.RPCClient.Client().CallContext(ctx, &validators, "qbft_getValidatorsByBlockNumber", "latest")
	require.NoError(t, err)
	require.Len(t, validators, 4)

	stdout, err := eth.ForgeScript(chain.Faucet, testvalues.E2EDeployScriptPath)
	require.NoError(t, err)

	contracts, err := ethereum.GetEthContractsFromDeployOutput(string(stdout))
	require.NoError(t, err)
	require.NotEmpty(t, contracts.Ics26Router)
	require.NotEmpty(t, contracts.Ics20Transfer)
	require.NotEmpty(t, contracts.Erc20)

	proof, err := eth.GetProof(ctx, ethcommon.HexToAddress(contracts.Ics26Router), nil, "latest")
	require.NoError(t, err)
	require.NotEmpty(t, proof.AccountProof)
}
