// SPDX-License-Identifier: Apache-2.0

package tests

import (
	"math/big"
	"reflect"
	"testing"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/besuerrors"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/besumsgs"
	"github.com/ethereum/go-ethereum/common"
)

func checkPayloadRoundTrip[T any](t *testing.T, value T, pack func(T) []byte, unpack func([]byte) (T, error)) {
	t.Helper()
	// Packers include a function selector; light-client payloads do not.
	got, err := unpack(pack(value)[4:])
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(got, value) {
		t.Fatalf("decoded %#v, want %#v", got, value)
	}
	if _, err := unpack(nil); err == nil {
		t.Fatal("accepted an empty payload")
	}
}

func TestBesuPayloadDecoders(t *testing.T) {
	bindings := besumsgs.NewBindings()
	height := besumsgs.IICS02ClientMsgsHeight{RevisionNumber: 1, RevisionHeight: 42}
	state := besumsgs.IBesuLightClientMsgsConsensusState{
		Timestamp:  1234,
		StateRoot:  common.HexToHash("0x1234"),
		Validators: []common.Address{common.HexToAddress("0x1234"), common.HexToAddress("0x5678")},
	}
	nodes := [][]byte{{1, 2, 3}, {4, 5}}
	t.Run("clientState", func(t *testing.T) {
		checkPayloadRoundTrip(t, besumsgs.IBesuLightClientMsgsClientState{
			IbcRouter: common.HexToAddress("0xabcd"), LatestHeight: height,
			TrustingPeriod: 3600, MaxClockDrift: 30,
		}, bindings.PackClientState, bindings.UnpackClientState)
	})
	t.Run("consensusState", func(t *testing.T) {
		checkPayloadRoundTrip(t, state, bindings.PackConsensusState, bindings.UnpackConsensusState)
	})
	t.Run("updateClient", func(t *testing.T) {
		checkPayloadRoundTrip(t, besumsgs.IBesuLightClientMsgsMsgUpdateClient{
			HeaderRlp: []byte{1, 2, 3}, TrustedHeight: height, ConsensusStatePreimage: state,
		}, bindings.PackUpdateClient, bindings.UnpackUpdateClient)
	})
	t.Run("membershipProof", func(t *testing.T) {
		checkPayloadRoundTrip(t, besumsgs.IBesuLightClientMsgsMembershipProof{
			ConsensusStatePreimage: state, AccountProofNodes: nodes, ProofNodes: [][]byte{},
		}, bindings.PackMembershipProof, bindings.UnpackMembershipProof)
	})
	t.Run("proofNodes", func(t *testing.T) {
		checkPayloadRoundTrip(t, nodes, bindings.PackProofNodes, bindings.UnpackProofNodes)
		checkPayloadRoundTrip(t, [][]byte{}, bindings.PackProofNodes, bindings.UnpackProofNodes)
	})
}

func TestBesuErrorDecoders(t *testing.T) {
	bindings := besuerrors.NewBindings()
	signer := common.HexToAddress("0x1234")
	for _, tc := range []struct {
		name string
		args []any
		want any
	}{
		{"InvalidTrustingPeriod", nil, &besuerrors.BindingsInvalidTrustingPeriod{}},
		{"ConsensusStateExpired", []any{uint64(100), big.NewInt(200), uint64(50)},
			&besuerrors.BindingsConsensusStateExpired{TrustedTimestamp: 100, CurrentTimestamp: big.NewInt(200), TrustingPeriod: 50}},
		{"UnknownCommitSealSigner", []any{signer}, &besuerrors.BindingsUnknownCommitSealSigner{Signer: signer}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			schema := bindings.GetABI().Errors[tc.name]
			payload, err := schema.Inputs.Pack(tc.args...)
			if err != nil {
				t.Fatal(err)
			}
			got, err := bindings.UnpackError(append(schema.ID.Bytes()[:4], payload...))
			if err != nil {
				t.Fatal(err)
			}
			if !reflect.DeepEqual(got, tc.want) {
				t.Fatalf("decoded %#v, want %#v", got, tc.want)
			}
		})
	}
}
