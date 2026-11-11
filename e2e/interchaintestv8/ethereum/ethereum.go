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
	"strconv"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"

	"cosmossdk.io/math"

	"github.com/lfdt-ibc/ibc-contracts/e2e/v8/testvalues"
)

const forgeScriptTimeout = 5 * time.Minute

// gloasGasLimitMultiplier scales gas limits on Gloas chains, where state creation costs around 7
// times more than the limits and simulations in the tests account for. It only applies once Gloas
// is active, since the scaled limits are above the transaction gas limit cap of Fulu. Limits are
// scaled rather than estimated because simulations of relay transactions can revert when their
// proofs are newer than the latest block, even though the transactions succeed once sent.
const gloasGasLimitMultiplier = 10

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

	isGloas, err := e.isGloasActive()
	if err != nil {
		return nil, err
	}
	if isGloas {
		gasLimit *= gloasGasLimitMultiplier
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
		return nil, fmt.Errorf("eth transaction was broadcasted, but failed on-chain with status %d (gas used %d of %d)", receipt.Status, receipt.GasUsed, gasLimit)
	}

	return receipt, nil
}

func (e Ethereum) ForgeScript(deployer *ecdsa.PrivateKey, solidityContract string, args ...string) ([]byte, error) {
	return e.forgeScript(forgeScriptTimeout, deployer, solidityContract, args...)
}

func (e Ethereum) forgeScript(timeout time.Duration, deployer *ecdsa.PrivateKey, solidityContract string, args ...string) ([]byte, error) {
	cmdArgs := []string{
		"script", "--rpc-url", e.RPC, "--private-key",
		hex.EncodeToString(crypto.FromECDSA(deployer)), "--broadcast",
		"--non-interactive", "-vvvv", solidityContract,
	}
	cmdArgs = append(cmdArgs, args...)

	isGloas, err := e.isGloasActive()
	if err != nil {
		return nil, err
	}
	if isGloas {
		// Forge simulates transactions with the gas rules of its configured EVM version
		cmdArgs = append(cmdArgs, "--gas-estimate-multiplier", strconv.Itoa(gloasGasLimitMultiplier*100))
	}

	ctx, cancel := context.WithTimeout(context.Background(), timeout)
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

	// Preserve live output while retaining stdout for deploy-result parsing.
	stdoutWriter := io.MultiWriter(os.Stdout, &stdoutBuf)

	cmd.Stdout = stdoutWriter
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		if ctx.Err() == context.DeadlineExceeded {
			return nil, fmt.Errorf("forge script %q timed out after %s: %w", solidityContract, timeout, ctx.Err())
		}
		return nil, fmt.Errorf("forge script %q failed: %w", solidityContract, err)
	}

	return stdoutBuf.Bytes(), nil
}

// isGloasActive returns whether the chain has reached the Gloas fork. Chains without a beacon node
// never reach it.
func (e Ethereum) isGloasActive() (bool, error) {
	if e.BeaconAPIClient == nil {
		return false, nil
	}

	spec, err := e.BeaconAPIClient.GetSpec()
	if err != nil {
		return false, err
	}
	header, err := e.BeaconAPIClient.GetHeader("head")
	if err != nil {
		return false, err
	}

	return uint64(header.Header.Message.Slot)/spec.SlotsPerEpoch >= spec.GloasForkEpoch, nil
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
