// SPDX-License-Identifier: Apache-2.0

package solana

import (
	"context"
	"fmt"
	"time"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/programs/system"
	"github.com/gagliardetto/solana-go/rpc"
)

// WaitForClusterReady waits for the Solana cluster to be fully initialized
func (s *Solana) WaitForClusterReady(ctx context.Context, timeout time.Duration) error {
	deadline := time.Now().Add(timeout)

	for time.Now().Before(deadline) {
		// Check 1: Can we get the latest blockhash?
		_, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
		if err != nil {
			time.Sleep(1 * time.Second)
			continue
		}

		// Check 2: Can we get the slot?
		slot, err := s.RPCClient.GetSlot(ctx, rpc.CommitmentConfirmed)
		if err != nil || slot < 5 {
			time.Sleep(1 * time.Second)
			continue
		}

		// Check 3: Is the faucet account funded and available?
		if s.Faucet != nil {
			balance, err := s.RPCClient.GetBalance(ctx, s.Faucet.PublicKey(), rpc.CommitmentConfirmed)
			if err != nil {
				time.Sleep(1 * time.Second)
				continue
			}

			// Ensure faucet has at least 10 SOL for funding operations
			minBalance := uint64(10_000_000_000) // 10 SOL in lamports
			if balance.Value < minBalance {
				time.Sleep(1 * time.Second)
				continue
			}
		}

		// Check 4: Can we get the version? (ensures RPC is fully responsive)
		_, err = s.RPCClient.GetVersion(ctx)
		if err != nil {
			time.Sleep(1 * time.Second)
			continue
		}

		// All checks passed
		return nil
	}

	return fmt.Errorf("cluster not ready after %v", timeout)
}

// FundUserWithRetry funds a user with retry logic and confirmed commitment
func (s *Solana) FundUserWithRetry(ctx context.Context, pubkey solana.PublicKey, amount uint64, retries int) (solana.Signature, error) {
	var lastErr error

	for i := range retries {
		// Wait a bit before retry (except first attempt)
		if i > 0 {
			time.Sleep(time.Duration(i) * time.Second)
		}

		// Check faucet balance first
		faucetBalance, err := s.RPCClient.GetBalance(ctx, s.Faucet.PublicKey(), rpc.CommitmentConfirmed)
		if err != nil {
			lastErr = fmt.Errorf("failed to get faucet balance: %w", err)
			continue
		}

		if faucetBalance.Value < amount {
			lastErr = fmt.Errorf("insufficient faucet balance: %d < %d", faucetBalance.Value, amount)
			continue
		}

		// Get latest blockhash with confirmed commitment for faster execution
		recent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
		if err != nil {
			lastErr = fmt.Errorf("failed to get blockhash: %w", err)
			continue
		}

		// Create transfer transaction
		tx, err := solana.NewTransaction(
			[]solana.Instruction{
				system.NewTransferInstruction(
					amount,
					s.Faucet.PublicKey(),
					pubkey,
				).Build(),
			},
			recent.Value.Blockhash,
			solana.TransactionPayer(s.Faucet.PublicKey()),
		)
		if err != nil {
			lastErr = fmt.Errorf("failed to create transaction: %w", err)
			continue
		}

		// Sign and broadcast with confirmed commitment for faster wallet setup
		sig, err := s.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.Faucet)
		if err == nil {
			// Verify the transfer succeeded with confirmed commitment
			balance, err := s.RPCClient.GetBalance(ctx, pubkey, rpc.CommitmentConfirmed)
			if err == nil && balance.Value >= amount {
				return sig, nil
			}
		}

		lastErr = err
	}

	return solana.Signature{}, fmt.Errorf("failed to fund user after %d retries: %w", retries, lastErr)
}
