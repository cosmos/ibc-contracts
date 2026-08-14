// SPDX-License-Identifier: Apache-2.0

package ibctesting

import comettypes "github.com/cometbft/cometbft/types"

const (
	FirstClientID  = "07-tendermint-0"
	SecondClientID = "07-tendermint-1"
	FirstChannelID = "channel-0"
	InvalidID      = "IDisInvalid"
)

func MakeBlockID(hash []byte, partSetSize uint32, partSetHash []byte) comettypes.BlockID {
	return comettypes.BlockID{
		Hash: hash,
		PartSetHeader: comettypes.PartSetHeader{
			Total: partSetSize,
			Hash:  partSetHash,
		},
	}
}
