// SPDX-License-Identifier: Apache-2.0

package chainconfig

import (
	"os"

	interchaintest "github.com/cosmos/interchaintest/v11"
	"github.com/cosmos/interchaintest/v11/ibc"
)

const defaultIbcGoDockerTag = "v11.0.0"

var DefaultChainSpecs = []*interchaintest.ChainSpec{
	// -- IBC-Go --
	IbcGoChainSpec("ibc-go-simd-1", "simd-1"),
}

func IbcGoChainSpec(name, chainId string) *interchaintest.ChainSpec {
	ibcGoDockerTag := os.Getenv("E2E_IBC_GO_DOCKER_TAG")
	if ibcGoDockerTag == "" {
		ibcGoDockerTag = defaultIbcGoDockerTag
	}

	return &interchaintest.ChainSpec{
		ChainConfig: ibc.ChainConfig{
			Type:    "cosmos",
			Name:    name,
			ChainID: chainId,
			Images: []ibc.DockerImage{
				{
					Repository: "ghcr.io/cosmos/ibc-go-wasm-simd", // FOR LOCAL IMAGE USE: Docker Image Name
					Version:    ibcGoDockerTag,                    // FOR LOCAL IMAGE USE: Docker Image Tag
					UIDGID:     "1025:1025",
				},
			},
			Bin:            "simd",
			Bech32Prefix:   "cosmos",
			Denom:          "stake",
			GasPrices:      "0.00stake",
			GasAdjustment:  1.3,
			EncodingConfig: SDKEncodingConfig(),
			ModifyGenesis:  defaultModifyGenesis(),
			TrustingPeriod: "508h",
			NoHostMount:    false,
		},
	}
}

func WfchainChainSpec(name, chainId string) *interchaintest.ChainSpec {
	return &interchaintest.ChainSpec{
		ChainConfig: ibc.ChainConfig{
			Type:    "cosmos",
			Name:    name,
			ChainID: chainId,
			Images: []ibc.DockerImage{
				{
					Repository: "ghcr.io/cosmos/wfchain",
					Version:    "latest",
					UIDGID:     "1025:1025",
				},
			},
			Bin:            "wfchaind",
			Bech32Prefix:   "wf",
			Denom:          "stake",
			GasPrices:      "0.00stake",
			GasAdjustment:  1.5,
			EncodingConfig: SDKEncodingConfig(),
			ModifyGenesis:  wfchainModifyGenesis(),
			TrustingPeriod: "508h",
			NoHostMount:    false,
		},
	}
}
