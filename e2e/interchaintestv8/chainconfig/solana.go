// SPDX-License-Identifier: Apache-2.0

package chainconfig

import (
	"context"
	"os"
	"os/exec"
	"time"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

// TODO: The agave docker image currently does not work on Apple Silicon.
// Once it does, we can migrate to using the agave docker image instead of local agave instance.
// <https://github.com/anza-xyz/agave/issues/2627>

type SolanaLocalnetChain struct {
	OsProcess *os.Process
	RPCClient *rpc.Client
	Faucet    *solana.Wallet
}

func binaryPath() string {
	return "solana-test-validator"
}

func cleanupLedgerDir() error {
	return os.RemoveAll(testvalues.SolanaLedgerDir)
}

func StartLocalnet(context.Context) (SolanaLocalnetChain, error) {
	solanaChain := SolanaLocalnetChain{}
	solanaChain.Faucet = solana.NewWallet()

	if err := cleanupLedgerDir(); err != nil {
		return SolanaLocalnetChain{}, err
	}

	cmd := exec.Command(binaryPath(), "--reset", "--mint", solanaChain.Faucet.PublicKey().String(), "--ledger", testvalues.SolanaLedgerDir)
	if err := cmd.Start(); err != nil {
		return SolanaLocalnetChain{}, err
	}

	// Wait for the Solana localnet to start
	time.Sleep(6 * time.Second)

	solanaChain.OsProcess = cmd.Process
	solanaChain.RPCClient = rpc.New(rpc.LocalNet.RPC)

	return solanaChain, nil
}

func (s SolanaLocalnetChain) Destroy() error {
	if s.OsProcess != nil {
		if err := s.OsProcess.Kill(); err != nil {
			return err
		}

		if err := cleanupLedgerDir(); err != nil {
			return err
		}
	}

	return nil
}
