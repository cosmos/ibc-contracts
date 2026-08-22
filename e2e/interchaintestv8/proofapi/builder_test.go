// SPDX-License-Identifier: Apache-2.0

package proofapi

import (
	"encoding/json"
	"testing"
)

func TestEVMToEVMBuildersEmitSourceChainOnlyAtRoutingLevel(t *testing.T) {
	b := NewConfigBuilder().
		EthToEthAttested(EthToEthAttestedParams{SrcChainID: "eth-a", DstChainID: "eth-b"}).
		BesuToBesu(BesuToBesuParams{SrcChainID: "besu-a", DstChainID: "besu-b"})
	wantSource := map[string]string{ModuleEthToEth: "eth-a", ModuleBesuToBesu: "besu-a"}

	for _, module := range b.modules {
		config, err := json.Marshal(module.Config)
		if err != nil {
			t.Fatal(err)
		}
		var fields map[string]any
		if err := json.Unmarshal(config, &fields); err != nil {
			t.Fatal(err)
		}
		if _, ok := fields["src_chain_id"]; ok {
			t.Errorf("%s config contains obsolete src_chain_id", module.Name)
		}
		if module.SrcChain != wantSource[module.Name] {
			t.Errorf("%s routing src_chain = %q, want %q", module.Name, module.SrcChain, wantSource[module.Name])
		}
	}
}
