// SPDX-License-Identifier: Apache-2.0

package chainconfig

import (
	"context"
	"crypto/ecdsa"
	"embed"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"math/big"
	"os"
	"os/exec"
	"path/filepath"
	"slices"
	"strings"
	"time"

	dockernetwork "github.com/docker/docker/api/types/network"
	dockerclient "github.com/moby/moby/client"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/ethereum/go-ethereum/rlp"

	"github.com/cosmos/interchaintest/v11/testutil"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

const (
	besuComposeFile = "docker-compose.yml"
	besuProjectName = "besu"

	defaultBesuChainID uint64 = 1337
	defaultBesuSubnet         = "10.42.0.0/16"
	defaultBesuGateway        = "10.42.0.1"

	besuTxProbeReceiptTimeout = 30 * time.Second
)

var defaultBesuValidatorIPs = [5]string{"10.42.0.2", "10.42.0.3", "10.42.0.4", "10.42.0.5", "10.42.0.6"}

//go:embed testdata/besu
var besuAssets embed.FS

var besuServices = []string{"validator1", "validator2", "validator3", "validator4"}

// besuSpareService is not in the genesis validator set and only runs once AddValidator is called for it.
const (
	besuSpareService = "validator5"
	besuSpareProfile = "spare"
)

type BesuParams struct {
	// Consensus is testvalues.BesuConsensusQBFT or testvalues.BesuConsensusIBFT2.
	Consensus string
	ChainID   uint64
	Subnet    string
	Gateway   string
	// ValidatorIPs are the IPs of the genesis validators followed by the spare validator.
	ValidatorIPs        [5]string
	DockerRPCAlias      string
	InterchainNetworkID string
}

// DefaultBesuParams returns the topology from the embedded Besu fixture.
func DefaultBesuParams(consensus string) BesuParams {
	return BesuParams{
		Consensus:    consensus,
		ChainID:      defaultBesuChainID,
		Subnet:       defaultBesuSubnet,
		Gateway:      defaultBesuGateway,
		ValidatorIPs: defaultBesuValidatorIPs,
	}
}

type BesuChain struct {
	RPC       string
	DockerRPC string
	Faucet    *ecdsa.PrivateKey

	// rpcPrefix is the namespace of the consensus specific RPC methods.
	rpcPrefix   string
	projectName string
	projectDir  string
}

func (c BesuChain) WaitForTransactionHandling(ctx context.Context) error {
	return waitForBesuTransactionHandling(ctx, c.RPC, c.Faucet)
}

func SpinUpBesu(ctx context.Context, params BesuParams) (chain BesuChain, err error) {
	var rpcPrefix string
	switch params.Consensus {
	case testvalues.BesuConsensusQBFT:
		rpcPrefix = "qbft"
	case testvalues.BesuConsensusIBFT2:
		rpcPrefix = "ibft"
	default:
		return BesuChain{}, fmt.Errorf("unsupported besu consensus %q", params.Consensus)
	}

	faucet, err := crypto.HexToECDSA(testvalues.E2EDeployerPrivateKeyHex)
	if err != nil {
		return BesuChain{}, fmt.Errorf("parse besu faucet key: %w", err)
	}

	projectDir, err := os.MkdirTemp("", besuProjectName+"-*")
	if err != nil {
		return BesuChain{}, fmt.Errorf("create besu temp dir: %w", err)
	}

	chain = BesuChain{
		Faucet:     faucet,
		rpcPrefix:  rpcPrefix,
		projectDir: projectDir,
	}
	cleanupChain := BesuChain{projectDir: projectDir}
	defer func() {
		if err != nil {
			cleanupCtx := context.Background()
			if cleanupChain.projectName != "" {
				if logErr := cleanupChain.DumpLogs(cleanupCtx); logErr != nil {
					fmt.Printf("failed to dump besu logs after startup error: %v\n", logErr)
				}
			}
			cleanupChain.Destroy(cleanupCtx)
		}
	}()

	if err := materializeBesuAssets(projectDir); err != nil {
		return BesuChain{}, fmt.Errorf("materialize besu assets: %w", err)
	}

	if err := patchBesuGenesis(filepath.Join(projectDir, "genesis.json"), params.ChainID, params.Consensus); err != nil {
		return BesuChain{}, fmt.Errorf("patch besu genesis: %w", err)
	}

	if err := patchBesuConfig(filepath.Join(projectDir, "config.toml"), rpcPrefix); err != nil {
		return BesuChain{}, fmt.Errorf("patch besu config: %w", err)
	}

	if err := patchBesuTopology(projectDir, params); err != nil {
		return BesuChain{}, fmt.Errorf("patch besu topology: %w", err)
	}

	chain.projectName = filepath.Base(projectDir)
	cleanupChain.projectName = chain.projectName
	if _, err := chain.runCompose(ctx, "up", "--detach"); err != nil {
		return BesuChain{}, fmt.Errorf("start besu compose stack: %w", err)
	}

	validator1Output, err := chain.runCompose(ctx, "ps", "-q", "validator1")
	if err != nil {
		return BesuChain{}, fmt.Errorf("get validator1 container: %w", err)
	}
	validator1ID := strings.TrimSpace(string(validator1Output))
	if validator1ID == "" {
		return BesuChain{}, fmt.Errorf("get validator1 container: docker compose returned no container ID")
	}

	dockerClient, err := dockerclient.NewClientWithOpts(dockerclient.FromEnv, dockerclient.WithAPIVersionNegotiation())
	if err != nil {
		return BesuChain{}, fmt.Errorf("create docker client: %w", err)
	}
	defer dockerClient.Close()

	validator1, err := dockerClient.ContainerInspect(ctx, validator1ID)
	if err != nil {
		return BesuChain{}, fmt.Errorf("inspect validator1 container: %w", err)
	}
	if validator1.NetworkSettings == nil {
		return BesuChain{}, fmt.Errorf("resolve validator1 rpc port: container has no network settings")
	}
	bindings := validator1.NetworkSettings.Ports["8545/tcp"]
	if len(bindings) == 0 || bindings[0].HostPort == "" {
		return BesuChain{}, fmt.Errorf("resolve validator1 rpc port: no 8545/tcp binding")
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
			return BesuChain{}, fmt.Errorf("connect besu rpc container to interchain network: %w", err)
		}
	}

	if err := waitForBesuReady(ctx, chain.RPC, rpcPrefix); err != nil {
		return BesuChain{}, fmt.Errorf("wait for besu readiness: %w", err)
	}

	if err := waitForBesuTransactionHandling(ctx, chain.RPC, faucet); err != nil {
		return BesuChain{}, fmt.Errorf("wait for besu transaction handling: %w", err)
	}

	return chain, nil
}

func (c BesuChain) Destroy(ctx context.Context) {
	if c.projectName != "" && c.projectDir != "" {
		if _, err := c.runCompose(ctx, "--profile", besuSpareProfile, "down", "--volumes", "--remove-orphans"); err != nil {
			fmt.Printf("failed to tear down besu stack: %v\n", err)
		}
	}

	if c.projectDir != "" {
		if err := os.RemoveAll(c.projectDir); err != nil {
			fmt.Printf("failed to remove besu temp dir %s: %v\n", c.projectDir, err)
		}
	}
}

func (c BesuChain) DumpLogs(ctx context.Context) error {
	if c.projectName == "" || c.projectDir == "" {
		return nil
	}

	args := append([]string{"--profile", besuSpareProfile, "logs", "--no-color"}, besuServices...)
	args = append(args, besuSpareService)
	logs, err := c.runCompose(ctx, args...)
	if len(logs) > 0 {
		fmt.Print(string(logs))
	}
	return err
}

// AddValidator starts the node run by service if it is not running, votes it into the validator set, and returns the
// first observed height whose validator set includes it.
func (c BesuChain) AddValidator(ctx context.Context, service string) (uint64, error) {
	if _, err := c.runCompose(ctx, "up", "--detach", service); err != nil {
		return 0, fmt.Errorf("start %s: %w", service, err)
	}
	return c.voteValidator(ctx, service, true)
}

// RemoveValidator votes the validator run by service out of the validator set, and returns the first observed height
// whose validator set excludes it. The node keeps running as a non-validator.
func (c BesuChain) RemoveValidator(ctx context.Context, service string) (uint64, error) {
	return c.voteValidator(ctx, service, false)
}

// voteValidator has every current validator vote to add or remove the validator run by service, and waits until the
// change applies. The votes are discarded afterwards, also on failure, so that later changes start from a clean slate.
func (c BesuChain) voteValidator(ctx context.Context, service string, add bool) (height uint64, err error) {
	validator, err := besuValidatorAddress(service)
	if err != nil {
		return 0, err
	}

	client, err := ethclient.DialContext(ctx, c.RPC)
	if err != nil {
		return 0, fmt.Errorf("dial rpc: %w", err)
	}
	defer client.Close()

	validatorsAt := func(height string) ([]common.Address, error) {
		var validators []common.Address
		err := client.Client().CallContext(ctx, &validators, c.rpcPrefix+"_getValidatorsByBlockNumber", height)
		return validators, err
	}

	validators, err := validatorsAt("latest")
	if err != nil {
		return 0, fmt.Errorf("get validators: %w", err)
	}
	// Only validators propose blocks, so only their votes count.
	var voters []string
	for _, voter := range append(slices.Clone(besuServices), besuSpareService) {
		address, err := besuValidatorAddress(voter)
		if err != nil {
			return 0, err
		}
		if slices.Contains(validators, address) {
			voters = append(voters, voter)
		}
	}

	defer func() {
		for _, voter := range voters {
			if discardErr := c.validatorRPC(ctx, voter, c.rpcPrefix+"_discardValidatorVote", validator); discardErr != nil {
				err = errors.Join(err, discardErr)
			}
		}
	}()
	for _, voter := range voters {
		if err := c.validatorRPC(ctx, voter, c.rpcPrefix+"_proposeValidatorVote", validator, add); err != nil {
			return 0, err
		}
	}

	err = testutil.WaitForCondition(2*time.Minute, time.Second, func() (bool, error) {
		latest, err := client.BlockNumber(ctx)
		if err != nil {
			return false, err
		}
		validators, err := validatorsAt(hexutil.EncodeUint64(latest))
		if err != nil {
			return false, err
		}
		height = latest
		return slices.Contains(validators, validator) == add, nil
	})
	if err != nil {
		return 0, fmt.Errorf("wait for validator set change: %w", err)
	}

	return height, nil
}

func besuValidatorAddress(service string) (common.Address, error) {
	keyHex, err := besuAssets.ReadFile(fmt.Sprintf("testdata/besu/keys/%s/key", service))
	if err != nil {
		return common.Address{}, fmt.Errorf("read %s key: %w", service, err)
	}
	key, err := crypto.HexToECDSA(strings.TrimPrefix(strings.TrimSpace(string(keyHex)), "0x"))
	if err != nil {
		return common.Address{}, fmt.Errorf("parse %s key: %w", service, err)
	}
	return crypto.PubkeyToAddress(key.PublicKey), nil
}

// validatorRPC calls method on the RPC endpoint of service from inside its container, since only validator1 publishes
// its RPC port. The image ships without curl, so the request goes through bash's /dev/tcp.
func (c BesuChain) validatorRPC(ctx context.Context, service, method string, params ...any) error {
	body, err := json.Marshal(map[string]any{"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
	if err != nil {
		return err
	}
	const script = `exec 3<>/dev/tcp/127.0.0.1/8545 && ` +
		`printf 'POST / HTTP/1.0\r\nContent-Type: application/json\r\nContent-Length: %d\r\n\r\n%s' "${#1}" "$1" >&3 && ` +
		`cat <&3`
	output, err := c.runCompose(ctx, "exec", "-T", service, "bash", "-c", script, "bash", string(body))
	if err != nil {
		return fmt.Errorf("%s on %s: %w", method, service, err)
	}

	_, respBody, ok := strings.Cut(string(output), "\r\n\r\n")
	if !ok {
		return fmt.Errorf("%s on %s: malformed http response %q", method, service, output)
	}
	var resp struct {
		Error *struct {
			Message string `json:"message"`
		} `json:"error"`
	}
	if err := json.Unmarshal([]byte(respBody), &resp); err != nil {
		return fmt.Errorf("%s on %s: decode response %q: %w", method, service, respBody, err)
	}
	if resp.Error != nil {
		return fmt.Errorf("%s on %s: %s", method, service, resp.Error.Message)
	}
	return nil
}

func (c BesuChain) runCompose(ctx context.Context, args ...string) ([]byte, error) {
	composeArgs := []string{
		"compose",
		"--project-name", c.projectName,
		"--file", filepath.Join(c.projectDir, besuComposeFile),
	}
	composeArgs = append(composeArgs, args...)

	output, err := exec.CommandContext(ctx, "docker", composeArgs...).CombinedOutput()
	if err != nil {
		return output, fmt.Errorf("docker compose %s: %w: %s", strings.Join(args, " "), err, strings.TrimSpace(string(output)))
	}
	return output, nil
}

func materializeBesuAssets(dst string) error {
	sub, err := fs.Sub(besuAssets, "testdata/besu")
	if err != nil {
		return err
	}

	return os.CopyFS(dst, sub)
}

// patchBesuGenesis sets the chain ID and, for IBFT2, converts the embedded QBFT genesis to IBFT2.
func patchBesuGenesis(path string, chainID uint64, consensus string) error {
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

	if consensus == testvalues.BesuConsensusIBFT2 {
		config["ibft2"] = config["qbft"]
		delete(config, "qbft")

		extraData, ok := genesis["extraData"].(string)
		if !ok {
			return fmt.Errorf("genesis extraData missing or invalid")
		}
		genesis["extraData"], err = ibft2GenesisExtraData(extraData)
		if err != nil {
			return err
		}
	}

	updated, err := json.MarshalIndent(genesis, "", "  ")
	if err != nil {
		return err
	}
	updated = append(updated, '\n')

	// Besu reads the generated genesis file as an unprivileged container user.
	return os.WriteFile(path, updated, 0o644) //nolint:gosec
}

// ibft2GenesisExtraData re-encodes QBFT genesis extraData, RLP([vanity, validators, vote, round, seals]), with the
// IBFT2 encoding of an absent vote (empty string instead of empty list) and of round 0 (4 bytes instead of an integer).
func ibft2GenesisExtraData(qbftExtraData string) (string, error) {
	extraData, err := hexutil.Decode(qbftExtraData)
	if err != nil {
		return "", err
	}
	var items []rlp.RawValue
	if err := rlp.DecodeBytes(extraData, &items); err != nil {
		return "", err
	}
	if len(items) != 5 {
		return "", fmt.Errorf("expected 5 extraData items, got %d", len(items))
	}
	items[2] = rlp.RawValue{0x80}
	items[3] = rlp.RawValue{0x84, 0, 0, 0, 0}
	extraData, err = rlp.EncodeToBytes(items)
	if err != nil {
		return "", err
	}
	return hexutil.Encode(extraData), nil
}

// patchBesuConfig enables the RPC namespace of the consensus in place of QBFT's.
func patchBesuConfig(path, rpcPrefix string) error {
	contents, err := os.ReadFile(path)
	if err != nil {
		return err
	}
	updated := strings.Replace(string(contents), `"QBFT"`, fmt.Sprintf("%q", strings.ToUpper(rpcPrefix)), 1)
	// Besu reads the generated config file as an unprivileged container user.
	return os.WriteFile(path, []byte(updated), 0o644) //nolint:gosec
}

// patchBesuTopology rewrites the default subnet and validator IPs in the compose file and static node lists.
func patchBesuTopology(projectDir string, params BesuParams) error {
	replacer := strings.NewReplacer(
		defaultBesuSubnet, params.Subnet,
		defaultBesuGateway, params.Gateway,
		defaultBesuValidatorIPs[0], params.ValidatorIPs[0],
		defaultBesuValidatorIPs[1], params.ValidatorIPs[1],
		defaultBesuValidatorIPs[2], params.ValidatorIPs[2],
		defaultBesuValidatorIPs[3], params.ValidatorIPs[3],
		defaultBesuValidatorIPs[4], params.ValidatorIPs[4],
	)

	paths := []string{filepath.Join(projectDir, besuComposeFile)}
	for _, service := range append(slices.Clone(besuServices), besuSpareService) {
		paths = append(paths, filepath.Join(projectDir, "static-nodes", service+".json"))
	}
	for _, path := range paths {
		contents, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		// Docker Compose and Besu read these generated files outside the Go process.
		if err := os.WriteFile(path, []byte(replacer.Replace(string(contents))), 0o644); err != nil { //nolint:gosec
			return err
		}
	}
	return nil
}

func waitForBesuReady(ctx context.Context, rpcURL, rpcPrefix string) error {
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
		if peerCount < uint64(len(besuServices)-1) {
			lastErr = fmt.Errorf("peer count %d, want at least %d", peerCount, len(besuServices)-1)
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
		if err := client.Client().CallContext(ctx, &validators, rpcPrefix+"_getValidatorsByBlockNumber", "latest"); err != nil {
			lastErr = err
			return false, nil
		}

		if len(validators) != len(besuServices) {
			lastErr = fmt.Errorf("validator count %d, want exactly %d", len(validators), len(besuServices))
			return false, nil
		}

		return true, nil
	})
	if err != nil && lastErr != nil {
		return fmt.Errorf("%w (last readiness observation: %v)", err, lastErr)
	}
	return err
}

func waitForBesuTransactionHandling(ctx context.Context, rpcURL string, key *ecdsa.PrivateKey) error {
	client, err := ethclient.DialContext(ctx, rpcURL)
	if err != nil {
		return fmt.Errorf("dial transaction probe rpc: %w", err)
	}
	defer client.Close()

	var lastErr error
	err = testutil.WaitForCondition(2*time.Minute, 2*time.Second, func() (bool, error) {
		txHash, err := sendBesuProbeTx(ctx, client, key)
		if err != nil {
			lastErr = err
			return false, nil
		}

		receiptCtx, cancel := context.WithTimeout(ctx, besuTxProbeReceiptTimeout)
		receipt, err := bind.WaitMinedHash(receiptCtx, client, txHash)
		cancel()
		if err != nil {
			lastErr = err
			return false, nil
		}
		if receipt.Status != ethtypes.ReceiptStatusSuccessful {
			return false, fmt.Errorf("besu transaction probe failed on-chain with status %d", receipt.Status)
		}

		return true, nil
	})
	if err != nil && lastErr != nil {
		return fmt.Errorf("%w (last transaction probe error: %v)", err, lastErr)
	}
	return err
}

func sendBesuProbeTx(ctx context.Context, client *ethclient.Client, key *ecdsa.PrivateKey) (common.Hash, error) {
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
