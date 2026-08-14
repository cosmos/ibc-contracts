// SPDX-License-Identifier: Apache-2.0

package types

import (
	"fmt"
	"math/rand"
	"os"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

// SupportedProofType is an enum for supported proof types.
type SupportedProofType int

const (
	// ProofTypeGroth16 represents the Groth16 SP1 proof type.
	ProofTypeGroth16 SupportedProofType = iota
	// ProofTypePlonk represents the Plonk SP1 proof type.
	ProofTypePlonk
)

// String returns the string representation of the proof type.
func (pt SupportedProofType) String() string {
	return [...]string{"groth16", "plonk"}[pt]
}

// ToOperatorArgs returns the proof type as arguments for the operator command.
func (pt SupportedProofType) ToOperatorArgs() []string {
	return []string{"-p", pt.String()}
}

// GetEnvProofType returns a proof type based on the environment variable SP1_PROOF_TYPE.
// If the variable is not set, it returns a random proof type.
func GetEnvProofType() SupportedProofType {
	envProofType := os.Getenv(testvalues.EnvKeyE2EProofType)
	switch envProofType {
	case "":
		return SupportedProofType(rand.Intn(2))
	case testvalues.EnvValueProofType_Groth16:
		return ProofTypeGroth16
	case testvalues.EnvValueProofType_Plonk:
		return ProofTypePlonk
	default:
		panic(fmt.Errorf("unsupported proof type in env: %s", envProofType))
	}
}
