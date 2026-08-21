// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"testing"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

func TestSetupConfigRejectsBesuQBFTWithDummyLightClient(t *testing.T) {
	cfg := setupConfig{
		ethereum: ethereumConfig{testnetType: testvalues.EthTestnetTypeBesuQBFT},
		cosmos:   cosmosConfig{lightClientType: testvalues.EthLCOnCosmosTypeDummyWasm},
	}

	if err := cfg.validate(); err == nil {
		t.Fatal("expected Besu QBFT with a dummy light client to be rejected")
	}
}
