// SPDX-License-Identifier: Apache-2.0

package solana

// IFT PDA helpers - manually maintained because:
// 1. The IFT IDL has non-UTF8 bytes in const seeds that corrupt auto-generation
// 2. Auto-generated names would have verbose "WithAccountSeed" suffix
//
// The type `IftPDAs` and singleton `Ics27Ift` are defined in pda.go (auto-generated).
// Only the methods are defined here to avoid corruption from `just solana::generate-pda`.

import (
	"fmt"

	solanago "github.com/gagliardetto/solana-go"
)

func (iftPDAs) IftAppMintStatePDA(programID solanago.PublicKey, mint []byte) (solanago.PublicKey, uint8) {
	pda, bump, err := solanago.FindProgramAddress(
		[][]byte{[]byte("ift_app_mint_state"), mint},
		programID,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to derive Ift.IftAppMintStatePDA PDA: %v", err))
	}
	return pda, bump
}

func (iftPDAs) IftBridgePDA(programID solanago.PublicKey, mint []byte, clientId []byte) (solanago.PublicKey, uint8) {
	pda, bump, err := solanago.FindProgramAddress(
		[][]byte{[]byte("ift_bridge"), mint, clientId},
		programID,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to derive Ift.IftBridgePDA PDA: %v", err))
	}
	return pda, bump
}

func (iftPDAs) IftMintAuthorityPDA(programID solanago.PublicKey, mint []byte) (solanago.PublicKey, uint8) {
	pda, bump, err := solanago.FindProgramAddress(
		[][]byte{[]byte("ift_mint_authority"), mint},
		programID,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to derive Ift.IftMintAuthorityPDA PDA: %v", err))
	}
	return pda, bump
}

func (iftPDAs) PendingTransferPDA(programID solanago.PublicKey, mint []byte, clientId []byte, sequence []byte) (solanago.PublicKey, uint8) {
	pda, bump, err := solanago.FindProgramAddress(
		[][]byte{[]byte("pending_transfer"), mint, clientId, sequence},
		programID,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to derive Ift.PendingTransferPDA PDA: %v", err))
	}
	return pda, bump
}
