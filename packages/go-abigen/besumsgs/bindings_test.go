// SPDX-License-Identifier: Apache-2.0

package besumsgs_test

import (
	"bytes"
	"encoding/json"
	"os"
	"testing"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/besumsgs"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/crypto"
)

func TestQBFTFixtureBindings(t *testing.T) {
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
	bindings := besumsgs.NewBindings()

	t.Run("proof nodes match Solidity fixture", func(t *testing.T) {
		// Decode only to recover the fixture's input nodes. The independent
		// expected encoding is the account proof consumed by the Solidity test.
		schema, err := besumsgs.BindingsMetaData.ParseABI()
		if err != nil {
			t.Fatal(err)
		}
		values, err := schema.Methods["proofNodes"].Inputs.Unpack(fixture.NonAdjacentUpdate.AccountProof)
		if err != nil {
			t.Fatal(err)
		}
		if len(values) != 1 {
			t.Fatalf("fixture decoded to %d values, want one", len(values))
		}
		nodes, ok := values[0].([][]byte)
		if !ok {
			t.Fatalf("fixture proof nodes decoded as %T, want [][]byte", values[0])
		}
		if len(nodes) == 0 {
			t.Fatal("fixture has no account proof nodes")
		}
		// Generated packers include the function selector; wire payloads do not.
		encoded := bindings.PackProofNodes(nodes)[4:]
		if !bytes.Equal(encoded, fixture.NonAdjacentUpdate.AccountProof) {
			t.Fatal("account proof encoding differs from Solidity fixture")
		}
	})

	t.Run("consensus state matches Foundry commitment", func(t *testing.T) {
		state := besumsgs.IBesuLightClientMsgsConsensusState{
			Timestamp:   fixture.InitialTrustedTimestamp,
			StorageRoot: fixture.InitialTrustedStorageRoot,
			Validators:  fixture.InitialTrustedValidators,
		}
		// Captured from BesuQBFTLightClient.getConsensusStateHash after deploying
		// with qbft.json's initial trusted state in Foundry.
		want := common.HexToHash("0xe90678de9bc0821f64b0634c05ba8dc7726ff865d40fbf283d3bdb32ac38d4b9")
		got := crypto.Keccak256Hash(bindings.PackConsensusState(state)[4:])
		if got != want {
			t.Fatalf("consensus state hash: got %s, want %s", got, want)
		}
	})
}
