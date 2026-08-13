// SPDX-License-Identifier: Apache-2.0

// Package attestations provides local attestations light client types for codec registration.
// These types mirror the ibc-go attestations light client types but are defined locally
// to avoid upgrading the entire ibc-go dependency tree.
package attestations

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	"github.com/cosmos/ibc-go/v11/modules/core/exported"
)

const ClientType = "attestations"

var (
	_ exported.ClientState    = (*ClientState)(nil)
	_ exported.ConsensusState = (*ConsensusState)(nil)
	_ exported.ClientMessage  = (*AttestationProof)(nil)
)

// RegisterInterfaces registers the attestations light client types to the interface registry.
func RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	registry.RegisterImplementations(
		(*exported.ClientState)(nil),
		&ClientState{},
	)
	registry.RegisterImplementations(
		(*exported.ConsensusState)(nil),
		&ConsensusState{},
	)
	registry.RegisterImplementations(
		(*exported.ClientMessage)(nil),
		&AttestationProof{},
	)
}

// ClientState interface implementation

func (ClientState) ClientType() string {
	return ClientType
}

func (cs ClientState) Validate() error {
	if len(cs.AttestorAddresses) == 0 {
		return ErrEmptyAttestorAddresses
	}
	if cs.MinRequiredSigs == 0 {
		return ErrInvalidMinRequiredSigs
	}
	if cs.MinRequiredSigs > uint32(len(cs.AttestorAddresses)) {
		return ErrMinRequiredSigsExceedsAttestors
	}
	if cs.LatestHeight == 0 {
		return ErrInvalidLatestHeight
	}
	return nil
}

// ConsensusState interface implementation

func (ConsensusState) ClientType() string {
	return ClientType
}

func (ConsensusState) GetTimestamp() uint64 {
	panic("GetTimestamp is deprecated")
}

func (cs ConsensusState) ValidateBasic() error {
	if cs.Timestamp == 0 {
		return ErrInvalidTimestamp
	}
	return nil
}

// AttestationProof interface implementation

func (AttestationProof) ClientType() string {
	return ClientType
}

func (ap AttestationProof) ValidateBasic() error {
	if len(ap.AttestationData) == 0 {
		return ErrEmptyAttestationData
	}
	if len(ap.Signatures) == 0 {
		return ErrEmptySignatures
	}
	return nil
}
