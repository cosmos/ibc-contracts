// SPDX-License-Identifier: Apache-2.0

package attestations

import "errors"

var (
	ErrEmptyAttestorAddresses          = errors.New("attestor addresses cannot be empty")
	ErrInvalidMinRequiredSigs          = errors.New("min required sigs cannot be 0")
	ErrMinRequiredSigsExceedsAttestors = errors.New("min required sigs cannot exceed number of attestors")
	ErrInvalidLatestHeight             = errors.New("latest height must be greater than 0")
	ErrInvalidTimestamp                = errors.New("timestamp cannot be 0")
	ErrEmptyAttestationData            = errors.New("attestation data cannot be empty")
	ErrEmptySignatures                 = errors.New("signatures cannot be empty")
)
