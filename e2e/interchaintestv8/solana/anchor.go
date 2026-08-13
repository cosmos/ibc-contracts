// SPDX-License-Identifier: Apache-2.0

package solana

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"

	"github.com/gagliardetto/solana-go"
)

// deployerAddress must match `deployer_wallet.json`
const deployerAddress = "8ntLtUdGwBaXfFPCrNis9MWsKMdEUYyonwuw7NQwhs5z"

// DeployerPubkey is the public key of the deployer wallet used for deploying all Solana programs.
var DeployerPubkey = solana.MustPublicKeyFromBase58(deployerAddress)

// LoadDeployerWallet loads the deployer wallet from the standard keypair file.
// The deployer is the upgrade authority for all programs and must sign initialize transactions.
func LoadDeployerWallet(deployerKeypairPath string) (*solana.Wallet, error) {
	privKey, err := solana.PrivateKeyFromSolanaKeygenFile(deployerKeypairPath)
	if err != nil {
		return nil, fmt.Errorf("failed to load deployer wallet from %s: %w", deployerKeypairPath, err)
	}
	return &solana.Wallet{PrivateKey: privKey}, nil
}

func DeploySolanaProgram(ctx context.Context, programSoFile, programKeypairFile, payerKeypairFile, rpcURL string) (solana.PublicKey, solana.Signature, error) {
	absProgramFile, err := filepath.Abs(programSoFile)
	if err != nil {
		return solana.PublicKey{}, solana.Signature{}, fmt.Errorf("failed to get absolute path for program file: %w", err)
	}

	absKeypairFile, err := filepath.Abs(programKeypairFile)
	if err != nil {
		return solana.PublicKey{}, solana.Signature{}, fmt.Errorf("failed to get absolute path for program keypair file: %w", err)
	}

	absPayerFile, err := filepath.Abs(payerKeypairFile)
	if err != nil {
		return solana.PublicKey{}, solana.Signature{}, fmt.Errorf("failed to get absolute path for payer keypair file: %w", err)
	}

	cmd := exec.Command(
		"solana", "program", "deploy",
		"--url", rpcURL, // Specify RPC URL to avoid using default Solana config
		"--program-id", absKeypairFile,
		"--keypair", absPayerFile,
		"--upgrade-authority", absPayerFile,
		"--use-rpc", // Use RPC transport instead of QUIC to avoid connection issues during parallel deployments
		absProgramFile,
	)

	cmd.Env = os.Environ()

	var stdoutBuf bytes.Buffer
	var stderrBuf bytes.Buffer
	multiWriter := io.MultiWriter(os.Stdout, &stdoutBuf)
	stderrMultiWriter := io.MultiWriter(os.Stderr, &stderrBuf)

	cmd.Stdout = multiWriter
	cmd.Stderr = stderrMultiWriter

	if err := cmd.Run(); err != nil {
		fmt.Printf("Error deploy command: %v\nArgs: %v\nStderr: %s\nStdout: %s\n",
			err, cmd.Args, stderrBuf.String(), stdoutBuf.String())
		return solana.PublicKey{}, solana.Signature{}, fmt.Errorf("%w: stderr=%s", err, stderrBuf.String())
	}

	stdoutBytes := stdoutBuf.Bytes()

	return getProgramIDAndSignatureFromSolanaDeploy(stdoutBytes)
}

// Parses raw Solana CLI deploy output and extracts Program ID and Signature.
func getProgramIDAndSignatureFromSolanaDeploy(stdout []byte) (solana.PublicKey, solana.Signature, error) {
	outputStr := string(stdout)

	programIDRe := regexp.MustCompile(`(?m)Program Id:\s+([1-9A-HJ-NP-Za-km-z]{32,44})`)
	signatureRe := regexp.MustCompile(`(?m)Signature:\s+([1-9A-HJ-NP-Za-km-z]{32,88})`)

	programIDMatch := programIDRe.FindStringSubmatch(outputStr)
	signatureMatch := signatureRe.FindStringSubmatch(outputStr)

	if len(programIDMatch) < 2 {
		return solana.PublicKey{}, solana.Signature{}, fmt.Errorf("program id not found in output")
	}
	if len(signatureMatch) < 2 {
		return solana.PublicKey{}, solana.Signature{}, fmt.Errorf("signature not found in output")
	}

	programID, err := solana.PublicKeyFromBase58(programIDMatch[1])
	if err != nil {
		return solana.PublicKey{}, solana.Signature{}, fmt.Errorf("invalid Program Id: %w", err)
	}

	signature, err := solana.SignatureFromBase58(signatureMatch[1])
	if err != nil {
		return solana.PublicKey{}, solana.Signature{}, fmt.Errorf("invalid Signature: %w", err)
	}

	return programID, signature, nil
}

// WriteProgramBuffer writes a program binary to a buffer account for later upgrade.
// Returns the buffer account public key.
func WriteProgramBuffer(ctx context.Context, programSoFile, payerKeypairFile, rpcURL string) (solana.PublicKey, error) {
	absProgramFile, err := filepath.Abs(programSoFile)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to get absolute path for program file: %w", err)
	}

	absPayerFile, err := filepath.Abs(payerKeypairFile)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to get absolute path for payer keypair file: %w", err)
	}

	cmd := exec.Command(
		"solana", "program", "write-buffer",
		"--url", rpcURL,
		"--keypair", absPayerFile,
		"--use-rpc",
		absProgramFile,
	)

	cmd.Env = os.Environ()

	var stdoutBuf bytes.Buffer
	multiWriter := io.MultiWriter(os.Stdout, &stdoutBuf)

	cmd.Stdout = multiWriter
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		fmt.Println("Error write-buffer command", cmd.Args, err)
		return solana.PublicKey{}, err
	}

	stdoutBytes := stdoutBuf.Bytes()
	return getBufferAddressFromWriteBuffer(stdoutBytes)
}

// Parses raw Solana CLI write-buffer output and extracts buffer address.
func getBufferAddressFromWriteBuffer(stdout []byte) (solana.PublicKey, error) {
	outputStr := string(stdout)

	bufferRe := regexp.MustCompile(`(?m)Buffer:\s+([1-9A-HJ-NP-Za-km-z]{32,44})`)

	bufferMatch := bufferRe.FindStringSubmatch(outputStr)

	if len(bufferMatch) < 2 {
		return solana.PublicKey{}, fmt.Errorf("buffer address not found in output")
	}

	bufferAddr, err := solana.PublicKeyFromBase58(bufferMatch[1])
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("invalid buffer address: %w", err)
	}

	return bufferAddr, nil
}

// SetBufferAuthority changes the buffer authority to a new authority.
// This is required when preparing a buffer for upgrade via AccessManager.
func SetBufferAuthority(ctx context.Context, bufferAddress, newAuthority solana.PublicKey, currentAuthorityKeypairFile, rpcURL string) error {
	absAuthorityFile, err := filepath.Abs(currentAuthorityKeypairFile)
	if err != nil {
		return fmt.Errorf("failed to get absolute path for authority keypair file: %w", err)
	}

	cmd := exec.Command(
		"solana", "program", "set-buffer-authority",
		"--url", rpcURL,
		"--keypair", absAuthorityFile, // Fee payer
		"--buffer-authority", absAuthorityFile, // Current authority (signer)
		"--new-buffer-authority", newAuthority.String(),
		bufferAddress.String(),
	)

	cmd.Env = os.Environ()
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		fmt.Println("Error set-buffer-authority command", cmd.Args, err)
		return err
	}

	return nil
}

// SetUpgradeAuthority changes the program upgrade authority to a new authority.
// This is required to transfer control from deployer to AccessManager.
// Supports setting PDAs as new authority via --skip-new-upgrade-authority-signer-check flag.
func SetUpgradeAuthority(ctx context.Context, programID, newAuthority solana.PublicKey, currentAuthorityKeypairFile, rpcURL string) error {
	absAuthorityFile, err := filepath.Abs(currentAuthorityKeypairFile)
	if err != nil {
		return fmt.Errorf("failed to get absolute path for authority keypair file: %w", err)
	}

	cmd := exec.Command(
		"solana", "program", "set-upgrade-authority",
		"--url", rpcURL,
		"--keypair", absAuthorityFile, // Fee payer
		"--upgrade-authority", absAuthorityFile, // Current authority (signer)
		"--new-upgrade-authority", newAuthority.String(),
		"--skip-new-upgrade-authority-signer-check", // Allow setting PDA as authority
		programID.String(),
	)

	cmd.Env = os.Environ()
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		fmt.Println("Error set-upgrade-authority command", cmd.Args, err)
		return err
	}

	return nil
}

// UpgradeProgramDirect attempts to upgrade a program directly using BPF Loader (bypassing AccessManager).
// This is used in negative tests to verify that the old authority cannot bypass AccessManager
// after upgrade authority has been transferred to the AccessManager PDA.
func UpgradeProgramDirect(ctx context.Context, programID, bufferAddress solana.PublicKey, upgradeAuthorityKeypairFile, rpcURL string) error {
	absAuthorityFile, err := filepath.Abs(upgradeAuthorityKeypairFile)
	if err != nil {
		return fmt.Errorf("failed to get absolute path for authority keypair file: %w", err)
	}

	cmd := exec.Command(
		"solana", "program", "deploy",
		"--url", rpcURL,
		"--keypair", absAuthorityFile, // Fee payer
		"--upgrade-authority", absAuthorityFile, // Upgrade authority (should fail if not current authority)
		"--program-id", programID.String(),
		"--buffer", bufferAddress.String(),
		"--use-rpc",
	)

	cmd.Env = os.Environ()

	var stderrBuf bytes.Buffer
	cmd.Stdout = os.Stdout
	cmd.Stderr = io.MultiWriter(os.Stderr, &stderrBuf)

	if err := cmd.Run(); err != nil {
		// Include stderr in the error message for better error checking
		stderrStr := stderrBuf.String()
		if stderrStr != "" {
			return fmt.Errorf("%w: %s", err, stderrStr)
		}
		return err
	}

	return nil
}

// WriteKeypairToTempFile writes a wallet's private key to a temporary file in Solana CLI format (JSON byte array).
// The caller is responsible for removing the file when no longer needed.
func WriteKeypairToTempFile(wallet *solana.Wallet) (string, error) {
	// Solana CLI expects a JSON array of integers, not a base64 string.
	// json.Marshal([]byte) would produce base64, so convert to []int first.
	intSlice := make([]int, len(wallet.PrivateKey))
	for i, b := range wallet.PrivateKey {
		intSlice[i] = int(b)
	}
	jsonBytes, err := json.Marshal(intSlice)
	if err != nil {
		return "", fmt.Errorf("failed to marshal private key: %w", err)
	}

	tmpFile, err := os.CreateTemp("", "solana-keypair-*.json")
	if err != nil {
		return "", fmt.Errorf("failed to create temp file: %w", err)
	}
	defer tmpFile.Close()

	if _, err := tmpFile.Write(jsonBytes); err != nil {
		os.Remove(tmpFile.Name())
		return "", fmt.Errorf("failed to write keypair: %w", err)
	}

	return tmpFile.Name(), nil
}
