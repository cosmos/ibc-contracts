// SPDX-License-Identifier: Apache-2.0

package solana

import (
	"crypto/sha256"
	"encoding/binary"
	"fmt"

	"github.com/gagliardetto/solana-go"

	access_manager "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/accessmanager"
	attestation_light_client "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/attestation"
	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
	ics26_router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"
	ics27_gmp "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics27gmp"
)

// GMPCallResultPDA derives the PDA for a GMP call result account.
// This PDA stores the acknowledgement or timeout result of a GMP call.
// Seeds: ["gmp_result", source_client, sequence (little-endian u64)]
func GMPCallResultPDA(programID solana.PublicKey, sourceClient string, sequence uint64) (solana.PublicKey, uint8) {
	seqBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(seqBytes, sequence)
	pda, bump, err := solana.FindProgramAddress(
		[][]byte{[]byte("gmp_result"), []byte(sourceClient), seqBytes},
		programID,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to derive GMPCallResultPDA: %v", err))
	}
	return pda, bump
}

// IBCAckCommitment computes the IBC acknowledgement commitment.
// Format: sha256(0x02 || sha256(ack)) where 0x02 is the IBC version byte.
func IBCAckCommitment(ack []byte) [32]byte {
	ackHash := sha256.Sum256(ack)
	combined := append([]byte{0x02}, ackHash[:]...)
	return sha256.Sum256(combined)
}

// CallResultStatus represents the status of a GMP call result.
type CallResultStatus uint8

const (
	// CallResultStatusAcknowledgement indicates the call received an acknowledgement.
	CallResultStatusAcknowledgement CallResultStatus = 0
	// CallResultStatusTimeout indicates the call timed out.
	CallResultStatusTimeout CallResultStatus = 1
)

// GMPCallResultAccount represents the on-chain data for a GMP call result.
type GMPCallResultAccount struct {
	Version         uint8            // Account schema version (0 = V1)
	Sender          solana.PublicKey // Original sender pubkey
	Sequence        uint64           // IBC packet sequence number
	SourceClient    string           // Source client ID
	DestClient      string           // Destination client ID
	Status          CallResultStatus // Acknowledgement (with IBC commitment) or Timeout
	AckCommitment   [32]byte         // IBC acknowledgement commitment (only valid when Status == Acknowledgement)
	ResultTimestamp int64            // Timestamp when result was recorded (Unix seconds)
	Bump            uint8            // PDA bump seed
}

// DecodeGMPCallResultAccount deserializes a GMPCallResultAccount from Borsh-encoded data.
// The data should include the 8-byte Anchor discriminator prefix.
func DecodeGMPCallResultAccount(data []byte) (*GMPCallResultAccount, error) {
	if len(data) < 8 {
		return nil, fmt.Errorf("data too short: need at least 8 bytes for discriminator, got %d", len(data))
	}

	// Skip 8-byte discriminator
	offset := 8

	// Helper to read u32 length-prefixed string
	readString := func() (string, error) {
		if offset+4 > len(data) {
			return "", fmt.Errorf("not enough data for string length at offset %d", offset)
		}
		strLen := binary.LittleEndian.Uint32(data[offset:])
		offset += 4
		if offset+int(strLen) > len(data) {
			return "", fmt.Errorf("not enough data for string of length %d at offset %d", strLen, offset)
		}
		s := string(data[offset : offset+int(strLen)])
		offset += int(strLen)
		return s, nil
	}

	result := &GMPCallResultAccount{}

	// Version (u8)
	if offset >= len(data) {
		return nil, fmt.Errorf("not enough data for version")
	}
	result.Version = data[offset]
	offset++

	// Sender (Pubkey - 32 bytes)
	if offset+32 > len(data) {
		return nil, fmt.Errorf("not enough data for sender pubkey")
	}
	copy(result.Sender[:], data[offset:offset+32])
	offset += 32

	// Sequence (u64)
	if offset+8 > len(data) {
		return nil, fmt.Errorf("not enough data for sequence")
	}
	result.Sequence = binary.LittleEndian.Uint64(data[offset:])
	offset += 8

	// SourceClient (String)
	var err error
	result.SourceClient, err = readString()
	if err != nil {
		return nil, fmt.Errorf("reading source_client: %w", err)
	}

	// DestClient (String)
	result.DestClient, err = readString()
	if err != nil {
		return nil, fmt.Errorf("reading dest_client: %w", err)
	}

	// Status (Borsh enum: discriminant + optional data)
	// Acknowledgement = 0 + [u8; 32] IBC commitment
	// Timeout = 1 (no data)
	if offset >= len(data) {
		return nil, fmt.Errorf("not enough data for status")
	}
	result.Status = CallResultStatus(data[offset])
	offset++

	// For Acknowledgement, read the 32-byte IBC commitment from the enum data
	if result.Status == CallResultStatusAcknowledgement {
		if offset+32 > len(data) {
			return nil, fmt.Errorf("not enough data for ack_commitment at offset %d", offset)
		}
		copy(result.AckCommitment[:], data[offset:offset+32])
		offset += 32
	}
	// For Timeout, AckCommitment remains zero-initialized

	// ResultTimestamp (i64)
	if offset+8 > len(data) {
		return nil, fmt.Errorf("not enough data for result_timestamp")
	}
	result.ResultTimestamp = int64(binary.LittleEndian.Uint64(data[offset:]))
	offset += 8

	// Bump (u8)
	if offset >= len(data) {
		return nil, fmt.Errorf("not enough data for bump")
	}
	result.Bump = data[offset]

	return result, nil
}

func (s *Solana) CreateIBCAddressLookupTableAccounts(cosmosChainID string, gmpPortID string, clientID string, userPubKey solana.PublicKey) []solana.PublicKey {
	accessManagerPDA, _ := AccessManager.AccessManagerPDA(access_manager.ProgramID)
	routerStatePDA, _ := Ics26Router.RouterStatePDA(ics26_router.ProgramID)
	ibcAppPDA, _ := Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(gmpPortID))
	gmpAppStatePDA, _ := Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)
	clientPDA, _ := Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(clientID))
	clientStatePDA, _ := Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
	ics07AppStatePDA, _ := Ics07Tendermint.AppStatePDA(ics07_tendermint.ProgramID)

	return []solana.PublicKey{
		solana.SystemProgramID,
		ComputeBudgetProgramID(),
		access_manager.ProgramID,
		solana.SysVarInstructionsPubkey,
		ics26_router.ProgramID,
		ics07_tendermint.ProgramID,
		ics27_gmp.ProgramID,
		accessManagerPDA,
		routerStatePDA,
		userPubKey,
		ibcAppPDA,
		gmpAppStatePDA,
		clientPDA,
		clientStatePDA,
		ics07AppStatePDA,
	}
}

// CreateAttestationLightClientALTAccounts returns accounts to add to ALT for attestation light client transactions.
// These should be added in addition to the base IBC accounts.
func (s *Solana) CreateAttestationLightClientALTAccounts(clientID string) []solana.PublicKey {
	appStatePDA, _ := Attestation.AppStatePDA(attestation_light_client.ProgramID)
	clientStatePDA, _ := Attestation.ClientPDA(attestation_light_client.ProgramID)

	return []solana.PublicKey{
		attestation_light_client.ProgramID,
		appStatePDA,
		clientStatePDA,
	}
}
