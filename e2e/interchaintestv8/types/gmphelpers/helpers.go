// SPDX-License-Identifier: Apache-2.0

package gmphelpers

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	"github.com/ethereum/go-ethereum/accounts/abi"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	gmptypes "github.com/cosmos/ibc-go/v11/modules/apps/27-gmp/types"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	solanatypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/solana"
)

// packedAccountSize is the size of a single packed account entry: pubkey(32) + is_signer(1) + is_writable(1).
const packedAccountSize = 34

// MarshalGMPSolanaPayload encodes a GMPSolanaPayload using the specified encoding.
//
// For protobuf: standard proto.Marshal.
// For ABI: packs accounts as 34-byte entries and produces abi.encode(GMPSolanaPayload)
// matching the Solidity SolanaIFTSendCallConstructor format.
func MarshalGMPSolanaPayload(payload *solanatypes.GMPSolanaPayload, encoding string) ([]byte, error) {
	if encoding == testvalues.Ics27AbiEncoding {
		return marshalGMPSolanaPayloadABI(payload)
	}
	return proto.Marshal(payload)
}

func marshalGMPSolanaPayloadABI(payload *solanatypes.GMPSolanaPayload) ([]byte, error) {
	packed := make([]byte, len(payload.Accounts)*packedAccountSize)
	for i, acct := range payload.Accounts {
		off := i * packedAccountSize
		copy(packed[off:off+32], acct.Pubkey)
		if acct.IsSigner {
			packed[off+32] = 1
		}
		if acct.IsWritable {
			packed[off+33] = 1
		}
	}

	tupleType, err := abi.NewType("tuple", "", []abi.ArgumentMarshaling{
		{Name: "packedAccounts", Type: "bytes"},
		{Name: "instructionData", Type: "bytes"},
		{Name: "prefundLamports", Type: "uint64"},
	})
	if err != nil {
		return nil, fmt.Errorf("creating GMPSolanaPayload ABI type: %w", err)
	}

	args := abi.Arguments{{Type: tupleType}}

	abiPayload := struct {
		PackedAccounts  []byte `json:"packedAccounts"`
		InstructionData []byte `json:"instructionData"`
		PrefundLamports uint64 `json:"prefundLamports"`
	}{
		PackedAccounts:  packed,
		InstructionData: payload.Data,
		PrefundLamports: payload.PrefundLamports,
	}

	encoded, err := args.Pack(abiPayload)
	if err != nil {
		return nil, fmt.Errorf("ABI encoding GMPSolanaPayload: %w", err)
	}

	return encoded, nil
}

// NewPayload_FromProto creates a new payload to be submitted to cosmos through gmp.
func NewPayload_FromProto(msgs []proto.Message) ([]byte, error) {
	cosmosMsgs := make([]*codectypes.Any, len(msgs))
	for i, msg := range msgs {
		protoAny, err := codectypes.NewAnyWithValue(msg)
		if err != nil {
			return nil, err
		}

		cosmosMsgs[i] = protoAny
	}

	cosmosTx := gmptypes.CosmosTx{
		Messages: cosmosMsgs,
	}
	cosmosTxBz, err := proto.Marshal(&cosmosTx)
	if err != nil {
		return nil, err
	}

	return cosmosTxBz, nil
}

// UnmarshalAck decodes a GMP acknowledgement using the appropriate encoding.
func UnmarshalAck(data []byte, encoding string) (gmptypes.Acknowledgement, error) {
	if encoding == testvalues.Ics27AbiEncoding {
		return DecodeABIAck(data)
	}

	var ack gmptypes.Acknowledgement
	if err := proto.Unmarshal(data, &ack); err != nil {
		return gmptypes.Acknowledgement{}, fmt.Errorf("unmarshalling protobuf ack: %w", err)
	}

	return ack, nil
}

// DecodeABIAck decodes an ABI-encoded GMP acknowledgement.
func DecodeABIAck(data []byte) (gmptypes.Acknowledgement, error) {
	tupleType, err := abi.NewType("tuple", "", []abi.ArgumentMarshaling{
		{Name: "result", Type: "bytes"},
	})
	if err != nil {
		return gmptypes.Acknowledgement{}, fmt.Errorf("creating abi type: %w", err)
	}

	args := abi.Arguments{{Type: tupleType}}
	unpacked, err := args.Unpack(data)
	if err != nil {
		return gmptypes.Acknowledgement{}, fmt.Errorf("unpacking abi ack: %w", err)
	}

	parsed, ok := unpacked[0].(struct {
		Result []byte `json:"result"`
	})
	if !ok {
		return gmptypes.Acknowledgement{}, fmt.Errorf("unexpected abi ack type")
	}

	return gmptypes.Acknowledgement{Result: parsed.Result}, nil
}
