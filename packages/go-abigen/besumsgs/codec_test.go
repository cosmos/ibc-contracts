// SPDX-License-Identifier: Apache-2.0

package besumsgs_test

import (
	"bytes"
	"encoding/json"
	"os"
	"reflect"
	"testing"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/besumsgs"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

func TestQBFTFixtureCodec(t *testing.T) {
	data, err := os.ReadFile("../../../ibc-solidity/test/besu-bft/fixtures/qbft.json")
	if err != nil {
		t.Fatal(err)
	}
	var fixture struct {
		InitialTrustedTimestamp   uint64
		InitialTrustedStorageRoot common.Hash
		InitialTrustedValidators  []common.Address
		NonAdjacentUpdate         struct {
			AccountProof hexutil.Bytes
		}
	}
	if err := json.Unmarshal(data, &fixture); err != nil {
		t.Fatal(err)
	}

	t.Run("proof nodes", func(t *testing.T) {
		nodes, err := besumsgs.DecodeProofNodes(fixture.NonAdjacentUpdate.AccountProof)
		if err != nil {
			t.Fatal(err)
		}
		if len(nodes) == 0 {
			t.Fatal("fixture has no account proof nodes")
		}
		encoded, err := besumsgs.EncodeProofNodes(nodes)
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(encoded, fixture.NonAdjacentUpdate.AccountProof) {
			t.Fatal("account proof encoding differs from fixture")
		}
	})

	t.Run("consensus state", func(t *testing.T) {
		state := besumsgs.IBesuLightClientMsgsConsensusState{
			Timestamp:   fixture.InitialTrustedTimestamp,
			StorageRoot: fixture.InitialTrustedStorageRoot,
			Validators:  fixture.InitialTrustedValidators,
		}
		encoded, err := besumsgs.EncodeConsensusState(state)
		if err != nil {
			t.Fatal(err)
		}
		decoded, err := besumsgs.DecodeConsensusState(encoded)
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(state, decoded) {
			t.Fatalf("consensus state mismatch: got %+v, want %+v", decoded, state)
		}

		// Captured from BesuQBFTLightClient.getConsensusStateHash after deploying
		// with qbft.json's initial trusted state in Foundry.
		want := common.HexToHash("0xe90678de9bc0821f64b0634c05ba8dc7726ff865d40fbf283d3bdb32ac38d4b9")
		got, err := besumsgs.ConsensusStateHash(state)
		if err != nil {
			t.Fatal(err)
		}
		if got != want {
			t.Fatalf("consensus state hash: got %s, want %s", got, want)
		}
	})
}

func TestEmptyProofNodes(t *testing.T) {
	encoded, err := besumsgs.EncodeProofNodes(nil)
	if err != nil {
		t.Fatal(err)
	}
	nodes, err := besumsgs.DecodeProofNodes(encoded)
	if err != nil {
		t.Fatal(err)
	}
	if len(nodes) != 0 {
		t.Fatalf("got %d nodes, want none", len(nodes))
	}
}

func TestDecodeRejectsEmptyInput(t *testing.T) {
	if _, err := besumsgs.DecodeClientState(nil); err == nil {
		t.Error("DecodeClientState accepted empty input")
	}
	if _, err := besumsgs.DecodeConsensusState(nil); err == nil {
		t.Error("DecodeConsensusState accepted empty input")
	}
	if _, err := besumsgs.DecodeUpdateClient(nil); err == nil {
		t.Error("DecodeUpdateClient accepted empty input")
	}
	if _, err := besumsgs.DecodeMembershipProof(nil); err == nil {
		t.Error("DecodeMembershipProof accepted empty input")
	}
	if _, err := besumsgs.DecodeProofNodes(nil); err == nil {
		t.Error("DecodeProofNodes accepted empty input")
	}
}
