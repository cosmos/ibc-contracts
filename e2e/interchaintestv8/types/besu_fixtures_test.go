// SPDX-License-Identifier: Apache-2.0

package types

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"testing"

	ethcommon "github.com/ethereum/go-ethereum/common"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"
)

func TestBuildLowOverlapFixtureNeedsNoAccountProof(t *testing.T) {
	t.Chdir("../../..")

	fixtureJSON, err := os.ReadFile("ibc-solidity/test/besu-bft/fixtures/qbft.json")
	if err != nil {
		t.Fatal(err)
	}
	var fixture besuFixture
	if err := json.Unmarshal(fixtureJSON, &fixture); err != nil {
		t.Fatal(err)
	}
	validatorKeys, err := loadQBFTValidatorKeys()
	if err != nil {
		t.Fatal(err)
	}

	update, err := buildLowOverlapFixture(
		fixture.InitialTrustedHeight,
		fixture.UpdateHeight12.Height+1,
		liveHeader{HeaderRLP: ethcommon.FromHex(fixture.UpdateHeight12.HeaderRlp)},
		validatorKeys,
	)
	if err != nil {
		t.Fatal(err)
	}
	if update.AccountProof != "0x" {
		t.Fatalf("account proof: got %q, want empty hex bytes", update.AccountProof)
	}

	encoded, err := json.Marshal(update)
	if err != nil {
		t.Fatal(err)
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(encoded, &fields); err != nil {
		t.Fatal(err)
	}
	for _, field := range []string{"expectedTimestamp", "expectedStorageRoot", "expectedValidators"} {
		if _, ok := fields[field]; ok {
			t.Fatalf("unexpected success field %q", field)
		}
	}
}

func TestRejectionFixturesOnlyContainUpdateInputs(t *testing.T) {
	t.Chdir("../../..")

	fixtureJSON, err := os.ReadFile("ibc-solidity/test/besu-bft/fixtures/qbft.json")
	if err != nil {
		t.Fatal(err)
	}
	var fixture besuFixture
	if err := json.Unmarshal(fixtureJSON, &fixture); err != nil {
		t.Fatal(err)
	}

	lowQuorum, err := buildLowQuorumFixture(
		fixture.UpdateHeight12,
		liveHeader{HeaderRLP: ethcommon.FromHex(fixture.UpdateHeight12.HeaderRlp)},
	)
	if err != nil {
		t.Fatal(err)
	}
	if lowQuorum.AccountProof != "0x" {
		t.Fatalf("low quorum account proof: got %q, want empty hex bytes", lowQuorum.AccountProof)
	}
	if fixture.ConflictingHeight12.AccountProof == "" || fixture.ConflictingHeight12.AccountProof == "0x" {
		t.Fatal("conflicting update must retain its account proof")
	}

	for name, update := range map[string]besuRejectionUpdateFixture{
		"low quorum":  lowQuorum,
		"conflicting": fixture.ConflictingHeight12,
	} {
		encoded, err := json.Marshal(update)
		if err != nil {
			t.Fatal(err)
		}
		var fields map[string]json.RawMessage
		if err := json.Unmarshal(encoded, &fields); err != nil {
			t.Fatal(err)
		}
		for _, field := range []string{"expectedTimestamp", "expectedStorageRoot", "expectedValidators"} {
			if _, ok := fields[field]; ok {
				t.Errorf("%s: unexpected success field %q", name, field)
			}
		}
	}
}

func TestPacketCommitment(t *testing.T) {
	packetData := transfertypes.NewFungibleTokenPacketData("uatom", "1000000", "sender", "receiver", "memo")
	value, err := transfertypes.EncodeABIFungibleTokenPacketData(&packetData)
	if err != nil {
		t.Fatal(err)
	}

	packet := ics26router.IICS26RouterMsgsPacket{
		Sequence:         1,
		SourceClient:     "channel-0",
		DestClient:       "channel-1",
		TimeoutTimestamp: 100,
		Payloads: []ics26router.IICS26RouterMsgsPayload{{
			SourcePort: transfertypes.PortID,
			DestPort:   transfertypes.PortID,
			Version:    transfertypes.V1,
			Encoding:   transfertypes.EncodingABI,
			Value:      value,
		}},
	}

	// Golden vector shared with ibc-solidity/test/solidity-ibc/ICS24HostTest.t.sol.
	const expected = "b691a1950f6fb0bbbcf4bdb16fe2c4d0aa7ef783eb7803073f475cb8164d9b7a"
	if actual := hex.EncodeToString(packetCommitment(packet)); actual != expected {
		t.Fatalf("packet commitment: got %s, want %s", actual, expected)
	}
}
