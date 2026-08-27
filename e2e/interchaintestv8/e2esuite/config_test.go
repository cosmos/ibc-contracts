// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"testing"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

func TestSetupConfigValidateEthereumCompatibility(t *testing.T) {
	const unknown = "unknown"

	tests := []struct {
		name        string
		testnetType string
		clientType  string
		wantErr     bool
	}{
		{"anvil dummy", testvalues.EthTestnetTypeAnvil, testvalues.EthLCOnCosmosTypeDummyWasm, false},
		{"anvil attestor", testvalues.EthTestnetTypeAnvil, testvalues.EthLCOnCosmosTypeAttestorNative, false},
		{"pos full", testvalues.EthTestnetTypePoS, testvalues.EthLCOnCosmosTypeFullWasm, false},
		{"pos attestor", testvalues.EthTestnetTypePoS, testvalues.EthLCOnCosmosTypeAttestorNative, false},
		{"besu attestor", testvalues.EthTestnetTypeBesuQBFT, testvalues.EthLCOnCosmosTypeAttestorNative, false},
		{"anvil full", testvalues.EthTestnetTypeAnvil, testvalues.EthLCOnCosmosTypeFullWasm, true},
		{"pos dummy", testvalues.EthTestnetTypePoS, testvalues.EthLCOnCosmosTypeDummyWasm, true},
		{"besu dummy", testvalues.EthTestnetTypeBesuQBFT, testvalues.EthLCOnCosmosTypeDummyWasm, true},
		{"besu full", testvalues.EthTestnetTypeBesuQBFT, testvalues.EthLCOnCosmosTypeFullWasm, true},
		{"active empty client", testvalues.EthTestnetTypeAnvil, "", true},
		{"active unknown client", testvalues.EthTestnetTypePoS, unknown, true},
		{"unknown testnet", unknown, testvalues.EthLCOnCosmosTypeAttestorNative, true},
		{"disabled empty", "", unknown, false},
		{"disabled none", testvalues.EthTestnetType_None, unknown, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cfg := setupConfig{
				ethereum: ethereumConfig{testnetType: tt.testnetType},
				cosmos:   cosmosConfig{lightClientType: tt.clientType},
			}
			if err := cfg.validate(); (err != nil) != tt.wantErr {
				t.Fatalf("validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}

	for _, clientType := range []string{"", unknown} {
		cfg := setupConfig{
			ethereum: ethereumConfig{testnetType: testvalues.EthTestnetTypeAnvil, anvilCount: 2},
			cosmos:   cosmosConfig{lightClientType: clientType},
		}
		if err := cfg.validate(); err != nil {
			t.Errorf("multiple Anvil chains with ETH_LC_ON_COSMOS=%q: %v", clientType, err)
		}
	}
}
