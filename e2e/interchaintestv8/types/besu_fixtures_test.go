// SPDX-License-Identifier: Apache-2.0

package types

import (
	"bytes"
	"testing"

	"github.com/ethereum/go-ethereum/accounts/abi"
)

func TestEncodeProofNodesABI(t *testing.T) {
	t.Parallel()

	encoded, err := encodeProofNodes([]string{"0xc20102", "0x83aabbcc"})
	if err != nil {
		t.Fatalf("encode proof nodes: %v", err)
	}

	bytesArrayType, err := abi.NewType("bytes[]", "", nil)
	if err != nil {
		t.Fatalf("create bytes[] ABI type: %v", err)
	}
	values, err := (abi.Arguments{{Type: bytesArrayType}}).Unpack(encoded)
	if err != nil {
		t.Fatalf("unpack proof nodes: %v", err)
	}
	nodes, ok := values[0].([][]byte)
	if !ok {
		t.Fatalf("unexpected decoded type %T", values[0])
	}
	if len(nodes) != 2 || !bytes.Equal(nodes[0], []byte{0xc2, 0x01, 0x02}) ||
		!bytes.Equal(nodes[1], []byte{0x83, 0xaa, 0xbb, 0xcc}) {
		t.Fatalf("unexpected decoded proof nodes: %x", nodes)
	}
}
