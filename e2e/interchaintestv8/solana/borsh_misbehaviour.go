// SPDX-License-Identifier: Apache-2.0

package solana

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"os/exec"
	"strings"

	"github.com/cosmos/gogoproto/proto"

	tmclient "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"
)

// MisbehaviourToBorsh converts a Tendermint misbehaviour to Borsh format using the operator binary.
func MisbehaviourToBorsh(clientID string, misbehaviour *tmclient.Misbehaviour) ([]byte, error) {
	misbehaviour.ClientId = clientID

	protoBytes, err := proto.Marshal(misbehaviour)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal misbehaviour to protobuf: %w", err)
	}

	cmd := exec.Command("operator", "solana-misbehaviour-to-borsh")
	cmd.Stdin = bytes.NewReader(protoBytes)

	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	if err := cmd.Run(); err != nil {
		return nil, fmt.Errorf("operator solana-misbehaviour-to-borsh failed: %w\nstderr: %s", err, stderr.String())
	}

	hexStr := strings.TrimSpace(stdout.String())
	return hex.DecodeString(hexStr)
}
