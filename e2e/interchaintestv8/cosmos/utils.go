// SPDX-License-Identifier: Apache-2.0

package cosmos

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"

	"github.com/cosmos/cosmos-sdk/client/grpc/cmtservice"
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	abcitypes "github.com/cometbft/cometbft/abci/types"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
)

// CloneAppend returns a new slice with the contents of the provided slices.
func CloneAppend(bz []byte, tail []byte) (res []byte) {
	res = make([]byte, len(bz)+len(tail))
	copy(res, bz)
	copy(res[len(bz):], tail)
	return res
}

// BankBalanceKey returns the store key for a given address and denomination pair.
func BankBalanceKey(addr sdk.AccAddress, denom string) ([]byte, error) {
	keyCodec := collections.PairKeyCodec(sdk.AccAddressKey, collections.StringKey)
	cKey := collections.Join(addr, denom)
	return collections.EncodeKeyWithPrefix(banktypes.BalancesPrefix, keyCodec, cKey)
}

// GetEventValue retrieves the value of a specific attribute from a specific event type in the provided events.
func GetEventValue(events []abcitypes.Event, eventType, attrKey string) (string, error) {
	for _, event := range events {
		if event.Type != eventType {
			continue
		}

		for _, attr := range event.Attributes {
			if attr.Key == attrKey {
				return attr.Value, nil
			}
		}
	}

	return "", fmt.Errorf("event type %s with attribute key %s not found", eventType, attrKey)
}

// Chain wraps a CosmosChain to implement e2esuite.BlockTimeFetcher.
type Chain struct {
	Cosmos *cosmos.CosmosChain
}

// GetBlockTime returns the latest block timestamp as Unix seconds.
func (c *Chain) GetBlockTime(ctx context.Context) (int64, error) {
	header, err := FetchCosmosHeader(ctx, c.Cosmos)
	if err != nil {
		return 0, err
	}
	return header.Time.Unix(), nil
}

// FetchCosmosHeader fetches the latest header from the given chain.
func FetchCosmosHeader(ctx context.Context, chain *cosmos.CosmosChain) (*cmtservice.Header, error) {
	latestHeight, err := chain.Height(ctx)
	if err != nil {
		return nil, err
	}

	headerResp, err := e2esuite.GRPCQuery[cmtservice.GetBlockByHeightResponse](ctx, chain, &cmtservice.GetBlockByHeightRequest{
		Height: latestHeight,
	})
	if err != nil {
		return nil, err
	}

	return &headerResp.SdkBlock.Header, nil
}
