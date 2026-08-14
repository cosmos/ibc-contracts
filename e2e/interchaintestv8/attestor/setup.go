// SPDX-License-Identifier: Apache-2.0

package attestor

import (
	"context"
	"errors"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"

	dockerclient "github.com/moby/moby/client"
	"github.com/stretchr/testify/require"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

// TransformLocalhostToDockerHost transforms localhost URLs to use host.docker.internal
// so that Docker containers can reach services running on the host machine.
func TransformLocalhostToDockerHost(url string) string {
	url = strings.ReplaceAll(url, "localhost", "host.docker.internal")
	url = strings.ReplaceAll(url, "127.0.0.1", "host.docker.internal")
	return url
}

// SetupParams contains parameters for setting up attestors.
type SetupParams struct {
	// Number of attestor instances to start
	NumAttestors int
	// Keystore path template (e.g., "/tmp/attestor_keystore_%d")
	KeystorePathTemplate string
	// Chain type for the attestor
	ChainType ChainType
	// Adapter URL (RPC endpoint)
	AdapterURL string
	// Router address (for EVM/Solana, empty for Cosmos)
	RouterAddress string
	// Docker client
	DockerClient *dockerclient.Client
	// Docker network ID
	NetworkID string
	// Enable access to host machine via host.docker.internal (for services running on host)
	EnableHostAccess bool
}

// SetupResult contains the result of setting up attestors.
type SetupResult struct {
	// Attestor containers that were started
	Containers []*AttestorContainer
	// Attestor endpoints (e.g., "http://127.0.0.1:32768") - for host-side connections
	Endpoints []string
	// Docker endpoints (e.g., "http://container-name:2025") - for Docker network connections
	DockerEndpoints []string
	// Attestor addresses (Ethereum addresses)
	Addresses []string
	// Keystore paths used for each attestor
	KeystorePaths []string
}

// attestorInstance holds data for a single attestor setup
type attestorInstance struct {
	index     int
	container *AttestorContainer
	err       error
}

// SetupAttestors starts multiple attestor containers in parallel and returns their endpoints.
func SetupAttestors(ctx context.Context, t *testing.T, params SetupParams) SetupResult {
	t.Helper()
	require.NotEmpty(t, params.KeystorePathTemplate, "KeystorePathTemplate is required")
	require.NotNil(t, params.DockerClient, "DockerClient is required")

	t.Logf("[attestor-setup] Setting up %d attestor(s) for chain type %s", params.NumAttestors, params.ChainType)
	t.Logf("[attestor-setup] Adapter URL: %s, Router Address: %s", params.AdapterURL, params.RouterAddress)
	t.Logf("[attestor-setup] Network ID: %s, EnableHostAccess: %v", params.NetworkID, params.EnableHostAccess)

	setupStart := time.Now()

	// Start all attestors in parallel and collect results via channel
	resultCh := make(chan attestorInstance, params.NumAttestors)

	for i := range params.NumAttestors {
		go func(index int) {
			// Recover from any panic to ensure we always send to the channel
			defer func() {
				if r := recover(); r != nil {
					resultCh <- attestorInstance{index: index, err: fmt.Errorf("attestor %d panicked: %v", index, r)}
				}
			}()

			fmt.Printf("[attestor-setup] Goroutine %d starting\n", index)
			keystorePath := fmt.Sprintf(params.KeystorePathTemplate, index)

			// Create config
			cfg := DefaultAttestorConfig()
			cfg.Adapter.URL = params.AdapterURL
			cfg.Adapter.RouterAddress = params.RouterAddress
			cfg.Adapter.FinalityOffset = 0
			// Note: Server.ListenAddr uses the default container port (2025)
			// The actual exposed port is dynamically assigned by Docker

			fmt.Printf("[attestor-setup] Goroutine %d calling StartAttestorDocker\n", index)
			container, err := StartAttestorDocker(ctx, StartAttestorDockerParams{
				Client:           params.DockerClient,
				NetworkID:        params.NetworkID,
				Config:           cfg,
				ChainType:        params.ChainType,
				KeystorePath:     keystorePath,
				Index:            index,
				EnableHostAccess: params.EnableHostAccess,
			})
			if err != nil {
				fmt.Printf("[attestor-setup] Goroutine %d StartAttestorDocker failed: %v\n", index, err)
				resultCh <- attestorInstance{index: index, err: fmt.Errorf("attestor %d failed to start: %w", index, err)}
				return
			}

			fmt.Printf("[attestor-setup] Goroutine %d StartAttestorDocker succeeded\n", index)
			resultCh <- attestorInstance{
				index:     index,
				container: container,
			}
		}(i)
	}

	// Collect results
	instances := make([]*AttestorContainer, params.NumAttestors)
	var errs []error
	for range params.NumAttestors {
		res := <-resultCh
		if res.err != nil {
			errs = append(errs, res.err)
		} else {
			instances[res.index] = res.container
		}
	}

	// If any errors, clean up started containers
	if len(errs) > 0 {
		for _, container := range instances {
			if container != nil {
				container.Cleanup(ctx)
			}
		}
		require.NoError(t, errors.Join(errs...))
	}

	// Build result in order and register cleanup
	var result SetupResult
	for i, container := range instances {
		result.Containers = append(result.Containers, container)
		result.Endpoints = append(result.Endpoints, container.Endpoint)
		result.DockerEndpoints = append(result.DockerEndpoints, container.DockerEndpoint)
		result.Addresses = append(result.Addresses, container.Address)
		result.KeystorePaths = append(result.KeystorePaths, container.KeystorePath)
		t.Logf("%s Attestor %d address: %s, endpoint: %s, docker: %s", params.ChainType, i, container.Address, container.Endpoint, container.DockerEndpoint)

		// Register cleanup for this container
		t.Cleanup(func() {
			container.Cleanup(context.Background())
			_ = os.RemoveAll(container.KeystorePath)
		})
	}

	// Wait for attestor containers to fully stabilize.
	// This gives Docker port forwarding time to become fully reliable for the relayer.
	t.Log("[attestor-setup] Waiting 5 seconds for attestor containers to fully stabilize...")
	time.Sleep(5 * time.Second)

	// Verify all attestors are still running and healthy after stabilization
	t.Log("[attestor-setup] Verifying attestor containers are still running...")
	for i, container := range result.Containers {
		statusAndLogs, err := container.GetStatusAndLogs(ctx)
		if err != nil {
			t.Logf("[attestor-setup] WARNING: Failed to get status for attestor %d: %v", i, err)
			continue
		}
		t.Logf("[attestor-setup] Attestor %d status:\n%s", i, statusAndLogs)

		// Verify the attestor is still responding to gRPC calls
		if err := CheckAttestorHealth(ctx, result.Endpoints[i]); err != nil {
			t.Logf("[attestor-setup] ERROR: Attestor %d at %s health check failed after stabilization: %v", i, result.Endpoints[i], err)

			// Try one more time after a brief delay
			time.Sleep(500 * time.Millisecond)
			if err := CheckAttestorHealth(ctx, result.Endpoints[i]); err != nil {
				t.Fatalf("[attestor-setup] Attestor %d at %s is not healthy after retry: %v", i, result.Endpoints[i], err)
			}

			t.Logf("[attestor-setup] Attestor %d recovered after retry", i)
		} else {
			t.Logf("[attestor-setup] Attestor %d health check passed after stabilization", i)
		}
	}

	t.Logf("[attestor-setup] All %d attestor(s) setup completed and verified in %v", params.NumAttestors, time.Since(setupStart))
	return result
}

// SetupEthAttestors sets up EVM attestors for Eth→Cosmos direction.
// chainType should be ChainTypeEvm for PoW chains or ChainTypeCosmos for PoS chains.
func SetupEthAttestors(ctx context.Context, t *testing.T, client *dockerclient.Client, networkID, ethRPC, ics26Address string, chainType ChainType) SetupResult {
	t.Helper()
	return SetupAttestors(ctx, t, SetupParams{
		NumAttestors:         testvalues.NumAttestors,
		KeystorePathTemplate: testvalues.AttestorKeystorePathTemplate,
		ChainType:            chainType,
		AdapterURL:           ethRPC,
		RouterAddress:        ics26Address,
		DockerClient:         client,
		NetworkID:            networkID,
	})
}

// SetupCosmosAttestors sets up Cosmos attestors for Cosmos→Eth direction.
func SetupCosmosAttestors(ctx context.Context, t *testing.T, client *dockerclient.Client, networkID, tmRPC string) SetupResult {
	t.Helper()
	return SetupAttestors(ctx, t, SetupParams{
		NumAttestors:         testvalues.NumAttestors,
		KeystorePathTemplate: testvalues.AttestorKeystorePathTemplate,
		ChainType:            ChainTypeCosmos,
		AdapterURL:           tmRPC,
		RouterAddress:        "", // Cosmos doesn't use router address
		DockerClient:         client,
		NetworkID:            networkID,
	})
}

// SetupSolanaAttestors sets up Solana attestors for Solana→Cosmos direction.
// Since Solana localnet runs on the host machine (not in Docker), we enable host access
// and transform localhost URLs to host.docker.internal so the attestor container can reach it.
func SetupSolanaAttestors(ctx context.Context, t *testing.T, client *dockerclient.Client, networkID, solanaRPC, ics26RouterProgramID string) SetupResult {
	t.Helper()

	// Transform localhost URLs to host.docker.internal for Docker container access
	dockerSolanaRPC := TransformLocalhostToDockerHost(solanaRPC)

	return SetupAttestors(ctx, t, SetupParams{
		NumAttestors:         testvalues.NumAttestors,
		KeystorePathTemplate: testvalues.AttestorKeystorePathTemplate,
		ChainType:            ChainTypeSolana,
		AdapterURL:           dockerSolanaRPC,
		RouterAddress:        ics26RouterProgramID,
		DockerClient:         client,
		NetworkID:            networkID,
		EnableHostAccess:     true, // Solana localnet runs on host, need host access
	})
}

// CleanupContainers stops and removes all attestor containers.
func CleanupContainers(ctx context.Context, t *testing.T, containers []*AttestorContainer) {
	t.Helper()
	for _, c := range containers {
		if c != nil {
			c.Cleanup(ctx)
		}
	}
}
