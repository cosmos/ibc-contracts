// SPDX-License-Identifier: Apache-2.0

// Package ics07_tendermint_patches contains manually patched instruction builders to work around
// bugs in anchor-go for the ics07_tendermint program.
//
// This package exists separately from the auto-generated packages to survive `just solana::generate-solana-types`
// which deletes and regenerates the entire packages/go-anchor/{package} directories.
//
// BUG: anchor-go (https://github.com/gagliardetto/anchor-go) has a bug where instructions
// with no arguments incorrectly omit the 8-byte discriminator from the instruction data.
// This causes the Solana program to fail because it cannot identify which instruction to execute.
//
// The generated NewCleanupIncompleteUploadInstruction function sets instruction data to `nil`
// instead of encoding the discriminator. This file provides a corrected version.
//
// Tracking: This should be removed once the anchor-go bug is fixed upstream.
// See: https://github.com/gagliardetto/anchor-go/issues/50
// See: https://github.com/gagliardetto/anchor-go/pull/52
package ics07_tendermint_patches

import (
	"bytes"
	"fmt"

	binary "github.com/gagliardetto/binary"
	solanago "github.com/gagliardetto/solana-go"

	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
)

// NewCleanupIncompleteUploadInstruction builds a "cleanup_incomplete_upload" instruction
// with the correct discriminator encoding.
//
// This is a patched version that correctly includes the instruction discriminator.
// The auto-generated version in ics07tendermint incorrectly omits it.
//
// Clean up incomplete header uploads.
// This can be called to reclaim rent from failed or abandoned uploads.
// Closes both `HeaderChunk` and `SignatureVerification` PDAs owned by the submitter.
func NewCleanupIncompleteUploadInstruction(
	submitterAccount solanago.PublicKey,
) (solanago.Instruction, error) {
	buf__ := new(bytes.Buffer)
	enc__ := binary.NewBorshEncoder(buf__)

	// Encode the instruction discriminator.
	// This is the critical part that anchor-go omits for no-argument instructions.
	err := enc__.WriteBytes(ics07_tendermint.Instruction_CleanupIncompleteUpload[:], false)
	if err != nil {
		return nil, fmt.Errorf("failed to write instruction discriminator: %w", err)
	}

	accounts__ := solanago.AccountMetaSlice{}

	// Add the accounts to the instruction.
	{
		// Account 0 "submitter": Writable, Signer, Required
		// The original submitter who gets their rent back
		// Must be the signer to prove they own the upload
		accounts__.Append(solanago.NewAccountMeta(submitterAccount, true, true))
	}

	// Create the instruction.
	return solanago.NewInstruction(
		ics07_tendermint.ProgramID,
		accounts__,
		buf__.Bytes(),
	), nil
}
