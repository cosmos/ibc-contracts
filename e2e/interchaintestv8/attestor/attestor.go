// SPDX-License-Identifier: Apache-2.0

package attestor

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	dockerclient "github.com/moby/moby/client"
	grpc "google.golang.org/grpc"
	insecure "google.golang.org/grpc/credentials/insecure"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/dockerutil"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	attestortypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/attestor"
)

// ChainType represents the type of blockchain adapter
type ChainType string

const (
	ChainTypeEvm    ChainType = testvalues.Attestor_ChainType_EVM
	ChainTypeCosmos ChainType = testvalues.Attestor_ChainType_Cosmos
	ChainTypeSolana ChainType = testvalues.Attestor_ChainType_Solana

	// DefaultAttestorImage is the default Docker image for the attestor.
	DefaultAttestorImage = "ghcr.io/cosmos/ibc-attestor:v0.3.0"
	// EnvKeyAttestorImage is the environment variable to override the Docker image.
	EnvKeyAttestorImage = "IBC_ATTESTOR_IMAGE"
)

// GetAttestorImage returns the Docker image to use for attestor containers.
func GetAttestorImage() string {
	if img := os.Getenv(EnvKeyAttestorImage); img != "" {
		return img
	}
	return DefaultAttestorImage
}

// KeystorePath returns the keystore path for a given attestor index.
func KeystorePath(index int) string {
	return fmt.Sprintf(testvalues.AttestorKeystorePathTemplate, index)
}

// GetAttestationServiceClient returns an AttestationServiceClient for the attestor.
func GetAttestationServiceClient(addr string) (attestortypes.AttestationServiceClient, error) {
	// Strip http:// or https:// prefix if present, as Go gRPC clients don't accept them
	addr = e2esuite.StripHTTPPrefix(addr)

	conn, err := grpc.NewClient(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, err
	}

	return attestortypes.NewAttestationServiceClient(conn), nil
}

// CheckAttestorHealth performs a health check on an attestor by making a state attestation request.
// It properly manages the gRPC connection lifecycle, closing it after the check completes.
func CheckAttestorHealth(ctx context.Context, addr string) error {
	cleanAddr := e2esuite.StripHTTPPrefix(addr)

	conn, err := grpc.NewClient(cleanAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		fmt.Printf("[attestor] Health check failed to create gRPC connection to %s: %v\n", cleanAddr, err)
		return err
	}
	defer conn.Close()

	client := attestortypes.NewAttestationServiceClient(conn)
	_, err = client.StateAttestation(ctx, &attestortypes.StateAttestationRequest{Height: 1})
	if err != nil {
		fmt.Printf("[attestor] Health check StateAttestation failed for %s: %v\n", cleanAddr, err)
		return err
	}

	fmt.Printf("[attestor] Health check passed for %s\n", cleanAddr)
	return nil
}

// GetStateAttestation is a simple wrapper for the StateAttestation gRPC call.
func GetStateAttestation(ctx context.Context, client attestortypes.AttestationServiceClient, height uint64) (*attestortypes.StateAttestationResponse, error) {
	req := &attestortypes.StateAttestationRequest{
		Height: height,
	}
	return client.StateAttestation(ctx, req)
}

// GetPacketAttestation is a simple wrapper for the PacketAttestation gRPC call.
func GetPacketAttestation(ctx context.Context, client attestortypes.AttestationServiceClient, packets [][]byte, height uint64, commitmentType ...attestortypes.CommitmentType) (*attestortypes.PacketAttestationResponse, error) {
	req := &attestortypes.PacketAttestationRequest{
		Packets: packets,
		Height:  height,
	}
	// Optional commitment type parameter
	if len(commitmentType) > 0 {
		req.CommitmentType = commitmentType[0]
	}
	return client.PacketAttestation(ctx, req)
}

const (
	// attestorContainerPort is the gRPC port exposed by the attestor container.
	attestorContainerPort = "2025"
	// attestorConfigPath is the path where the config file is mounted in the container.
	attestorConfigPath = "/config/attestor.toml"
	// attestorKeystorePath is the path where the keystore is mounted in the container.
	attestorKeystorePath = "/keystore"
	// defaultAttestorHealthCheckTimeout is the default timeout for health checks.
	defaultAttestorHealthCheckTimeout = 60 * time.Second
)

// AttestorContainer represents a Docker-based attestor instance.
type AttestorContainer struct {
	Container      *dockerutil.Container
	Endpoint       string // Host address for gRPC connection (e.g., "http://127.0.0.1:32768")
	DockerEndpoint string // Docker network address for gRPC connection (e.g., "http://container-name:2025")
	Address        string // Ethereum address of the attestor
	KeystorePath   string // Host path to the keystore
	ConfigDir      string // Host path to config directory (for cleanup)
}

// StartAttestorDockerParams holds parameters for starting an attestor in Docker.
type StartAttestorDockerParams struct {
	Client           *dockerclient.Client
	NetworkID        string
	Config           *AttestorConfig
	ChainType        ChainType
	KeystorePath     string // Host path to the keystore directory
	Index            int    // Index for multiple attestors
	EnableHostAccess bool   // Enable access to host machine via host.docker.internal
}

// GenerateAttestorKeyDocker generates a new attestor key using a Docker container.
func GenerateAttestorKeyDocker(ctx context.Context, client *dockerclient.Client, keystoreHostPath string) error {
	// Ensure the keystore directory exists
	if err := os.MkdirAll(keystoreHostPath, 0o755); err != nil {
		return fmt.Errorf("failed to create keystore directory: %w", err)
	}
	// Explicitly chmod to 777 so the container user (UID 1000) can write to it
	// MkdirAll is affected by umask, so we need to chmod explicitly
	if err := os.Chmod(keystoreHostPath, 0o777); err != nil {
		return fmt.Errorf("failed to chmod keystore directory: %w", err)
	}

	containerName := fmt.Sprintf("attestor-keygen-%d", time.Now().UnixNano())

	// Run a one-shot container to generate the key
	container, err := dockerutil.CreateAndStart(ctx, client, dockerutil.ContainerConfig{
		Name:  containerName,
		Image: GetAttestorImage(),
		Cmd:   []string{"key", "generate", "--keystore", attestorKeystorePath},
		Mounts: []dockerutil.Mount{
			{
				HostPath:      keystoreHostPath,
				ContainerPath: attestorKeystorePath,
				ReadOnly:      false,
			},
		},
	})
	if err != nil {
		// Check if key already exists (not an error)
		if strings.Contains(err.Error(), "already found") {
			return nil
		}
		return fmt.Errorf("failed to create keygen container: %w", err)
	}

	// Wait for container to exit (it's a one-shot command)
	deadline := time.Now().Add(30 * time.Second)
	for time.Now().Before(deadline) {
		inspect, err := client.ContainerInspect(ctx, container.ID)
		if err != nil {
			container.Remove(ctx)
			return fmt.Errorf("failed to inspect keygen container: %w", err)
		}
		if !inspect.State.Running {
			// Container has exited
			if inspect.State.ExitCode != 0 {
				logs, _ := container.Logs(ctx)
				// Check if it's a "key already exists" error
				if strings.Contains(logs, "already found") {
					container.Remove(ctx)
					return nil
				}
				container.Remove(ctx)
				return fmt.Errorf("keygen container exited with code %d: %s", inspect.State.ExitCode, logs)
			}
			container.Remove(ctx)
			return nil
		}
		time.Sleep(500 * time.Millisecond)
	}

	container.Remove(ctx)
	return fmt.Errorf("keygen container timed out")
}

// ReadAttestorAddressDocker reads the attestor's public address using a Docker container.
func ReadAttestorAddressDocker(ctx context.Context, client *dockerclient.Client, keystoreHostPath string) (string, error) {
	containerName := fmt.Sprintf("attestor-keyshow-%d", time.Now().UnixNano())

	fmt.Printf("[attestor] Creating keyshow container %s...\n", containerName)

	// Run a one-shot container to show the key
	container, err := dockerutil.CreateAndStart(ctx, client, dockerutil.ContainerConfig{
		Name:  containerName,
		Image: GetAttestorImage(),
		Cmd:   []string{"key", "show", "--keystore", attestorKeystorePath},
		Mounts: []dockerutil.Mount{
			{
				HostPath:      keystoreHostPath,
				ContainerPath: attestorKeystorePath,
				ReadOnly:      true,
			},
		},
	})
	if err != nil {
		return "", fmt.Errorf("failed to create keyshow container: %w", err)
	}
	defer container.Remove(ctx)

	fmt.Printf("[attestor] Keyshow container created, waiting for exit...\n")

	// Wait for container to exit
	deadline := time.Now().Add(30 * time.Second)
	iteration := 0
	for time.Now().Before(deadline) {
		inspect, err := client.ContainerInspect(ctx, container.ID)
		if err != nil {
			return "", fmt.Errorf("failed to inspect keyshow container: %w", err)
		}
		if !inspect.State.Running {
			fmt.Printf("[attestor] Keyshow container exited with code %d\n", inspect.State.ExitCode)
			// Container has exited
			if inspect.State.ExitCode != 0 {
				logs, _ := container.Logs(ctx)
				return "", fmt.Errorf("keyshow container exited with code %d: %s", inspect.State.ExitCode, logs)
			}

			// Get the output
			logs, err := container.Logs(ctx)
			if err != nil {
				return "", fmt.Errorf("failed to get keyshow output: %w", err)
			}
			fmt.Printf("[attestor] Keyshow container raw output: %q\n", logs)

			// Extract 40-char hex address from output (handles various formats like "(addr" or "0xaddr")
			hexPattern := regexp.MustCompile(`[0-9a-fA-F]{40}`)
			match := hexPattern.FindString(logs)
			if match == "" {
				return "", fmt.Errorf("no valid ethereum address found in output: %s", logs)
			}
			address := "0x" + strings.ToLower(match)
			fmt.Printf("[attestor] Keyshow container returned address: %s\n", address)
			return address, nil
		}
		iteration++
		if iteration%10 == 0 {
			fmt.Printf("[attestor] Still waiting for keyshow container to exit (iteration %d)...\n", iteration)
		}
		time.Sleep(500 * time.Millisecond)
	}

	fmt.Printf("[attestor] Keyshow container timed out after 30s\n")
	return "", fmt.Errorf("keyshow container timed out")
}

// GenerateAttestorKeysParams holds parameters for generating multiple attestor keys.
type GenerateAttestorKeysParams struct {
	Client               *dockerclient.Client
	NumKeys              int
	KeystorePathTemplate string // Template with %d for index, e.g., "/tmp/attestor-keystore-%d"
}

// GenerateAttestorKeys generates multiple attestor keys and returns their addresses.
// This is useful when you need to generate keys for attestors without starting them
// (e.g., for light client registration).
func GenerateAttestorKeys(ctx context.Context, params GenerateAttestorKeysParams) ([]string, error) {
	addresses := make([]string, params.NumKeys)

	for i := 0; i < params.NumKeys; i++ {
		keystorePath := fmt.Sprintf(params.KeystorePathTemplate, i)

		// Generate key
		if err := GenerateAttestorKeyDocker(ctx, params.Client, keystorePath); err != nil {
			return nil, fmt.Errorf("failed to generate key %d: %w", i, err)
		}

		// Read address
		address, err := ReadAttestorAddressDocker(ctx, params.Client, keystorePath)
		if err != nil {
			return nil, fmt.Errorf("failed to read address %d: %w", i, err)
		}

		addresses[i] = address
	}

	return addresses, nil
}

// StartAttestorDocker starts the attestor as a Docker container.
func StartAttestorDocker(ctx context.Context, params StartAttestorDockerParams) (*AttestorContainer, error) {
	fmt.Printf("[attestor] Starting attestor Docker container (index=%d, chainType=%s, enableHostAccess=%v)\n",
		params.Index, params.ChainType, params.EnableHostAccess)
	fmt.Printf("[attestor] Adapter URL: %s, Router Address: %s\n", params.Config.Adapter.URL, params.Config.Adapter.RouterAddress)

	// Create a temporary directory for the config file
	configDir, err := os.MkdirTemp("", "attestor-config-*")
	if err != nil {
		return nil, fmt.Errorf("failed to create config directory: %w", err)
	}
	// Chmod config dir so container user (UID 1000) can read it
	if err := os.Chmod(configDir, 0o755); err != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to chmod config directory: %w", err)
	}

	// Ensure keystore directory exists and generate key if needed
	if err := os.MkdirAll(params.KeystorePath, 0o755); err != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to create keystore directory: %w", err)
	}
	// Explicitly chmod to 777 so the container user (UID 1000) can write to it
	if err := os.Chmod(params.KeystorePath, 0o777); err != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to chmod keystore directory: %w", err)
	}

	fmt.Printf("[attestor] Generating attestor key...\n")
	if err := GenerateAttestorKeyDocker(ctx, params.Client, params.KeystorePath); err != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to generate attestor key: %w", err)
	}
	fmt.Printf("[attestor] Key generated successfully\n")

	// Read the attestor address
	fmt.Printf("[attestor] Reading attestor address...\n")
	address, err := ReadAttestorAddressDocker(ctx, params.Client, params.KeystorePath)
	if err != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to read attestor address: %w", err)
	}
	fmt.Printf("[attestor] Address read: %s\n", address)

	// Update config to use container paths
	configCopy := *params.Config
	configCopy.Signer.KeystorePath = attestorKeystorePath

	// Write config to temporary file
	fmt.Printf("[attestor] Writing config file...\n")
	configFilePath := filepath.Join(configDir, "attestor.toml")
	if err := configCopy.WriteTomlConfig(configFilePath); err != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to write config file: %w", err)
	}
	// Chmod config file so container user (UID 1000) can read it
	if err := os.Chmod(configFilePath, 0o644); err != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to chmod config file: %w", err)
	}
	fmt.Printf("[attestor] Config file written\n")

	// Log the config for debugging
	configData, _ := os.ReadFile(configFilePath)
	fmt.Printf("[attestor] Config file contents (%d bytes):\n%s\n", len(configData), configData)

	// Check context before proceeding
	if ctx.Err() != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("context canceled before container creation: %w", ctx.Err())
	}

	// Generate unique container name
	containerName := dockerutil.GenerateContainerName()
	fmt.Printf("[attestor] Generated container name: %s\n", containerName)

	// Build extra hosts for host access (e.g., for Solana localnet running on host)
	var extraHosts []string
	if params.EnableHostAccess {
		extraHosts = []string{"host.docker.internal:host-gateway"}
	}

	fmt.Printf("[attestor] About to call CreateAndStart for attestor server container (networkID=%s)\n", params.NetworkID)

	// Create and start container
	container, err := dockerutil.CreateAndStart(ctx, params.Client, dockerutil.ContainerConfig{
		Name:      containerName,
		Image:     GetAttestorImage(),
		Cmd:       []string{"server", "--config", attestorConfigPath, "--chain-type", string(params.ChainType)},
		NetworkID: params.NetworkID,
		Mounts: []dockerutil.Mount{
			{
				HostPath:      configDir,
				ContainerPath: "/config",
				ReadOnly:      true,
			},
			{
				HostPath:      params.KeystorePath,
				ContainerPath: attestorKeystorePath,
				ReadOnly:      true,
			},
		},
		ExposedPorts: []string{attestorContainerPort},
		Env: []string{
			"RUST_LOG=info",
			"RUST_BACKTRACE=1",
		},
		ExtraHosts: extraHosts,
	})
	if err != nil {
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to create attestor container: %w", err)
	}

	// Get the host address for gRPC
	grpcAddr := container.GetHostAddress(attestorContainerPort)
	if grpcAddr == "" {
		container.StopAndRemove(ctx)
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("failed to get host port for attestor gRPC")
	}

	fmt.Printf("[attestor] Container %s created, host gRPC address: %s\n", containerName, grpcAddr)
	fmt.Printf("[attestor] Starting health check (timeout=%v)...\n", defaultAttestorHealthCheckTimeout)

	// Wait for the attestor to be healthy
	// Using CheckAttestorHealth which properly closes the gRPC connection after each check
	healthCheckStart := time.Now()
	healthCheck := func() error {
		return CheckAttestorHealth(ctx, grpcAddr)
	}

	if err := container.WaitForHealth(ctx, defaultAttestorHealthCheckTimeout, healthCheck); err != nil {
		logs, _ := container.Logs(ctx)
		container.StopAndRemove(ctx)
		os.RemoveAll(configDir)
		return nil, fmt.Errorf("attestor health check failed after %v: %w\nContainer logs:\n%s", time.Since(healthCheckStart), err, logs)
	}

	fmt.Printf("[attestor] Health check passed after %v\n", time.Since(healthCheckStart))

	// Log network info for debugging Docker network connectivity
	dockerEndpoint := fmt.Sprintf("http://%s:%s", containerName, attestorContainerPort)
	fmt.Printf("Attestor container started successfully:\n")
	fmt.Printf("  Container name: %s\n", containerName)
	fmt.Printf("  Host endpoint: http://%s\n", grpcAddr)
	fmt.Printf("  Docker endpoint: %s\n", dockerEndpoint)
	fmt.Printf("  Network ID: %s\n", params.NetworkID)

	if networkInfo, err := container.GetNetworkInfo(ctx); err == nil {
		fmt.Printf("%s\n", networkInfo)
	}

	return &AttestorContainer{
		Container:      container,
		Endpoint:       fmt.Sprintf("http://%s", grpcAddr),
		DockerEndpoint: dockerEndpoint,
		Address:        address,
		KeystorePath:   params.KeystorePath,
		ConfigDir:      configDir,
	}, nil
}

// Cleanup stops and removes the container and cleans up config files.
func (a *AttestorContainer) Cleanup(ctx context.Context) {
	a.Container.StopAndRemove(ctx)
	os.RemoveAll(a.ConfigDir)
}

// GetStatusAndLogs returns the container status and recent logs for debugging.
func (a *AttestorContainer) GetStatusAndLogs(ctx context.Context) (string, error) {
	running, err := a.Container.IsRunning(ctx)
	if err != nil {
		return "", fmt.Errorf("failed to check container status: %w", err)
	}

	logs, err := a.Container.Logs(ctx)
	if err != nil {
		return "", fmt.Errorf("failed to get container logs: %w", err)
	}

	status := "stopped"
	if running {
		status = "running"
	}

	return fmt.Sprintf("Container %s status: %s\nEndpoint: %s\nRecent logs:\n%s",
		a.Container.Name, status, a.Endpoint, logs), nil
}
