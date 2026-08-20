// SPDX-License-Identifier: Apache-2.0

package ethereum

import (
	"bytes"
	"context"
	"crypto/ecdsa"
	"encoding/hex"
	"fmt"
	"io"
	"math/big"
	"os"
	"os/exec"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"

	"cosmossdk.io/math"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

const forgeScriptTimeout = 5 * time.Minute

type Ethereum struct {
	ChainID         *big.Int
	RPC             string
	DockerRPC       string // Docker internal RPC address (for container-to-container communication)
	BeaconAPIClient *BeaconAPIClient
	RPCClient       *ethclient.Client

	Faucet *ecdsa.PrivateKey
}

func NewEthereum(ctx context.Context, rpc string, beaconAPIClient *BeaconAPIClient, faucet *ecdsa.PrivateKey) (Ethereum, error) {
	ethClient, err := ethclient.Dial(rpc)
	if err != nil {
		return Ethereum{}, err
	}
	chainID, err := ethClient.ChainID(ctx)
	if err != nil {
		return Ethereum{}, err
	}

	return Ethereum{
		ChainID:         chainID,
		RPC:             rpc,
		BeaconAPIClient: beaconAPIClient,
		RPCClient:       ethClient,
		Faucet:          faucet,
	}, nil
}

// GetBlockTime returns the latest block timestamp as Unix seconds.
func (e *Ethereum) GetBlockTime(ctx context.Context) (int64, error) {
	header, err := e.RPCClient.HeaderByNumber(ctx, nil)
	if err != nil {
		return 0, err
	}
	return int64(header.Time), nil
}

// BroadcastMessages broadcasts the provided messages to the given chain and signs them on behalf of the provided user.
// Once the transaction is mined, the receipt is returned.
func (e *Ethereum) BroadcastTx(ctx context.Context, userKey *ecdsa.PrivateKey, gasLimit uint64, address *ethcommon.Address, txBz []byte) (*ethtypes.Receipt, error) {
	txOpts, err := e.GetTransactOpts(userKey)
	if err != nil {
		return nil, err
	}

	tx := ethtypes.NewTx(&ethtypes.LegacyTx{
		Nonce:    txOpts.Nonce.Uint64(),
		To:       address,
		Value:    txOpts.Value,
		Gas:      gasLimit,
		GasPrice: txOpts.GasPrice,
		Data:     txBz,
	})

	signedTx, err := txOpts.Signer(txOpts.From, tx)
	if err != nil {
		return nil, err
	}

	err = e.RPCClient.SendTransaction(ctx, signedTx)
	if err != nil {
		return nil, err
	}

	receipt, err := e.GetTxReciept(ctx, signedTx.Hash())
	if err != nil {
		return nil, err
	}

	if receipt != nil && receipt.Status != ethtypes.ReceiptStatusSuccessful {
		return nil, fmt.Errorf("eth transaction was broadcasted, but failed on-chain with status %d", receipt.Status)
	}

	return receipt, nil
}

func (e Ethereum) ForgeScript(deployer *ecdsa.PrivateKey, solidityContract string, args ...string) ([]byte, error) {
	cmdArgs := []string{
		"script", "--rpc-url", e.RPC, "--private-key",
		hex.EncodeToString(crypto.FromECDSA(deployer)), "--broadcast",
		"--non-interactive", "-vvvv", solidityContract,
	}
	cmdArgs = append(cmdArgs, args...)
	ctx, cancel := context.WithTimeout(context.Background(), forgeScriptTimeout)
	defer cancel()

	cmd := exec.CommandContext(ctx, "forge", cmdArgs...)
	// Tests run from the repository root; forge must run from the Foundry project root
	// so that foundry.toml and script paths resolve.
	cmd.Dir = testvalues.SolidityProjectDir

	faucetAddress := crypto.PubkeyToAddress(e.Faucet.PublicKey)
	extraEnv := []string{
		fmt.Sprintf("%s=%s", testvalues.EnvKeyE2EFacuetAddress, faucetAddress.Hex()),
	}

	cmd.Env = os.Environ()
	cmd.Env = append(cmd.Env, extraEnv...)

	var stdoutBuf bytes.Buffer
	var stderrBuf bytes.Buffer

	// Create a MultiWriter to write to both os.Stdout and the buffer
	stdoutWriter := io.MultiWriter(os.Stdout, &stdoutBuf)
	stderrWriter := io.MultiWriter(os.Stderr, &stderrBuf)

	// Set the command's stdout to the MultiWriter
	cmd.Stdout = stdoutWriter
	cmd.Stderr = stderrWriter

	// Run the command
	if err := cmd.Run(); err != nil {
		if ctx.Err() == context.DeadlineExceeded {
			err = fmt.Errorf("forge script timed out after %s: %w", forgeScriptTimeout, err)
		}
		fmt.Println("Error start command", cmd.Args, err)
		return stdoutBuf.Bytes(), fmt.Errorf("%w\n%s", err, stderrBuf.String())
	}

	// Get the output as byte slices
	stdoutBytes := stdoutBuf.Bytes()

	return stdoutBytes, nil
}

func (e Ethereum) CreateUser() (*ecdsa.PrivateKey, error) {
	key, err := crypto.GenerateKey()
	if err != nil {
		return nil, err
	}

	return key, nil
}

// CreateUserFromKey creates a user from a hex-encoded private key.
func (e Ethereum) CreateUserFromKey(privateKeyHex string) (*ecdsa.PrivateKey, error) {
	return crypto.HexToECDSA(privateKeyHex)
}

// CreateAndFundUserFromKey creates and funds a user from a hex-encoded private key.
func (e Ethereum) CreateAndFundUserFromKey(privateKeyHex string) (*ecdsa.PrivateKey, error) {
	key, err := e.CreateUserFromKey(privateKeyHex)
	if err != nil {
		return nil, err
	}

	address := crypto.PubkeyToAddress(key.PublicKey)
	if err := e.FundUser(address, testvalues.StartingEthBalance); err != nil {
		return nil, err
	}

	return key, nil
}

func (e Ethereum) CreateAndFundUser() (*ecdsa.PrivateKey, error) {
	key, err := e.CreateUser()
	if err != nil {
		return nil, err
	}

	address := crypto.PubkeyToAddress(key.PublicKey)
	if err := e.FundUser(address, testvalues.StartingEthBalance); err != nil {
		return nil, err
	}

	return key, nil
}

func (e Ethereum) FundUser(address ethcommon.Address, amount math.Int) error {
	return e.SendEth(e.Faucet, address, amount)
}

func (e Ethereum) SendEth(key *ecdsa.PrivateKey, toAddress ethcommon.Address, amount math.Int) error {
	ctx := context.Background()
	txHash, err := SendTx(ctx, e.RPCClient, key, &toAddress, amount.BigInt(), nil, 0)
	if err != nil {
		return err
	}
	receipt, err := e.GetTxReciept(ctx, txHash)
	if err != nil {
		return err
	}
	if receipt.Status != ethtypes.ReceiptStatusSuccessful {
		return fmt.Errorf("SendEth transaction failed on-chain with status %d", receipt.Status)
	}
	return nil
}

func (e *Ethereum) GetTxReciept(ctx context.Context, hash ethcommon.Hash) (*ethtypes.Receipt, error) {
	return GetTxReciept(ctx, e.RPCClient, hash)
}

func (e *Ethereum) GetTransactOpts(key *ecdsa.PrivateKey) (*bind.TransactOpts, error) {
	return GetTransactOpts(context.Background(), e.RPCClient, key)
}

// SetIntervalMining sets the interval (in seconds) at which Anvil mines new blocks.
// Pass 0 to disable automatic mining (blocks won't be mined until manually triggered).
// This is an Anvil-specific RPC method (evm_setIntervalMining).
func (e *Ethereum) SetIntervalMining(ctx context.Context, intervalSeconds uint64) error {
	var result interface{}
	err := e.RPCClient.Client().CallContext(ctx, &result, "evm_setIntervalMining", intervalSeconds)
	if err != nil {
		return fmt.Errorf("failed to set interval mining: %w", err)
	}
	return nil
}

// MineBlock mines a single block.
// This is an Anvil-specific RPC method (evm_mine).
func (e *Ethereum) MineBlock(ctx context.Context) error {
	var result interface{}
	err := e.RPCClient.Client().CallContext(ctx, &result, "evm_mine")
	if err != nil {
		return fmt.Errorf("failed to mine block: %w", err)
	}
	return nil
}
