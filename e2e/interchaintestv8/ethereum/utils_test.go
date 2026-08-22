// SPDX-License-Identifier: Apache-2.0

package ethereum_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
)

func TestGetEthAddressFromStdout(t *testing.T) {
	const output = `== Return ==
0: string "{\"cosmosIftConstructor\":\"0x0000000000000000000000000000000000000000\",\"erc20\":\"0x851356ae760d987e095750cceb3bc6014560891c\",\"ics20Transfer\":\"0xb7f8bc63bbcad18155201308c8f3540b07f84f5e\",\"ics26Router\":\"0x2279b7a0a67db372996a5fab50d91eaa73d2ebe6\",\"ics27Gmp\":\"0x0dcd1bf9a1b36ce34237eeafef220932846bcd82\",\"ift\":\"0x0b306bf915c4d645ff596e518faf3f9669b97016\",\"solanaIftConstructor\":\"0x0000000000000000000000000000000000000000\",\"verifierGroth16\":\"0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0\",\"verifierMock\":\"0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9\",\"verifierPlonk\":\"0xe7f1725e7734ce288f8367e1bb143e90bb3f0512\"}"`

	deployedContracts, err := ethereum.GetEthContractsFromDeployOutput(output)
	require.NoError(t, err)
	require.Equal(t, "0x2279b7a0a67db372996a5fab50d91eaa73d2ebe6", deployedContracts.Ics26Router)
	require.Equal(t, "0xb7f8bc63bbcad18155201308c8f3540b07f84f5e", deployedContracts.Ics20Transfer)
	require.Equal(t, "0x851356ae760d987e095750cceb3bc6014560891c", deployedContracts.Erc20)
}
