package chainconfig

import (
	"context"
	"crypto/ecdsa"
	"embed"
	"encoding/json"
	"fmt"
	"io"
	"io/fs"
	"math/big"
	"os"
	"path/filepath"
	"strings"
	"time"

	dockernetwork "github.com/docker/docker/api/types/network"
	dockerclient "github.com/moby/moby/client"

	"github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/testcontainers/testcontainers-go/modules/compose"
	"github.com/testcontainers/testcontainers-go/wait"

	"github.com/cosmos/interchaintest/v11/testutil"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

const (
	besuQBFTComposeFile = "docker-compose.yml"
	besuQBFTProjectName = "besu-qbft"

	defaultBesuQBFTSubnet  = "10.42.0.0/16"
	defaultBesuQBFTGateway = "10.42.0.1"

	besuQBFTReadyStableBlocks     = 6
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

type BesuQBFTChain struct {
	RPC       string
	DockerRPC string
	Faucet    *ecdsa.PrivateKey

	stack      *compose.DockerCompose
	projectDir string
}

func SpinUpBesuQBFT(ctx context.Context, params BesuQBFTParams) (chain BesuQBFTChain, err error) {
	faucet, err := crypto.HexToECDSA(testvalues.E2EDeployerPrivateKeyHex)
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("parse besu qbft faucet key: %w", err)
	}

	projectDir, err := os.MkdirTemp("", "besu-qbft-*")
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("create besu qbft temp dir: %w", err)
	}

	chain = BesuQBFTChain{
		Faucet:     faucet,
		projectDir: projectDir,
	}
	defer func() {
		if err != nil {
			chain.Destroy(context.Background())
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

	stack, err := compose.NewDockerComposeWith(
		compose.StackIdentifier(fmt.Sprintf("%s-%d", besuQBFTProjectName, time.Now().UnixNano())),
		compose.WithStackFiles(filepath.Join(projectDir, besuQBFTComposeFile)),
	)
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("create besu qbft compose stack: %w", err)
	}
	chain.stack = stack

	if err := stack.
		WaitForService("validator1", wait.ForListeningPort("8545/tcp")).
		Up(ctx, compose.Wait(true)); err != nil {
		return BesuQBFTChain{}, fmt.Errorf("start besu qbft compose stack: %w", err)
	}

	validator1, err := stack.ServiceContainer(ctx, "validator1")
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("get validator1 container: %w", err)
	}

	mappedPort, err := validator1.MappedPort(ctx, "8545/tcp")
	if err != nil {
		return BesuQBFTChain{}, fmt.Errorf("resolve validator1 rpc port: %w", err)
	}

	chain.RPC = fmt.Sprintf("http://127.0.0.1:%s", mappedPort.Port())
	if params.DockerRPCAlias != "" {
		chain.DockerRPC = fmt.Sprintf("http://%s:8545", params.DockerRPCAlias)
	}

	if params.InterchainNetworkID != "" {
		if err := connectBesuQBFTToInterchainNetwork(ctx, params.InterchainNetworkID, validator1.GetContainerID(), params.DockerRPCAlias); err != nil {
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
	if c.stack != nil {
		if err := c.stack.Down(ctx, compose.RemoveOrphans(true), compose.RemoveVolumes(true)); err != nil {
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
	if c.stack == nil {
		return nil
	}

	for _, service := range besuQBFTServices {
		container, err := c.stack.ServiceContainer(ctx, service)
		if err != nil {
			return fmt.Errorf("get %s container: %w", service, err)
		}

		logs, err := container.Logs(ctx)
		if err != nil {
			return fmt.Errorf("get %s logs: %w", service, err)
		}

		fmt.Printf("===== %s logs =====\n", service)
		if _, err := io.Copy(os.Stdout, logs); err != nil {
			_ = logs.Close()
			return fmt.Errorf("copy %s logs: %w", service, err)
		}
		fmt.Println()
		if err := logs.Close(); err != nil {
			return fmt.Errorf("close %s logs: %w", service, err)
		}
	}

	return nil
}

func materializeBesuQBFTAssets(dst string) error {
	sub, err := fs.Sub(besuQBFTAssets, "testdata/besu/qbft")
	if err != nil {
		return err
	}

	return fs.WalkDir(sub, ".", func(path string, d fs.DirEntry, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}

		target := filepath.Join(dst, path)
		if d.IsDir() {
			return os.MkdirAll(target, 0o755)
		}

		contents, err := fs.ReadFile(sub, path)
		if err != nil {
			return err
		}

		mode := os.FileMode(0o644)
		if filepath.Base(path) == "key" || strings.HasPrefix(path, "keys/") {
			mode = 0o600
		}

		return os.WriteFile(target, contents, mode)
	})
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

	return os.WriteFile(path, updated, 0o644)
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

	return os.WriteFile(path, []byte(replacer.Replace(string(contents))), 0o644)
}

func connectBesuQBFTToInterchainNetwork(ctx context.Context, interchainNetworkID, containerID, alias string) error {
	dockerClient, err := dockerclient.NewClientWithOpts(dockerclient.FromEnv, dockerclient.WithAPIVersionNegotiation())
	if err != nil {
		return fmt.Errorf("create docker client: %w", err)
	}
	defer dockerClient.Close()

	settings := &dockernetwork.EndpointSettings{}
	if alias != "" {
		settings.Aliases = []string{alias}
	}

	return dockerClient.NetworkConnect(ctx, interchainNetworkID, containerID, settings)
}

func waitForBesuQBFTReady(ctx context.Context, rpcURL string) error {
	var (
		stableStartBlock uint64
		lastErr          error
	)

	err := testutil.WaitForCondition(3*time.Minute, 2*time.Second, func() (bool, error) {
		client, err := ethclient.DialContext(ctx, rpcURL)
		if err != nil {
			lastErr = err
			return false, nil
		}
		defer client.Close()

		syncProgress, err := client.SyncProgress(ctx)
		if err != nil {
			lastErr = err
			stableStartBlock = 0
			return false, nil
		}
		if syncProgress != nil {
			stableStartBlock = 0
			return false, nil
		}

		peerCount, err := client.PeerCount(ctx)
		if err != nil {
			lastErr = err
			stableStartBlock = 0
			return false, nil
		}
		if peerCount < uint64(len(besuQBFTServices)-1) {
			stableStartBlock = 0
			return false, nil
		}

		blockNumber, err := client.BlockNumber(ctx)
		if err != nil || blockNumber == 0 {
			lastErr = err
			stableStartBlock = 0
			return false, nil
		}

		var validators []common.Address
		if err := client.Client().CallContext(ctx, &validators, "qbft_getValidatorsByBlockNumber", "latest"); err != nil {
			lastErr = err
			stableStartBlock = 0
			return false, nil
		}

		if len(validators) != len(besuQBFTServices) {
			stableStartBlock = 0
			return false, nil
		}

		if stableStartBlock == 0 {
			stableStartBlock = blockNumber
			return false, nil
		}

		return blockNumber >= stableStartBlock+besuQBFTReadyStableBlocks, nil
	})
	if err != nil && lastErr != nil {
		return fmt.Errorf("%w (last readiness observation: %v)", err, lastErr)
	}
	return err
}

func waitForBesuQBFTTransactionHandling(ctx context.Context, rpcURL string, key *ecdsa.PrivateKey) error {
	var lastErr error

	err := testutil.WaitForCondition(2*time.Minute, 2*time.Second, func() (bool, error) {
		client, err := ethclient.DialContext(ctx, rpcURL)
		if err != nil {
			lastErr = err
			return false, nil
		}
		defer client.Close()

		txHash, err := sendBesuQBFTProbeTx(ctx, client, key)
		if err != nil {
			lastErr = err
			return false, nil
		}

		receipt, err := waitForBesuQBFTProbeReceipt(ctx, client, txHash)
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

func waitForBesuQBFTProbeReceipt(ctx context.Context, client *ethclient.Client, txHash common.Hash) (*ethtypes.Receipt, error) {
	receiptCtx, cancel := context.WithTimeout(ctx, besuQBFTTxProbeReceiptTimeout)
	defer cancel()

	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	for {
		receipt, err := client.TransactionReceipt(receiptCtx, txHash)
		if err == nil && receipt != nil {
			return receipt, nil
		}

		select {
		case <-receiptCtx.Done():
			return nil, receiptCtx.Err()
		case <-ticker.C:
		}
	}
}
