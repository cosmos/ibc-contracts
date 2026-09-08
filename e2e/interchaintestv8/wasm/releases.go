// SPDX-License-Identifier: Apache-2.0

package wasm

type Release struct {
	TagName string `json:"tag_name"`
}

func (r Release) BaseDownloadURL() string {
	return "https://github.com/cosmos/solidity-ibc-eureka/releases/download/" + r.TagName
}

func (r Release) WasmDownloadURL() string {
	return r.BaseDownloadURL() + "/cw_ics08_wasm_eth.wasm.gz"
}
