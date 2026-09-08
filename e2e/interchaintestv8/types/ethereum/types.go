// SPDX-License-Identifier: Apache-2.0

package ethereum

import "strconv"

const MigrationCodeOnly = "code_only"

// MigrateMsg is the CosmWasm Ethereum light client migration message
type MigrateMsg struct {
	// Migration is any because it's an enum. See the msg.rs file for concrete types.
	Migration any `json:"migration"`
}

func (h BeaconBlockHeader) GetSlot() uint64 {
	slot, err := strconv.ParseUint(h.Slot, 0, 0)
	if err != nil {
		panic(err)
	}

	return slot
}

func (u LightClientUpdate) GetSignatureSlot() uint64 {
	slot, err := strconv.ParseUint(u.SignatureSlot, 0, 0)
	if err != nil {
		panic(err)
	}

	return slot
}
