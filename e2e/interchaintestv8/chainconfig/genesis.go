// SPDX-License-Identifier: Apache-2.0

package chainconfig

import (
	"bytes"
	"encoding/json"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	govv1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"

	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

func defaultModifyGenesis() func(ibc.ChainConfig, []byte) ([]byte, error) {
	return func(chainConfig ibc.ChainConfig, genBz []byte) ([]byte, error) {
		appGenesis, err := genutiltypes.AppGenesisFromReader(bytes.NewReader(genBz))
		if err != nil {
			return nil, fmt.Errorf("failed to unmarshal genesis bytes: %w", err)
		}

		var appState genutiltypes.AppMap
		if err := json.Unmarshal(appGenesis.AppState, &appState); err != nil {
			return nil, fmt.Errorf("failed to unmarshal app state: %w", err)
		}

		// modify the gov v1 app state
		govGenBz, err := modifyGovV1AppState(chainConfig, appState[govtypes.ModuleName])
		if err != nil {
			return nil, fmt.Errorf("failed to modify gov v1 app state: %w", err)
		}

		appState[govtypes.ModuleName] = govGenBz

		// marshal the app state
		appGenesis.AppState, err = json.Marshal(appState)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal app state: %w", err)
		}

		res, err := json.MarshalIndent(appGenesis, "", "  ")
		if err != nil {
			return nil, fmt.Errorf("failed to marshal app genesis: %w", err)
		}

		return res, nil
	}
}

func wfchainModifyGenesis() func(ibc.ChainConfig, []byte) ([]byte, error) {
	return func(chainConfig ibc.ChainConfig, genBz []byte) ([]byte, error) {
		appGenesis, err := genutiltypes.AppGenesisFromReader(bytes.NewReader(genBz))
		if err != nil {
			return nil, fmt.Errorf("failed to unmarshal genesis bytes: %w", err)
		}

		var appState genutiltypes.AppMap
		if err := json.Unmarshal(appGenesis.AppState, &appState); err != nil {
			return nil, fmt.Errorf("failed to unmarshal app state: %w", err)
		}

		// modify the gov v1 app state
		govGenBz, err := modifyGovV1AppState(chainConfig, appState[govtypes.ModuleName])
		if err != nil {
			return nil, fmt.Errorf("failed to modify gov v1 app state: %w", err)
		}
		appState[govtypes.ModuleName] = govGenBz

		// modify the IFT module state to set authority to gov module
		iftGenBz, err := modifyIFTAppState(chainConfig, appState["ift"])
		if err != nil {
			return nil, fmt.Errorf("failed to modify ift app state: %w", err)
		}
		appState["ift"] = iftGenBz

		// marshal the app state
		appGenesis.AppState, err = json.Marshal(appState)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal app state: %w", err)
		}

		res, err := json.MarshalIndent(appGenesis, "", "  ")
		if err != nil {
			return nil, fmt.Errorf("failed to marshal app genesis: %w", err)
		}

		return res, nil
	}
}

// modifyIFTAppState sets the IFT module authority to governance module
func modifyIFTAppState(chainConfig ibc.ChainConfig, iftAppState []byte) ([]byte, error) {
	var iftGenesis map[string]interface{}

	if len(iftAppState) == 0 {
		iftGenesis = make(map[string]interface{})
	} else {
		if err := json.Unmarshal(iftAppState, &iftGenesis); err != nil {
			return nil, fmt.Errorf("failed to unmarshal ift genesis: %w", err)
		}
	}

	// Get or create params
	params, ok := iftGenesis["params"].(map[string]interface{})
	if !ok {
		params = make(map[string]interface{})
	}

	// Set authority to governance module address with chain's bech32 prefix
	govModuleAddr := authtypes.NewModuleAddress(govtypes.ModuleName)
	bech32Addr, err := sdk.Bech32ifyAddressBytes(chainConfig.Bech32Prefix, govModuleAddr)
	if err != nil {
		return nil, fmt.Errorf("failed to create gov module address: %w", err)
	}

	params["authority"] = bech32Addr
	iftGenesis["params"] = params

	return json.Marshal(iftGenesis)
}

// modifyGovV1AppState takes the existing gov app state and marshals it to a govv1 GenesisState.
func modifyGovV1AppState(chainConfig ibc.ChainConfig, govAppState []byte) ([]byte, error) {
	cdc := SDKEncodingConfig().Codec

	govGenesisState := &govv1.GenesisState{}
	if err := cdc.UnmarshalJSON(govAppState, govGenesisState); err != nil {
		return nil, fmt.Errorf("failed to unmarshal genesis bytes into gov genesis state: %w", err)
	}

	if govGenesisState.Params == nil {
		govGenesisState.Params = &govv1.Params{}
	}

	govGenesisState.Params.MinDeposit = sdk.NewCoins(sdk.NewCoin(chainConfig.Denom, govv1.DefaultMinDepositTokens))
	govGenesisState.Params.MaxDepositPeriod = &testvalues.MaxDepositPeriod
	govGenesisState.Params.VotingPeriod = &testvalues.VotingPeriod

	// govGenBz := MustProtoMarshalJSON(govGenesisState)

	govGenBz, err := cdc.MarshalJSON(govGenesisState)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal gov genesis state: %w", err)
	}

	return govGenBz, nil
}
