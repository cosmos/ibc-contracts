// SPDX-License-Identifier: Apache-2.0

package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"

	solanago "github.com/gagliardetto/solana-go"

	access_manager "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/accessmanager"
)

var upgradeCmd = &cobra.Command{
	Use:   "upgrade",
	Short: "Program upgrade operations",
}

var programCmd = &cobra.Command{
	Use:   "program <cluster-url> <upgrader-keypair> <target-program-id> <buffer-address> <access-manager-program-id> <program-data-address>",
	Short: "Execute program upgrade via AccessManager",
	Args:  cobra.ExactArgs(6),
	Run: func(cmd *cobra.Command, args []string) {
		clusterURL := args[0]
		upgraderKeypairPath := args[1]
		targetProgramID := solanago.MustPublicKeyFromBase58(args[2])
		bufferAddress := solanago.MustPublicKeyFromBase58(args[3])
		accessManagerProgramID := solanago.MustPublicKeyFromBase58(args[4])
		programDataAddress := solanago.MustPublicKeyFromBase58(args[5])

		upgraderWallet := loadWallet(upgraderKeypairPath)

		accessManagerPda, _, _ := solanago.FindProgramAddress(
			[][]byte{[]byte("access_manager")},
			accessManagerProgramID,
		)

		upgradeAuthorityPda, _, _ := solanago.FindProgramAddress(
			[][]byte{[]byte("upgrade_authority"), targetProgramID.Bytes()},
			accessManagerProgramID,
		)

		upgradeIx, err := access_manager.NewUpgradeProgramInstruction(
			targetProgramID,
			accessManagerPda,
			targetProgramID,
			programDataAddress,
			bufferAddress,
			upgradeAuthorityPda,
			upgraderWallet.PublicKey(),
			upgraderWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
			solanago.BPFLoaderUpgradeableProgramID,
			solanago.SysVarRentPubkey,
			solanago.SysVarClockPubkey,
		)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error building upgrade instruction: %v\n", err)
			os.Exit(1)
		}

		computeBudgetIx := createComputeBudgetInstruction(400_000)

		fmt.Println("Sending upgrade transaction...")

		sig := sendTransaction(clusterURL, upgraderWallet, []solanago.Instruction{computeBudgetIx, upgradeIx})

		fmt.Printf("✅ Upgrade transaction sent!\n")
		fmt.Printf("   Signature: %s\n", sig)
		fmt.Printf("   Explorer: https://explorer.solana.com/tx/%s?cluster=custom&customUrl=%s\n", sig, clusterURL)

		fmt.Println("\nWaiting for confirmation...")

		if waitForConfirmation(clusterURL, sig) {
			fmt.Printf("✅ Upgrade confirmed! Program %s has been upgraded.\n", targetProgramID)
		}
	},
}

var derivePdaCmd = &cobra.Command{
	Use:   "derive-pda <access-manager-program-id> <target-program-id>",
	Short: "Derive upgrade authority PDA",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		accessManagerProgramID := solanago.MustPublicKeyFromBase58(args[0])
		targetProgramID := solanago.MustPublicKeyFromBase58(args[1])

		upgradeAuthorityPda, _, _ := solanago.FindProgramAddress(
			[][]byte{[]byte("upgrade_authority"), targetProgramID.Bytes()},
			accessManagerProgramID,
		)

		fmt.Println(upgradeAuthorityPda.String())
	},
}

func init() {
	upgradeCmd.AddCommand(programCmd)
	upgradeCmd.AddCommand(derivePdaCmd)
}
