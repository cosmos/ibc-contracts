// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"
	"crypto/ecdsa"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"regexp"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	solanago "github.com/gagliardetto/solana-go"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
)

// DeploySolanaProgramTask creates a ParallelTaskWithResult that deploys a
// Solana program. The returned task is ready for RunParallelTasksWithResults.
func DeploySolanaProgramTask(
	ctx context.Context,
	t *testing.T,
	chain solana.Solana,
	displayName, programName, keypairDir, deployerPath string,
) ParallelTaskWithResult[solanago.PublicKey] {
	t.Helper()
	return ParallelTaskWithResult[solanago.PublicKey]{
		Name: displayName,
		Run: func() (solanago.PublicKey, error) {
			t.Logf("Deploying %s...", displayName)
			keypairPath := fmt.Sprintf("%s/%s-keypair.json", keypairDir, programName)
			programID, err := chain.DeploySolanaProgramAsync(ctx, programName, keypairPath, deployerPath)
			if err == nil {
				t.Logf("Deployed %s at: %s", displayName, programID)
			}
			return programID, err
		},
	}
}

// SolanaIFTConstructorPDAs holds the pre-derived PDAs needed to deploy a
// SolanaIFTSendCallConstructor via forge script.
type SolanaIFTConstructorPDAs struct {
	AppState      solanago.PublicKey
	AppMintState  solanago.PublicKey
	IFTBridge     solanago.PublicKey
	Mint          solanago.PublicKey
	MintAuthority solanago.PublicKey
	GMPAccount    solanago.PublicKey
}

func pubkeyToBytes32Hex(pk solanago.PublicKey) string {
	return "0x" + hex.EncodeToString(pk[:])
}

// DeploySolanaIFTConstructor deploys a SolanaIFTSendCallConstructor contract
// via forge script and returns its address. The caller derives the PDAs and
// passes them in so this helper stays independent of go-anchor packages.
func DeploySolanaIFTConstructor(
	t *testing.T,
	eth *ethereum.Ethereum,
	deployer *ecdsa.PrivateKey,
	clientID string,
	pdas SolanaIFTConstructorPDAs,
) string {
	t.Helper()

	t.Logf("Constructor PDAs for %s (mint=%s): appState=%s, appMintState=%s, iftBridge=%s, mintAuthority=%s, gmpAccount=%s",
		clientID, pdas.Mint, pdas.AppState, pdas.AppMintState, pdas.IFTBridge, pdas.MintAuthority, pdas.GMPAccount)

	stdout, err := eth.ForgeScript(deployer,
		"scripts/DeploySolanaIFTConstructor.s.sol:DeploySolanaIFTConstructor",
		"--sig", "run(bytes32,bytes32,bytes32,bytes32,bytes32,bytes32)",
		pubkeyToBytes32Hex(pdas.AppState),
		pubkeyToBytes32Hex(pdas.AppMintState),
		pubkeyToBytes32Hex(pdas.IFTBridge),
		pubkeyToBytes32Hex(pdas.Mint),
		pubkeyToBytes32Hex(pdas.MintAuthority),
		pubkeyToBytes32Hex(pdas.GMPAccount),
	)
	require.NoError(t, err)

	output := string(stdout)
	cutOff := "== Return =="
	cutoffIndex := strings.Index(output, cutOff)
	require.Greater(t, cutoffIndex, 0, "forge script output missing '== Return ==' section")
	output = output[cutoffIndex+len(cutOff):]

	re := regexp.MustCompile(`\{.*\}`)
	jsonPart := re.FindString(output)
	jsonPart = strings.ReplaceAll(jsonPart, `\"`, `"`)
	jsonPart = strings.Trim(jsonPart, `"`)

	var result struct {
		SolanaIftConstructor string `json:"solanaIftConstructor"`
	}
	err = json.Unmarshal([]byte(jsonPart), &result)
	require.NoError(t, err)
	require.NotEmpty(t, result.SolanaIftConstructor, "SolanaIftConstructor address is empty")

	t.Logf("SolanaIFTSendCallConstructor for %s deployed at: %s", clientID, result.SolanaIftConstructor)
	return result.SolanaIftConstructor
}
