// SPDX-License-Identifier: Apache-2.0

package solana

import (
	"bytes"
	"context"
	"encoding/binary"
	"fmt"
	"slices"
	"testing"
	"time"

	bin "github.com/gagliardetto/binary"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"
	"github.com/gagliardetto/solana-go/rpc/ws"

	"github.com/cosmos/interchaintest/v11/testutil"

	ics26router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

type Solana struct {
	RPCClient *rpc.Client
	WSClient  *ws.Client
	Faucet    *solana.Wallet
	RPCURL    string
}

func NewSolana(rpcURL, wsURL string, faucet *solana.Wallet) (Solana, error) {
	wsClient, err := ws.Connect(context.TODO(), wsURL)
	if err != nil {
		return Solana{}, err
	}

	return Solana{
		RPCClient: rpc.New(rpcURL),
		WSClient:  wsClient,
		Faucet:    faucet,
		RPCURL:    rpcURL,
	}, nil
}

func NewLocalnetSolana(faucet *solana.Wallet) (Solana, error) {
	return NewSolana(rpc.LocalNet.RPC, rpc.LocalNet.WS, faucet)
}

// NewTransactionFromInstructions creates a new tx from the given transactions
// Uses Confirmed blockhash for faster transaction construction.
// The blockhash will be refreshed by SignAndBroadcastTxWithRetry before broadcasting.
func (s *Solana) NewTransactionFromInstructions(payerPubKey solana.PublicKey, instructions ...solana.Instruction) (*solana.Transaction, error) {
	recent, err := s.RPCClient.GetLatestBlockhash(context.TODO(), rpc.CommitmentConfirmed)
	if err != nil {
		return nil, err
	}

	return solana.NewTransaction(
		instructions,
		recent.Value.Blockhash,
		solana.TransactionPayer(payerPubKey),
	)
}

// SignTx signs a transaction with the provided signers.
// It modifies the transaction in place and returns the signatures.
func (s *Solana) SignTx(ctx context.Context, tx *solana.Transaction, signers ...*solana.Wallet) ([]solana.Signature, error) {
	if len(signers) == 0 {
		return nil, fmt.Errorf("no signers provided")
	}

	signerFn := func(key solana.PublicKey) *solana.PrivateKey {
		keyIdx := slices.IndexFunc(signers, func(signer *solana.Wallet) bool {
			return signer.PublicKey().Equals(key)
		})
		if keyIdx == -1 {
			panic(fmt.Sprintf("signer %s not found in provided signers", key))
		}
		return &signers[keyIdx].PrivateKey
	}

	return tx.Sign(signerFn)
}

// Higher numbers indicate higher confirmation levels.
func confirmationStatusLevel(status rpc.ConfirmationStatusType) int {
	switch status {
	case rpc.ConfirmationStatusProcessed:
		return 1
	case rpc.ConfirmationStatusConfirmed:
		return 2
	case rpc.ConfirmationStatusFinalized:
		return 3
	default:
		return 0
	}
}

// Waits for transaction reaching status
func (s *Solana) WaitForTxStatus(txSig solana.Signature, status rpc.ConfirmationStatusType) error {
	return testutil.WaitForCondition(time.Second*30, time.Second, func() (bool, error) {
		out, err := s.RPCClient.GetSignatureStatuses(context.TODO(), false, txSig)
		if err != nil {
			return false, err
		}

		if len(out.Value) == 0 || out.Value[0] == nil {
			return false, nil
		}

		if out.Value[0].Err != nil {
			return false, fmt.Errorf("transaction %s failed with error: %s", txSig, out.Value[0].Err)
		}

		if confirmationStatusLevel(out.Value[0].ConfirmationStatus) >= confirmationStatusLevel(status) {
			return true, nil
		}

		return false, nil
	})
}

func (s *Solana) CreateAndFundWallet() (*solana.Wallet, error) {
	wallet := solana.NewWallet()
	if _, err := s.FundUserWithRetry(context.TODO(), wallet.PublicKey(), testvalues.InitialSolBalance, 5); err != nil {
		return nil, err
	}
	return wallet, nil
}

// WaitForProgramAvailability waits for a program to become available with default timeout
func (s *Solana) WaitForProgramAvailability(ctx context.Context, programID solana.PublicKey) bool {
	return s.WaitForProgramAvailabilityWithTimeout(ctx, programID, 30)
}

// WaitForProgramAvailabilityWithTimeout waits for a program to become available with specified timeout
func (s *Solana) WaitForProgramAvailabilityWithTimeout(ctx context.Context, programID solana.PublicKey, timeoutSeconds int) bool {
	for range timeoutSeconds {
		// Use confirmed commitment to match relayer read commitment level
		accountInfo, err := s.RPCClient.GetAccountInfoWithOpts(ctx, programID, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		if err == nil && accountInfo.Value != nil && accountInfo.Value.Executable {
			return true
		}
		time.Sleep(1 * time.Second)
	}
	return false
}

// SignAndBroadcastTxWithRetry signs, broadcasts, and waits for the specified commitment level with retry (30s timeout).
// Commitment level must be explicitly specified:
//   - rpc.CommitmentFinalized: Waits for supermajority + 31 confirmations (~10-30s). Use when subsequent code immediately depends on finalized state.
//   - rpc.CommitmentConfirmed: Waits for supermajority confirmation (~1-5s). Use for non-critical setup operations.
//   - rpc.CommitmentProcessed: Waits for the transaction to be processed (~instant). Use with caution, minimal guarantees.
func (s *Solana) SignAndBroadcastTxWithRetry(ctx context.Context, tx *solana.Transaction, commitment rpc.CommitmentType, signers ...*solana.Wallet) (solana.Signature, error) {
	return s.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, commitment, 30, signers...)
}

// SignAndBroadcastTxWithRetryAndTimeout signs, broadcasts, and waits for the specified commitment level with custom timeout.
func (s *Solana) SignAndBroadcastTxWithRetryAndTimeout(ctx context.Context, tx *solana.Transaction, commitment rpc.CommitmentType, timeoutSeconds int, signers ...*solana.Wallet) (solana.Signature, error) {
	var lastErr error
	for range timeoutSeconds {
		// Fetch blockhash with requested commitment level
		recent, err := s.RPCClient.GetLatestBlockhash(ctx, commitment)
		if err != nil {
			lastErr = fmt.Errorf("failed to get latest blockhash: %w", err)
			time.Sleep(1 * time.Second)
			continue
		}
		tx.Message.RecentBlockhash = recent.Value.Blockhash

		// Broadcast with skip preflight and wait for requested confirmation status
		// CommitmentType and ConfirmationStatusType are compatible (both string types with same values)
		sig, err := s.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusType(commitment), signers...)
		if err == nil {
			return sig, nil
		}
		lastErr = err
		time.Sleep(1 * time.Second)
	}
	return solana.Signature{}, fmt.Errorf("transaction broadcast timed out after %d seconds: %w", timeoutSeconds, lastErr)
}

// SignAndBroadcastTxWithOpts signs with one or more signers, broadcasts (skipping preflight), and waits for requested confirmation status.
// This is the unified low-level function for broadcasting with specific commitment requirements.
func (s *Solana) SignAndBroadcastTxWithOpts(ctx context.Context, tx *solana.Transaction, status rpc.ConfirmationStatusType, signers ...*solana.Wallet) (solana.Signature, error) {
	_, err := s.SignTx(ctx, tx, signers...)
	if err != nil {
		return solana.Signature{}, err
	}

	sig, err := s.RPCClient.SendTransactionWithOpts(
		ctx,
		tx,
		rpc.TransactionOpts{
			SkipPreflight: true,
		},
	)
	if err != nil {
		return solana.Signature{}, err
	}

	err = s.WaitForTxStatus(sig, status)
	if err != nil {
		// Return the signature even on error so logs can be fetched
		return sig, err
	}

	return sig, err
}

// WaitForBalanceChange waits for an account balance to change from the initial value
func (s *Solana) WaitForBalanceChange(ctx context.Context, account solana.PublicKey, initialBalance uint64) (uint64, bool) {
	return s.WaitForBalanceChangeWithTimeout(ctx, account, initialBalance, 30)
}

// WaitForBalanceChangeWithTimeout waits for an account balance to change with specified timeout
func (s *Solana) WaitForBalanceChangeWithTimeout(ctx context.Context, account solana.PublicKey, initialBalance uint64, timeoutSeconds int) (uint64, bool) {
	for range timeoutSeconds {
		balanceResp, err := s.RPCClient.GetBalance(ctx, account, rpc.CommitmentConfirmed)
		if err == nil {
			currentBalance := balanceResp.Value
			if currentBalance != initialBalance {
				return currentBalance, true
			}
		}
		time.Sleep(1 * time.Second)
	}
	return initialBalance, false
}

// ComputeBudgetProgramID returns the Solana Compute Budget program ID
func ComputeBudgetProgramID() solana.PublicKey {
	return solana.MustPublicKeyFromBase58("ComputeBudget111111111111111111111111111111")
}

// NewComputeBudgetInstruction creates a SetComputeUnitLimit instruction to increase available compute units
func NewComputeBudgetInstruction(computeUnits uint32) solana.Instruction {
	data := make([]byte, 5)
	data[0] = 0x02
	binary.LittleEndian.PutUint32(data[1:], computeUnits)

	return solana.NewInstruction(
		ComputeBudgetProgramID(),
		solana.AccountMetaSlice{},
		data,
	)
}

// Returns the ALT address. Requires at least one account.
func (s *Solana) CreateAddressLookupTable(ctx context.Context, authority *solana.Wallet, accounts []solana.PublicKey) (solana.PublicKey, error) {
	if len(accounts) == 0 {
		return solana.PublicKey{}, fmt.Errorf("at least one account is required for ALT")
	}

	slot, err := s.RPCClient.GetSlot(ctx, rpc.CommitmentProcessed)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to get slot: %w", err)
	}

	// Derive Address Lookup Table PDA
	slotBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(slotBytes, slot)
	altAddress, bumpSeed, err := solana.FindProgramAddress(
		[][]byte{authority.PublicKey().Bytes(), slotBytes},
		solana.AddressLookupTableProgramID,
	)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to derive address lookup table PDA: %w", err)
	}

	var createBuf bytes.Buffer
	encoder := bin.NewBinEncoder(&createBuf)
	mustWrite(encoder.WriteUint32(0, bin.LE))
	mustWrite(encoder.WriteUint64(slot, bin.LE))
	mustWrite(encoder.WriteUint8(bumpSeed))
	createInstructionData := createBuf.Bytes()

	createAltIx := solana.NewInstruction(
		solana.AddressLookupTableProgramID,
		solana.AccountMetaSlice{
			solana.Meta(altAddress).WRITE(),
			solana.Meta(authority.PublicKey()).WRITE().SIGNER(),
			solana.Meta(authority.PublicKey()).WRITE().SIGNER(),
			solana.Meta(solana.SystemProgramID),
		},
		createInstructionData,
	)

	createTx, err := s.NewTransactionFromInstructions(authority.PublicKey(), createAltIx)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to create ALT transaction: %w", err)
	}

	_, err = s.SignAndBroadcastTxWithRetry(ctx, createTx, rpc.CommitmentConfirmed, authority)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to create ALT: %w", err)
	}

	var extendBuf bytes.Buffer
	extendEncoder := bin.NewBinEncoder(&extendBuf)
	mustWrite(extendEncoder.WriteUint32(2, bin.LE))
	mustWrite(extendEncoder.WriteUint64(uint64(len(accounts)), bin.LE))
	for _, acc := range accounts {
		mustWrite(extendEncoder.WriteBytes(acc.Bytes(), false))
	}
	extendInstructionData := extendBuf.Bytes()

	extendAltIx := solana.NewInstruction(
		solana.AddressLookupTableProgramID,
		solana.AccountMetaSlice{
			solana.Meta(altAddress).WRITE(),
			solana.Meta(authority.PublicKey()).WRITE().SIGNER(),
			solana.Meta(authority.PublicKey()).WRITE().SIGNER(),
			solana.Meta(solana.SystemProgramID),
		},
		extendInstructionData,
	)

	extendTx, err := s.NewTransactionFromInstructions(authority.PublicKey(), extendAltIx)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to create extend ALT transaction: %w", err)
	}

	_, err = s.SignAndBroadcastTxWithRetry(ctx, extendTx, rpc.CommitmentConfirmed, authority)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to extend ALT: %w", err)
	}

	return altAddress, nil
}

// mustWrite wraps encoder write calls and panics on error (should never happen with bytes.Buffer)
func mustWrite(err error) {
	if err != nil {
		panic(fmt.Sprintf("unexpected encoding error: %v", err))
	}
}

// LogTransactionDetails fetches and logs detailed information about a transaction
// including compute units consumed, error details, and program logs
func (s *Solana) LogTransactionDetails(ctx context.Context, t *testing.T, sig solana.Signature, context string) {
	t.Helper()
	t.Logf("=== Transaction Details: %s ===", context)
	t.Logf("Transaction signature: %s", sig)

	version := uint64(0)
	txDetails, err := s.RPCClient.GetTransaction(ctx, sig, &rpc.GetTransactionOpts{
		Encoding:                       solana.EncodingBase64,
		Commitment:                     rpc.CommitmentConfirmed,
		MaxSupportedTransactionVersion: &version,
	})
	if err != nil {
		t.Logf("❌ Failed to fetch transaction details: %v", err)
		return
	}

	if txDetails == nil || txDetails.Meta == nil {
		t.Logf("⚠️  Transaction details not available (may still be processing)")
		return
	}

	// Log compute units consumed
	if txDetails.Meta.ComputeUnitsConsumed != nil {
		t.Logf("⚙️  Compute units consumed: %d", *txDetails.Meta.ComputeUnitsConsumed)
	}

	t.Logf("💰 Fee: %d lamports (%.9f SOL)", txDetails.Meta.Fee, float64(txDetails.Meta.Fee)/1e9)

	if txDetails.Meta.Err != nil {
		t.Logf("❌ Transaction error: %+v", txDetails.Meta.Err)
	} else {
		t.Logf("✅ Transaction succeeded")
	}

	if len(txDetails.Meta.LogMessages) > 0 {
		t.Logf("📋 Program Logs (%d messages):", len(txDetails.Meta.LogMessages))
		for i, log := range txDetails.Meta.LogMessages {
			t.Logf("  [%d] %s", i, log)
		}
	}
	t.Logf("=====================================")
}

func (s *Solana) GetSolanaClockTime(ctx context.Context) (int64, error) {
	// Use confirmed commitment to match relayer read commitment level
	accountInfo, err := s.RPCClient.GetAccountInfoWithOpts(ctx, solana.SysVarClockPubkey, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	if err != nil {
		return 0, fmt.Errorf("failed to get clock sysvar account: %w", err)
	}
	if accountInfo.Value == nil {
		return 0, fmt.Errorf("clock sysvar account is nil")
	}

	data := accountInfo.Value.Data.GetBinary()
	if len(data) < 40 {
		return 0, fmt.Errorf("clock sysvar data too short: expected >= 40 bytes, got %d", len(data))
	}

	unixTimestamp := int64(binary.LittleEndian.Uint64(data[32:40]))
	return unixTimestamp, nil
}

// GetBlockTime returns the latest block timestamp as Unix seconds from the Solana clock sysvar.
func (s *Solana) GetBlockTime(ctx context.Context) (int64, error) {
	return s.GetSolanaClockTime(ctx)
}

// GetProgramDataAddress derives the ProgramData account address for an upgradeable program
func GetProgramDataAddress(programID solana.PublicKey) (solana.PublicKey, error) {
	pda, _, err := solana.FindProgramAddress(
		[][]byte{programID.Bytes()},
		solana.BPFLoaderUpgradeableProgramID,
	)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to derive program data address: %w", err)
	}
	return pda, nil
}

// GetWriteAcknowledgementEvents fetches a transaction and parses WriteAcknowledgementEvents from its logs
func (s *Solana) GetWriteAcknowledgementEvents(ctx context.Context, sig solana.Signature) ([]*ics26router.Ics26RouterEventsWriteAcknowledgementEvent, error) {
	version := uint64(0)
	txDetails, err := s.RPCClient.GetTransaction(ctx, sig, &rpc.GetTransactionOpts{
		Encoding:                       solana.EncodingBase64,
		Commitment:                     rpc.CommitmentConfirmed,
		MaxSupportedTransactionVersion: &version,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to fetch transaction: %w", err)
	}

	if txDetails == nil || txDetails.Meta == nil {
		return nil, fmt.Errorf("transaction details or meta is nil")
	}

	return ParseWriteAcknowledgementEventsFromLogs(txDetails.Meta.LogMessages)
}
