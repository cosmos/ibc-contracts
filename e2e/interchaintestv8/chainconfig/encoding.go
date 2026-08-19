// SPDX-License-Identifier: Apache-2.0

package chainconfig

import (
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdktestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/authz"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	govv1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	govv1beta1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	upgradetypes "github.com/cosmos/cosmos-sdk/x/upgrade/types"

	ibcwasmtypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/v11/types"
	gmptypes "github.com/cosmos/ibc-go/v11/modules/apps/27-gmp/types"
	icacontrollertypes "github.com/cosmos/ibc-go/v11/modules/apps/27-interchain-accounts/controller/types"
	icahosttypes "github.com/cosmos/ibc-go/v11/modules/apps/27-interchain-accounts/host/types"
	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"
	v7migrations "github.com/cosmos/ibc-go/v11/modules/core/02-client/migrations/v7"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	connectiontypes "github.com/cosmos/ibc-go/v11/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v11/modules/core/04-channel/types"
	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"
	solomachine "github.com/cosmos/ibc-go/v11/modules/light-clients/06-solomachine"
	ibctmtypes "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"

	proposaltypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/paramsproposal"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/attestations"
	ifttypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/wfchain/ift"
	tokenfactorytypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/wfchain/tokenfactory"
)

// Codec returns the global E2E protobuf codec.
func Codec() *codec.ProtoCodec {
	cdc, _ := codecAndEncodingConfig()
	return cdc
}

// SDKEncodingConfig returns the global E2E encoding config.
func SDKEncodingConfig() *sdktestutil.TestEncodingConfig {
	_, cfg := codecAndEncodingConfig()
	return &sdktestutil.TestEncodingConfig{
		InterfaceRegistry: cfg.InterfaceRegistry,
		Codec:             cfg.Codec,
		TxConfig:          cfg.TxConfig,
		Amino:             cfg.Amino,
	}
}

// codecAndEncodingConfig returns the codec and encoding config used in the E2E tests.
// Note: any new types added to the codec must be added here.
func codecAndEncodingConfig() (*codec.ProtoCodec, sdktestutil.TestEncodingConfig) {
	cfg := sdktestutil.MakeTestEncodingConfig()

	// ibc types
	gmptypes.RegisterInterfaces(cfg.InterfaceRegistry)
	icacontrollertypes.RegisterInterfaces(cfg.InterfaceRegistry)
	icahosttypes.RegisterInterfaces(cfg.InterfaceRegistry)
	solomachine.RegisterInterfaces(cfg.InterfaceRegistry)
	v7migrations.RegisterInterfaces(cfg.InterfaceRegistry)
	transfertypes.RegisterInterfaces(cfg.InterfaceRegistry)
	clienttypes.RegisterInterfaces(cfg.InterfaceRegistry)
	clienttypesv2.RegisterInterfaces(cfg.InterfaceRegistry)
	channeltypes.RegisterInterfaces(cfg.InterfaceRegistry)
	connectiontypes.RegisterInterfaces(cfg.InterfaceRegistry)
	ibctmtypes.RegisterInterfaces(cfg.InterfaceRegistry)
	ibcwasmtypes.RegisterInterfaces(cfg.InterfaceRegistry)
	channeltypesv2.RegisterInterfaces(cfg.InterfaceRegistry)
	attestations.RegisterInterfaces(cfg.InterfaceRegistry)
	gmptypes.RegisterInterfaces(cfg.InterfaceRegistry)

	// all other types
	upgradetypes.RegisterInterfaces(cfg.InterfaceRegistry)
	banktypes.RegisterInterfaces(cfg.InterfaceRegistry)
	govv1beta1.RegisterInterfaces(cfg.InterfaceRegistry)
	govv1.RegisterInterfaces(cfg.InterfaceRegistry)
	authtypes.RegisterInterfaces(cfg.InterfaceRegistry)
	cryptocodec.RegisterInterfaces(cfg.InterfaceRegistry)
	proposaltypes.RegisterInterfaces(cfg.InterfaceRegistry)
	authz.RegisterInterfaces(cfg.InterfaceRegistry)
	txtypes.RegisterInterfaces(cfg.InterfaceRegistry)

	// wfchain IFT and TokenFactory types
	registerWfchainTypes(cfg.InterfaceRegistry)

	cdc := codec.NewProtoCodec(cfg.InterfaceRegistry)
	return cdc, cfg
}

// registerWfchainTypes registers wfchain IFT and TokenFactory message types
func registerWfchainTypes(registry codectypes.InterfaceRegistry) {
	registry.RegisterImplementations((*sdk.Msg)(nil),
		&ifttypes.MsgRegisterIFTBridge{},
		&ifttypes.MsgRemoveIFTBridge{},
		&ifttypes.MsgIFTTransfer{},
		&ifttypes.MsgIFTMint{},
		&ifttypes.MsgUpdateParams{},
		&tokenfactorytypes.MsgCreateDenom{},
		&tokenfactorytypes.MsgMint{},
		&tokenfactorytypes.MsgBurn{},
	)
}
