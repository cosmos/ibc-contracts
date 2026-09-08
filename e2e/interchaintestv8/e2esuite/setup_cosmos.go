// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"
)

func (s *TestSuite) setupCosmosChains(ctx context.Context, chains []ibc.Chain) {
	for _, chain := range chains {
		if cosmosChain, ok := chain.(*cosmos.CosmosChain); ok {
			s.Cosmos.Chains = append(s.Cosmos.Chains, cosmosChain)
		}
	}

	if len(s.Cosmos.Chains) == 0 {
		return
	}

	// map all query request types to their gRPC method paths for cosmos chains
	s.Require().NoError(populateQueryReqToPath(ctx, s.Cosmos.Chains[0]))

	// Fund user accounts
	for _, chain := range s.Cosmos.Chains {
		s.Cosmos.Users = append(s.Cosmos.Users, s.CreateAndFundCosmosUser(ctx, chain))
	}

	s.Cosmos.proposalIDs = make(map[string]uint64)
	for _, chain := range s.Cosmos.Chains {
		s.Cosmos.proposalIDs[chain.Config().ChainID] = 1
	}
}
