// SPDX-License-Identifier: Apache-2.0

package chainconfig

import (
	"context"
	"crypto/ecdsa"
	"embed"
	"encoding/json"
	"fmt"
	"io/fs"
	"math/big"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	dockernetwork "github.com/docker/docker/api/types/network"
	dockerclient "github.com/moby/moby/client"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"

	"github.com/cosmos/interchaintest/v11/testutil"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

const (
	besuQBFTComposeFile = "docker-compose.yml"
	besuQBFTProjectName = "besu-qbft"

	defaultBesuQBFTChainID uint64 = 1337
	defaultBesuQBFTSubnet         = "10.42.0.0/16"
	defaultBesuQBFTGateway        = "10.42.0.1"

	besuQBFTTxProbeReceiptTimeout = 30 * time.Second
)

var defaultBesuQBFTValidatorIPs = [4]string{"10.42.0.2", "10.42.0.3", "10.42.0.4", "10.42.0.5"}

//go:embed testdata/besu/qbft
var besuQBFTAssets embed.FS

var besuQBFTServices = []string{"validator1", "validator2", "validator3", "validator4"}

type BesuQBFTParams struct {
	ChainID             uint64
	Subnet              string
	Gateway             string
	ValidatorIPs        [4]string
	DockerRPCAlias      string
	InterchainNetworkID string
}

// DefaultBesuQBFTParams returns the topology from the embedded Besu fixture.
func DefaultBesuQBFTParams() BesuQBFTParams {
	return BesuQBFTParams{
		ChainID:      defaultBesuQBFTChainID,
		Subnet:       defaultBesuQBFTSubnet,
		Gateway:      defaultBesuQBFTGateway,
		ValidatorIPs: defaultBesuQBFTValidatorIPs,
	}
}

type BesuQBFTChain struct {
	RPC       string
	DockerRPC string
	Faucet    *ecdsa.PrivateKey

	projectName string
	projectDir  string
}

func (c BesuQBFTChain) WaitForTransactionHandling(ctx context.Context) error {
	return waitForBesuQBFTTransactionHandling(ctx, c.RPC, c.Faucet)
}

func SpinUpBesuQBFT(ctx context.Context, params BesuQBFTParams) (chain BesuQBFTChain, err error) {
	faucet, err := crypto.HexToECDSA(testvalues.E2EDeployerPrivateKeyHex)
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("parse besu qbft faucet key: %w", err)
	}

	projectDir, err := os.MkdirTemp("", besuQBFTProjectName+"-*")
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("create besu qbft temp dir: %w", err)
	}

	chain = BesuQBFTChain{
		Faucet:     faucet,
		projectDir: projectDir,
	}
	cleanupChain := BesuQBFTChain{projectDir: projectDir}
	defer func() {
		if err != nil {
			cleanupCtx := context.Background()
			if cleanupChain.projectName != "" {
				if logErr := cleanupChain.DumpLogs(cleanupCtx); logErr != nil {
					fmt.Printf("failed to dump besu qbft logs after startup error: %v\n", logErr)
				}
			}
			cleanupChain.Destroy(cleanupCtx)
		}
	}()

	if err := materializeBesuQBFTAssets(projectDir); err != nil {
		return BesuQBFTChain{}, fmt.Errorf("materialize besu qbft assets: %w", err)
	}

	if err := patchBesuQBFTGenesis(filepath.Join(projectDir, "genesis.json"), params.ChainID); err != nil {
		return BesuQBFTChain{}, fmt.Errorf("patch besu qbft genesis: %w", err)
	}

	if err := patchBesuQBFTCompose(filepath.Join(projectDir, besuQBFTComposeFile), params); err != nil {
		return BesuQBFTChain{}, fmt.Errorf("patch besu qbft compose file: %w", err)
	}

	chain.projectName = filepath.Base(projectDir)
	cleanupChain.projectName = chain.projectName
	if _, err := chain.runCompose(ctx, "up", "--detach"); err != nil {
		return BesuQBFTChain{}, fmt.Errorf("start besu qbft compose stack: %w", err)
	}

	validator1Output, err := chain.runCompose(ctx, "ps", "-q", "validator1")
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("get validator1 container: %w", err)
	}
	validator1ID := strings.TrimSpace(string(validator1Output))
	if validator1ID == "" {
		return BesuQBFTChain{}, fmt.Errorf("get validator1 container: docker compose returned no container ID")
	}

	dockerClient, err := dockerclient.NewClientWithOpts(dockerclient.FromEnv, dockerclient.WithAPIVersionNegotiation())
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("create docker client: %w", err)
	}
	defer dockerClient.Close()

	validator1, err := dockerClient.ContainerInspect(ctx, validator1ID)
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("inspect validator1 container: %w", err)
	}
	if validator1.NetworkSettings == nil {
		return BesuQBFTChain{}, fmt.Errorf("resolve validator1 rpc port: container has no network settings")
	}
	bindings := validator1.NetworkSettings.Ports["8545/tcp"]
	if len(bindings) == 0 || bindings[0].HostPort == "" {
		return BesuQBFTChain{}, fmt.Errorf("resolve validator1 rpc port: no 8545/tcp binding")
	}

	chain.RPC = fmt.Sprintf("http://127.0.0.1:%s", bindings[0].HostPort)
	if params.DockerRPCAlias != "" {
		chain.DockerRPC = fmt.Sprintf("http://%s:8545", params.DockerRPCAlias)
	}

	if params.InterchainNetworkID != "" {
		settings := &dockernetwork.EndpointSettings{}
		if params.DockerRPCAlias != "" {
			settings.Aliases = []string{params.DockerRPCAlias}
		}
		if err := dockerClient.NetworkConnect(ctx, params.InterchainNetworkID, validator1ID, settings); err != nil {
			return BesuQBFTChain{}, fmt.Errorf("connect besu qbft rpc container to interchain network: %w", err)
		}
	}

	if err := waitForBesuQBFTReady(ctx, chain.RPC); err != nil {
		return BesuQBFTChain{}, fmt.Errorf("wait for besu qbft readiness: %w", err)
	}

	if err := waitForBesuQBFTTransactionHandling(ctx, chain.RPC, faucet); err != nil {
		return BesuQBFTChain{}, fmt.Errorf("wait for besu qbft transaction handling: %w", err)
	}

	return chain, nil
}

func (c BesuQBFTChain) Destroy(ctx context.Context) {
	if c.projectName != "" && c.projectDir != "" {
		if _, err := c.runCompose(ctx, "down", "--volumes", "--remove-orphans"); err != nil {
			fmt.Printf("failed to tear down besu qbft stack: %v\n", err)
		}
	}

	if c.projectDir != "" {
		if err := os.RemoveAll(c.projectDir); err != nil {
			fmt.Printf("failed to remove besu qbft temp dir %s: %v\n", c.projectDir, err)
		}
	}
}

func (c BesuQBFTChain) DumpLogs(ctx context.Context) error {
	if c.projectName == "" || c.projectDir == "" {
		return nil
	}

	args := append([]string{"logs", "--no-color"}, besuQBFTServices...)
	logs, err := c.runCompose(ctx, args...)
	if len(logs) > 0 {
		fmt.Print(string(logs))
	}
	return err
}

func (c BesuQBFTChain) runCompose(ctx context.Context, args ...string) ([]byte, error) {
	composeArgs := []string{
		"compose",
		"--project-name", c.projectName,
		"--file", filepath.Join(c.projectDir, besuQBFTComposeFile),
	}
	composeArgs = append(composeArgs, args...)

	output, err := exec.CommandContext(ctx, "docker", composeArgs...).CombinedOutput()
	if err != nil {
		return output, fmt.Errorf("docker compose %s: %w: %s", strings.Join(args, " "), err, strings.TrimSpace(string(output)))
	}
	return output, nil
}

func materializeBesuQBFTAssets(dst string) error {
	sub, err := fs.Sub(besuQBFTAssets, "testdata/besu/qbft")
	if err != nil {
		return err
	}

	return os.CopyFS(dst, sub)
}

func patchBesuQBFTGenesis(path string, chainID uint64) error {
	contents, err := os.ReadFile(path)
	if err != nil {
		return err
	}

	var genesis map[string]any
	if err := json.Unmarshal(contents, &genesis); err != nil {
		return err
	}

	config, ok := genesis["config"].(map[string]any)
	if !ok {
		return fmt.Errorf("genesis config missing or invalid")
	}
	config["chainId"] = chainID

	updated, err := json.MarshalIndent(genesis, "", "  ")
	if err != nil {
		return err
	}
	updated = append(updated, '\n')

	// Besu reads the generated genesis file as an unprivileged container user.
	return os.WriteFile(path, updated, 0o644) //nolint:gosec
}

func patchBesuQBFTCompose(path string, params BesuQBFTParams) error {
	contents, err := os.ReadFile(path)
	if err != nil {
		return err
	}

	replacer := strings.NewReplacer(
		defaultBesuQBFTSubnet, params.Subnet,
		defaultBesuQBFTGateway, params.Gateway,
		defaultBesuQBFTValidatorIPs[0], params.ValidatorIPs[0],
		defaultBesuQBFTValidatorIPs[1], params.ValidatorIPs[1],
		defaultBesuQBFTValidatorIPs[2], params.ValidatorIPs[2],
		defaultBesuQBFTValidatorIPs[3], params.ValidatorIPs[3],
	)

	// Docker Compose reads this generated configuration outside the Go process.
	return os.WriteFile(path, []byte(replacer.Replace(string(contents))), 0o644) //nolint:gosec
}

func waitForBesuQBFTReady(ctx context.Context, rpcURL string) error {
	client, err := ethclient.DialContext(ctx, rpcURL)
	if err != nil {
		return fmt.Errorf("dial readiness rpc: %w", err)
	}
	defer client.Close()

	var lastErr error
	err = testutil.WaitForCondition(3*time.Minute, 2*time.Second, func() (bool, error) {
		peerCount, err := client.PeerCount(ctx)
		if err != nil {
			lastErr = err
			return false, nil
		}
		if peerCount < uint64(len(besuQBFTServices)-1) {
			lastErr = fmt.Errorf("peer count %d, want at least %d", peerCount, len(besuQBFTServices)-1)
			return false, nil
		}

		blockNumber, err := client.BlockNumber(ctx)
		if err != nil {
			lastErr = err
			return false, nil
		}
		if blockNumber == 0 {
			lastErr = fmt.Errorf("block number is still zero")
			return false, nil
		}

		var validators []common.Address
		if err := client.Client().CallContext(ctx, &validators, "qbft_getValidatorsByBlockNumber", "latest"); err != nil {
			lastErr = err
			return false, nil
		}

		if len(validators) != len(besuQBFTServices) {
			lastErr = fmt.Errorf("validator count %d, want exactly %d", len(validators), len(besuQBFTServices))
			return false, nil
		}

		return true, nil
	})
	if err != nil && lastErr != nil {
		return fmt.Errorf("%w (last readiness observation: %v)", err, lastErr)
	}
	return err
}

func waitForBesuQBFTTransactionHandling(ctx context.Context, rpcURL string, key *ecdsa.PrivateKey) error {
	client, err := ethclient.DialContext(ctx, rpcURL)
	if err != nil {
		return fmt.Errorf("dial transaction probe rpc: %w", err)
	}
	defer client.Close()

	var lastErr error
	err = testutil.WaitForCondition(2*time.Minute, 2*time.Second, func() (bool, error) {
		txHash, err := sendBesuQBFTProbeTx(ctx, client, key)
		if err != nil {
			lastErr = err
			return false, nil
		}

		receiptCtx, cancel := context.WithTimeout(ctx, besuQBFTTxProbeReceiptTimeout)
		receipt, err := bind.WaitMinedHash(receiptCtx, client, txHash)
		cancel()
		if err != nil {
			lastErr = err
			return false, nil
		}
		if receipt.Status != ethtypes.ReceiptStatusSuccessful {
			return false, fmt.Errorf("besu qbft transaction probe failed on-chain with status %d", receipt.Status)
		}

		return true, nil
	})
	if err != nil && lastErr != nil {
		return fmt.Errorf("%w (last transaction probe error: %v)", err, lastErr)
	}
	return err
}

func sendBesuQBFTProbeTx(ctx context.Context, client *ethclient.Client, key *ecdsa.PrivateKey) (common.Hash, error) {
	from := crypto.PubkeyToAddress(key.PublicKey)

	chainID, err := client.ChainID(ctx)
	if err != nil {
		return common.Hash{}, err
	}

	nonce, err := client.PendingNonceAt(ctx, from)
	if err != nil {
		return common.Hash{}, err
	}

	tx := ethtypes.NewTx(&ethtypes.DynamicFeeTx{
		ChainID:   chainID,
		Nonce:     nonce,
		To:        &from,
		Value:     big.NewInt(0),
		Gas:       21_000,
		GasFeeCap: big.NewInt(1),
		GasTipCap: big.NewInt(1),
	})

	signedTx, err := ethtypes.SignTx(tx, ethtypes.LatestSignerForChainID(chainID), key)
	if err != nil {
		return common.Hash{}, err
	}
	if err := client.SendTransaction(ctx, signedTx); err != nil {
		return common.Hash{}, err
	}

	return signedTx.Hash(), nil
}
