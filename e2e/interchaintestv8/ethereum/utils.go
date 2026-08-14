// SPDX-License-Identifier: Apache-2.0

package ethereum

import (
	"encoding/json"
	"fmt"
	"math/big"
	"regexp"
	"strings"

	ethcommon "github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

type ForgeScriptReturnValues struct {
	InternalType string `json:"internal_type"`
	Value        string `json:"value"`
}

type ForgeDeployOutput struct {
	Returns map[string]ForgeScriptReturnValues `json:"returns"`
}

type DeployedContracts struct {
	// SP1Verifier for plonk
	VerifierPlonk string `json:"verifierPlonk"`
	// SP1Verifier for groth16
	VerifierGroth16 string `json:"verifierGroth16"`
	// Mock SP1 verifier
	VerifierMock  string `json:"verifierMock"`
	Ics26Router   string `json:"ics26Router"`
	Ics20Transfer string `json:"ics20Transfer"`
	Ics27Gmp      string `json:"ics27Gmp"`
	Erc20         string `json:"erc20"`
	// IFT contract (optional)
	Ift string `json:"ift,omitempty"`
	// CosmosIFTSendCallConstructor (optional, deployed when IFT_ICA_ADDRESS is set)
	CosmosIftConstructor string `json:"cosmosIftConstructor,omitempty"`
	SolanaIftConstructor string `json:"solanaIftConstructor,omitempty"`
}

// GetVerifierAddress returns the appropriate verifier address based on prover type and proof type.
// proofType should be "groth16" or "plonk".
func (c DeployedContracts) GetVerifierAddress(prover string, proofType string) (string, error) {
	if prover == testvalues.EnvValueSp1Prover_Mock {
		return c.VerifierMock, nil
	}

	switch proofType {
	case testvalues.EnvValueProofType_Groth16:
		return c.VerifierGroth16, nil
	case testvalues.EnvValueProofType_Plonk:
		return c.VerifierPlonk, nil
	default:
		return "", fmt.Errorf("invalid proof type: %s", proofType)
	}
}

func GetEthContractsFromDeployOutput(stdout string) (DeployedContracts, error) {
	// Remove everything above the JSON part
	cutOff := "== Return =="
	cutoffIndex := strings.Index(stdout, cutOff)
	stdout = stdout[cutoffIndex+len(cutOff):]

	// Extract the JSON part using regex
	re := regexp.MustCompile(`\{.*\}`)
	jsonPart := re.FindString(stdout)

	jsonPart = strings.ReplaceAll(jsonPart, `\"`, `"`)
	jsonPart = strings.Trim(jsonPart, `"`)

	var embeddedContracts DeployedContracts
	err := json.Unmarshal([]byte(jsonPart), &embeddedContracts)
	if err != nil {
		return DeployedContracts{}, err
	}

	if embeddedContracts.Erc20 == "" ||
		embeddedContracts.Ics20Transfer == "" ||
		embeddedContracts.Ics27Gmp == "" ||
		embeddedContracts.VerifierPlonk == "" ||
		embeddedContracts.VerifierGroth16 == "" ||
		embeddedContracts.VerifierMock == "" ||
		embeddedContracts.Ics26Router == "" {

		return DeployedContracts{}, fmt.Errorf("one or more contracts missing: %+v", embeddedContracts)
	}

	return embeddedContracts, nil
}

// From https://medium.com/@zhuytt4/verify-the-owner-of-safe-wallet-with-eth-getproof-7edc450504ff
func GetCommitmentsStorageKey(path []byte) ethcommon.Hash {
	commitmentStorageSlot := ethcommon.FromHex(testvalues.IbcCommitmentSlotHex)

	pathHash := crypto.Keccak256(path)

	// zero pad both to 32 bytes
	paddedSlot := ethcommon.LeftPadBytes(commitmentStorageSlot, 32)

	// keccak256(h(k) . slot)
	return crypto.Keccak256Hash(pathHash, paddedSlot)
}

func HexToBeBytes(hex string) []byte {
	bz := ethcommon.FromHex(hex)
	if len(bz) == 32 {
		return bz
	}
	if len(bz) > 32 {
		panic("TOO BIG!")
	}
	beBytes := make([]byte, 32)
	copy(beBytes[32-len(bz):32], bz)
	return beBytes
}

func BigIntToBeBytes(n *big.Int) [32]byte {
	bytes := n.Bytes()
	var beBytes [32]byte
	copy(beBytes[32-len(bytes):], bytes)
	return beBytes
}
