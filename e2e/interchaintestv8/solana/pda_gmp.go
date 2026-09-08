// SPDX-License-Identifier: Apache-2.0

package solana

// GMP PDA helpers - manually maintained because the GMP account PDA
// requires Borsh serialization and SHA256 hashing, which can't be auto-generated.

import (
	"crypto/sha256"
	"encoding/binary"
	"fmt"

	solanago "github.com/gagliardetto/solana-go"
)

// GmpAccountPDA derives the GMP account PDA using the new AccountIdentifier format.
// Seeds: ["gmp_account", sha256(borsh(AccountIdentifier))]
// AccountIdentifier is Borsh-serialized as: client_id (string) + sender (string) + salt (bytes)
func (ics27GmpPDAs) GmpAccountPDA(programID solanago.PublicKey, clientId []byte, sender []byte, salt []byte) (solanago.PublicKey, uint8) {
	// Borsh serialize AccountIdentifier
	// Borsh strings: u32 length prefix + bytes
	// Borsh Vec<u8>: u32 length prefix + bytes
	var data []byte

	// client_id as string (u32 len + bytes)
	lenBuf := make([]byte, 4)
	binary.LittleEndian.PutUint32(lenBuf, uint32(len(clientId)))
	data = append(data, lenBuf...)
	data = append(data, clientId...)

	// sender as string (u32 len + bytes)
	binary.LittleEndian.PutUint32(lenBuf, uint32(len(sender)))
	data = append(data, lenBuf...)
	data = append(data, sender...)

	// salt as Vec<u8> (u32 len + bytes)
	binary.LittleEndian.PutUint32(lenBuf, uint32(len(salt)))
	data = append(data, lenBuf...)
	data = append(data, salt...)

	// SHA256 hash of Borsh-serialized data
	accountIdHash := sha256.Sum256(data)

	pda, bump, err := solanago.FindProgramAddress(
		[][]byte{[]byte("gmp_account"), accountIdHash[:]},
		programID,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to derive Ics27Gmp.GmpAccountPDA PDA: %v", err))
	}
	return pda, bump
}
