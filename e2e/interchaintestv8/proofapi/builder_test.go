// SPDX-License-Identifier: Apache-2.0

package proofapi

import (
	"encoding/json"
	"testing"
)

func TestEthToEthBuilderUsesRoutedSourceChain(t *testing.T) {
	b := NewConfigBuilder().EthToEthAttested(EthToEthAttestedParams{
		SrcChainID: "eth-a",
		DstChainID: "eth-b",
	})
	module := b.modules[0]

	config, err := json.Marshal(module.Config)
	if err != nil {
		t.Fatal(err)
	}
	var fields map[string]any
	if err := json.Unmarshal(config, &fields); err != nil {
		t.Fatal(err)
	}
	if _, ok := fields["src_chain_id"]; ok {
		t.Error("module config contains obsolete src_chain_id")
	}
	if module.SrcChain != "eth-a" {
		t.Errorf("routing src_chain = %q, want eth-a", module.SrcChain)
	}
}
