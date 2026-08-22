// SPDX-License-Identifier: Apache-2.0

package ethereum

import (
	"encoding/hex"
	"io"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/ethereum/go-ethereum/crypto"
)

func TestForgeScriptFailureDoesNotExposePrivateKey(t *testing.T) {
	workingDir, err := os.Getwd()
	require.NoError(t, err)
	require.NoError(t, os.Chdir(filepath.Join(workingDir, "../../..")))
	t.Cleanup(func() { require.NoError(t, os.Chdir(workingDir)) })

	deployer, err := crypto.GenerateKey()
	require.NoError(t, err)
	secret := hex.EncodeToString(crypto.FromECDSA(deployer))
	forgePath := filepath.Join(t.TempDir(), "forge")
	t.Setenv("PATH", filepath.Dir(forgePath)+string(os.PathListSeparator)+os.Getenv("PATH"))
	chain := Ethereum{RPC: "http://localhost:8545", Faucet: deployer}

	tests := []struct {
		name    string
		script  string
		timeout time.Duration
		wantErr string
	}{
		{name: "failure", script: "#!/bin/sh\nexit 17\n", timeout: 10 * time.Second, wantErr: "exit status 17"},
		{name: "timeout", script: "#!/bin/sh\nexec /bin/sleep 10\n", timeout: 10 * time.Millisecond, wantErr: "timed out after 10ms"},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			require.NoError(t, os.WriteFile(forgePath, []byte(test.script), 0o600))
			require.NoError(t, os.Chmod(forgePath, 0o700))

			readOutput, writeOutput, err := os.Pipe()
			require.NoError(t, err)
			originalStdout := os.Stdout
			os.Stdout = writeOutput

			stdout, runErr := chain.forgeScript(test.timeout, deployer, "script/Test.s.sol")

			os.Stdout = originalStdout
			require.NoError(t, writeOutput.Close())
			consoleOutput, err := io.ReadAll(readOutput)
			require.NoError(t, err)
			require.NoError(t, readOutput.Close())

			require.Nil(t, stdout)
			require.ErrorContains(t, runErr, "script/Test.s.sol")
			require.ErrorContains(t, runErr, test.wantErr)
			require.NotContains(t, runErr.Error(), secret)
			require.NotContains(t, string(consoleOutput), secret)
		})
	}
}
