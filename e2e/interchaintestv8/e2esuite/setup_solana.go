// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"
	"fmt"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

func (s *TestSuite) setupSolanaAsync(ctx context.Context, cfg setupConfig, resultCh chan<- solanaSetupResult) {
	var result solanaSetupResult
	defer func() { resultCh <- result }()

	switch cfg.solana.testnetType {
	case testvalues.SolanaTestnetType_Localnet:
		chain, err := chainconfig.StartLocalnet(ctx)
		if err != nil {
			result.err = err
			return
		}
		result.chain = &chain
	case testvalues.SolanaTestnetType_None, "":
		// No Solana
	default:
		result.err = fmt.Errorf("unknown Solana testnet type: %s", cfg.solana.testnetType)
	}
}

func (s *TestSuite) processSolanaResult(chain *chainconfig.SolanaLocalnetChain) {
	if chain == nil {
		return
	}

	s.T().Cleanup(func() {
		if err := chain.Destroy(); err != nil {
			s.T().Logf("Failed to destroy Solana localnet: %v", err)
		}
	})

	var err error
	s.Solana.Chain, err = solana.NewLocalnetSolana(chain.Faucet)
	s.Require().NoError(err, "Failed to create Solana client")
}
