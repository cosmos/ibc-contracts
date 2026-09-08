// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"time"

	solanago "github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"
)

func parseRoleID(roleIDStr string) uint64 {
	var roleID uint64
	if _, err := fmt.Sscanf(roleIDStr, "%d", &roleID); err != nil {
		fmt.Fprintf(os.Stderr, "Invalid role ID: %v\n", err)
		os.Exit(1)
	}
	return roleID
}

func loadWallet(keypairPath string) *solanago.Wallet {
	keypairData, err := os.ReadFile(keypairPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading keypair: %v\n", err)
		os.Exit(1)
	}

	var secretKey []byte
	if err := json.Unmarshal(keypairData, &secretKey); err != nil {
		fmt.Fprintf(os.Stderr, "Error parsing keypair: %v\n", err)
		os.Exit(1)
	}

	wallet, err := solanago.WalletFromPrivateKeyBase58(solanago.PrivateKey(secretKey).String())
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error creating wallet: %v\n", err)
		os.Exit(1)
	}

	return wallet
}

func createComputeBudgetInstruction(computeUnits uint32) solanago.Instruction {
	computeBudgetData := make([]byte, 9)
	computeBudgetData[0] = 0x02
	computeBudgetData[1] = byte(computeUnits)
	computeBudgetData[2] = byte(computeUnits >> 8)
	computeBudgetData[3] = byte(computeUnits >> 16)
	computeBudgetData[4] = byte(computeUnits >> 24)

	return solanago.NewInstruction(
		solanago.MustPublicKeyFromBase58("ComputeBudget111111111111111111111111111111"),
		solanago.AccountMetaSlice{},
		computeBudgetData,
	)
}

func sendTransaction(clusterURL string, wallet *solanago.Wallet, instructions []solanago.Instruction, additionalSigners ...*solanago.Wallet) solanago.Signature {
	client := rpc.New(clusterURL)
	ctx := context.Background()

	recent, err := client.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error getting blockhash: %v\n", err)
		os.Exit(1)
	}

	tx, err := solanago.NewTransaction(
		instructions,
		recent.Value.Blockhash,
		solanago.TransactionPayer(wallet.PublicKey()),
	)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error creating transaction: %v\n", err)
		os.Exit(1)
	}

	signers := []*solanago.Wallet{wallet}
	signers = append(signers, additionalSigners...)

	_, err = tx.Sign(func(key solanago.PublicKey) *solanago.PrivateKey {
		for _, s := range signers {
			if key.Equals(s.PublicKey()) {
				return &s.PrivateKey
			}
		}
		return nil
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error signing transaction: %v\n", err)
		os.Exit(1)
	}

	sig, err := client.SendTransactionWithOpts(
		ctx,
		tx,
		rpc.TransactionOpts{
			SkipPreflight: true,
		},
	)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error sending transaction: %v\n", err)
		os.Exit(1)
	}

	return sig
}

func waitForConfirmation(clusterURL string, sig solanago.Signature) bool {
	client := rpc.New(clusterURL)
	ctx := context.Background()

	for range 30 {
		statuses, err := client.GetSignatureStatuses(ctx, false, sig)
		if err == nil && len(statuses.Value) > 0 && statuses.Value[0] != nil {
			if statuses.Value[0].Err != nil {
				fmt.Fprintf(os.Stderr, "❌ Transaction failed: %v\n", statuses.Value[0].Err)
				os.Exit(1)
			}
			if statuses.Value[0].ConfirmationStatus == rpc.ConfirmationStatusConfirmed ||
				statuses.Value[0].ConfirmationStatus == rpc.ConfirmationStatusFinalized {
				return true
			}
		}
		fmt.Print(".")
		time.Sleep(1 * time.Second)
	}

	fmt.Println("\n⚠️  Confirmation timeout - check transaction status manually")
	return false
}
