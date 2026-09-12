// SPDX-License-Identifier: Apache-2.0

package besumsgs

import (
	"fmt"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
)

var (
	consensusStateArgs  = mustEncodingInputs("consensusState")
	clientStateArgs     = mustEncodingInputs("clientState")
	updateClientArgs    = mustEncodingInputs("updateClient")
	membershipProofArgs = mustEncodingInputs("membershipProof")
	proofNodesArgs      = mustEncodingInputs("proofNodes")
)

// Use only the generated inputs: these payloads are abi.encode(value), not
// function calls with a selector.
func mustEncodingInputs(name string) abi.Arguments {
	schema, err := EncodingMetaData.GetAbi()
	if err != nil {
		panic(err)
	}

	method, ok := schema.Methods[name]
	if !ok || len(method.Inputs) != 1 {
		panic("invalid Besu encoding schema: " + name)
	}

	return method.Inputs
}

// unpackSingle decodes data as args' single tuple argument into dst, which
// must be a pointer to a struct whose first field is the tuple's Go mirror.
func unpackSingle(args abi.Arguments, data []byte, dst any) error {
	values, err := args.Unpack(data)
	if err != nil {
		return err
	}

	return args.Copy(dst, values)
}

// EncodeClientState returns abi.encode(value), without a function selector.
func EncodeClientState(value IBesuLightClientMsgsClientState) ([]byte, error) {
	return clientStateArgs.Pack(value)
}

// DecodeClientState decodes a wire value without applying light-client policy checks.
func DecodeClientState(data []byte) (IBesuLightClientMsgsClientState, error) {
	var decoded struct {
		Value IBesuLightClientMsgsClientState
	}
	err := unpackSingle(clientStateArgs, data, &decoded)
	return decoded.Value, err
}

// EncodeConsensusState returns abi.encode(value), without a function selector.
func EncodeConsensusState(value IBesuLightClientMsgsConsensusState) ([]byte, error) {
	return consensusStateArgs.Pack(value)
}

// DecodeConsensusState decodes a wire value without applying light-client policy checks.
func DecodeConsensusState(data []byte) (IBesuLightClientMsgsConsensusState, error) {
	var decoded struct {
		Value IBesuLightClientMsgsConsensusState
	}
	err := unpackSingle(consensusStateArgs, data, &decoded)
	return decoded.Value, err
}

// EncodeUpdateClient returns abi.encode(value), without a function selector.
func EncodeUpdateClient(value IBesuLightClientMsgsMsgUpdateClient) ([]byte, error) {
	return updateClientArgs.Pack(value)
}

// DecodeUpdateClient decodes a wire value without applying light-client policy checks.
func DecodeUpdateClient(data []byte) (IBesuLightClientMsgsMsgUpdateClient, error) {
	var decoded struct {
		Value IBesuLightClientMsgsMsgUpdateClient
	}
	err := unpackSingle(updateClientArgs, data, &decoded)
	return decoded.Value, err
}

// EncodeMembershipProof returns abi.encode(value), without a function selector.
func EncodeMembershipProof(value IBesuLightClientMsgsMembershipProof) ([]byte, error) {
	return membershipProofArgs.Pack(value)
}

// DecodeMembershipProof decodes a wire value without applying light-client policy checks.
func DecodeMembershipProof(data []byte) (IBesuLightClientMsgsMembershipProof, error) {
	var decoded struct {
		Value IBesuLightClientMsgsMembershipProof
	}
	err := unpackSingle(membershipProofArgs, data, &decoded)
	return decoded.Value, err
}

// EncodeProofNodes returns abi.encode(bytes[]) for ordered RLP trie nodes.
func EncodeProofNodes(nodes [][]byte) ([]byte, error) {
	return proofNodesArgs.Pack(nodes)
}

// DecodeProofNodes decodes ordered RLP trie nodes.
func DecodeProofNodes(data []byte) ([][]byte, error) {
	values, err := proofNodesArgs.Unpack(data)
	if err != nil {
		return nil, err
	}
	if len(values) != 1 {
		return nil, fmt.Errorf("expected one proof nodes value, got %d", len(values))
	}
	nodes, ok := values[0].([][]byte)
	if !ok {
		return nil, fmt.Errorf("proof nodes decoded as %T", values[0])
	}
	return nodes, nil
}

// ConsensusStateHash returns the keccak256(abi.encode(state)) commitment stored by the light client.
func ConsensusStateHash(state IBesuLightClientMsgsConsensusState) (common.Hash, error) {
	data, err := EncodeConsensusState(state)
	if err != nil {
		return common.Hash{}, fmt.Errorf("encode consensus state: %w", err)
	}
	return crypto.Keccak256Hash(data), nil
}
