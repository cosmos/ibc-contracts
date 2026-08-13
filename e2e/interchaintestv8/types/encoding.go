// SPDX-License-Identifier: Apache-2.0

package types

import (
	"fmt"
	"math/rand"
	"os"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

// SupportedEncodingType is an enum for supported GMP encoding types.
type SupportedEncodingType int

const (
	EncodingTypeProtobuf SupportedEncodingType = iota
	EncodingTypeAbi
)

// String returns the wire-format encoding string.
func (et SupportedEncodingType) String() string {
	return [...]string{testvalues.Ics27ProtobufEncoding, testvalues.Ics27AbiEncoding}[et]
}

// GetEnvEncodingType returns an encoding type based on the environment variable E2E_GMP_ENCODING.
// If the variable is not set, it returns a random encoding type.
func GetEnvEncodingType() SupportedEncodingType {
	envEncoding := os.Getenv(testvalues.EnvKeyE2EGmpEncoding)
	switch envEncoding {
	case "":
		return SupportedEncodingType(rand.Intn(2))
	case testvalues.Ics27ProtobufEncoding:
		return EncodingTypeProtobuf
	case testvalues.Ics27AbiEncoding:
		return EncodingTypeAbi
	default:
		panic(fmt.Errorf("unsupported GMP encoding type in env: %s", envEncoding))
	}
}
