// SPDX-License-Identifier: Apache-2.0

package wasm

import "testing"

func TestRelease_WasmDownloadURL(t *testing.T) {
	release := Release{TagName: "cw-ics08-wasm-eth-v1.3.0"}

	expected := "https://github.com/cosmos/solidity-ibc-eureka/releases/download/cw-ics08-wasm-eth-v1.3.0/cw_ics08_wasm_eth.wasm.gz"
	if got := release.WasmDownloadURL(); got != expected {
		t.Fatalf("unexpected wasm download URL: got %q, want %q", got, expected)
	}
}
