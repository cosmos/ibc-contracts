// SPDX-License-Identifier: Apache-2.0

package types

import (
	"encoding/hex"
	"testing"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"
)

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
