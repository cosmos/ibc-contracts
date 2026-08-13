// SPDX-License-Identifier: Apache-2.0

package dockerutil

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"io"
	"log"
	"strings"
	"time"

	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/mount"
	"github.com/docker/docker/api/types/network"
	"github.com/docker/go-connections/nat"
	dockerclient "github.com/moby/moby/client"
)

// Container wraps Docker container management for e2e tests.
type Container struct {
	ID        string
	Name      string
	client    *dockerclient.Client
	hostPorts map[string]string // containerPort -> hostPort
}

// ContainerConfig holds configuration for creating a container.
type ContainerConfig struct {
	Name         string
	Image        string
	Cmd          []string
	NetworkID    string
	Mounts       []Mount
	ExposedPorts []string
	Env          []string
	ExtraHosts   []string // Extra host mappings (e.g., "host.docker.internal:host-gateway")
}

// Mount represents a volume mount configuration.
type Mount struct {
	HostPath      string
	ContainerPath string
	ReadOnly      bool
}

// CreateAndStart creates a Docker container and starts it.
func CreateAndStart(ctx context.Context, client *dockerclient.Client, cfg ContainerConfig) (*Container, error) {
	log.Printf("[docker] Creating container %s with image %s", cfg.Name, cfg.Image)
	if len(cfg.ExtraHosts) > 0 {
		log.Printf("[docker] Extra hosts: %v", cfg.ExtraHosts)
	}

	// Build port bindings - expose to random host ports
	exposedPorts := nat.PortSet{}
	portBindings := nat.PortMap{}
	for _, port := range cfg.ExposedPorts {
		natPort := nat.Port(port + "/tcp")
		exposedPorts[natPort] = struct{}{}
		portBindings[natPort] = []nat.PortBinding{
			{HostIP: "0.0.0.0", HostPort: ""}, // Empty HostPort = random assignment
		}
	}

	// Build mounts
	var mounts []mount.Mount
	for _, m := range cfg.Mounts {
		mounts = append(mounts, mount.Mount{
			Type:     mount.TypeBind,
			Source:   m.HostPath,
			Target:   m.ContainerPath,
			ReadOnly: m.ReadOnly,
		})
	}

	// Build network config - only add if NetworkID is specified
	var networkConfig *network.NetworkingConfig
	if cfg.NetworkID != "" {
		networkConfig = &network.NetworkingConfig{
			EndpointsConfig: map[string]*network.EndpointSettings{
				cfg.NetworkID: {},
			},
		}
	}

	// Create container
	resp, err := client.ContainerCreate(
		ctx,
		&container.Config{
			Image:        cfg.Image,
			Cmd:          cfg.Cmd,
			ExposedPorts: exposedPorts,
			Env:          cfg.Env,
		},
		&container.HostConfig{
			PortBindings: portBindings,
			Mounts:       mounts,
			AutoRemove:   false,
			ExtraHosts:   cfg.ExtraHosts,
		},
		networkConfig,
		nil,
		cfg.Name,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create container: %w", err)
	}

	// Start container
	log.Printf("[docker] Container %s created (ID: %s), starting...", cfg.Name, resp.ID[:12])
	if err := client.ContainerStart(ctx, resp.ID, container.StartOptions{}); err != nil {
		// Clean up the created container
		_ = client.ContainerRemove(ctx, resp.ID, container.RemoveOptions{Force: true})
		return nil, fmt.Errorf("failed to start container: %w", err)
	}
	log.Printf("[docker] Container %s started, waiting for port bindings...", cfg.Name)

	// Get assigned host ports with retry logic
	// Sometimes Docker takes a moment to assign port bindings after container start
	var hostPorts map[string]string
	maxRetries := 30
	retryDelay := 100 * time.Millisecond

	for i := 0; i < maxRetries; i++ {
		inspect, err := client.ContainerInspect(ctx, resp.ID)
		if err != nil {
			_ = client.ContainerStop(ctx, resp.ID, container.StopOptions{})
			_ = client.ContainerRemove(ctx, resp.ID, container.RemoveOptions{Force: true})
			return nil, fmt.Errorf("failed to inspect container: %w", err)
		}

		hostPorts = make(map[string]string)
		for port, bindings := range inspect.NetworkSettings.Ports {
			if len(bindings) > 0 && bindings[0].HostPort != "" {
				hostPorts[port.Port()] = bindings[0].HostPort
			}
		}

		// Check if we got all expected port bindings
		if len(hostPorts) >= len(cfg.ExposedPorts) {
			break
		}

		// Log retry attempt for debugging
		if i > 0 {
			log.Printf("Waiting for port bindings (attempt %d/%d): got %d/%d ports",
				i+1, maxRetries, len(hostPorts), len(cfg.ExposedPorts))
		}

		time.Sleep(retryDelay)
	}

	// Final check - warn if we didn't get all ports but continue if we got at least some
	if len(hostPorts) == 0 && len(cfg.ExposedPorts) > 0 {
		_ = client.ContainerStop(ctx, resp.ID, container.StopOptions{})
		_ = client.ContainerRemove(ctx, resp.ID, container.RemoveOptions{Force: true})
		return nil, fmt.Errorf("no port bindings found after %d retries", maxRetries)
	}

	// Log the assigned port bindings
	for containerPort, hostPort := range hostPorts {
		log.Printf("[docker] Container %s port binding: %s -> 127.0.0.1:%s", cfg.Name, containerPort, hostPort)
	}

	return &Container{
		ID:        resp.ID,
		Name:      cfg.Name,
		client:    client,
		hostPorts: hostPorts,
	}, nil
}

// WaitForHealth waits for the container to be healthy using the provided health check function.
// The healthCheck function should return nil when the container is ready.
func (c *Container) WaitForHealth(ctx context.Context, timeout time.Duration, healthCheck func() error) error {
	deadline := time.Now().Add(timeout)
	backoff := 500 * time.Millisecond
	maxBackoff := 5 * time.Second

	for {
		if time.Now().After(deadline) {
			return fmt.Errorf("health check timed out after %v", timeout)
		}

		// Check if container is still running
		inspect, err := c.client.ContainerInspect(ctx, c.ID)
		if err != nil {
			return fmt.Errorf("container inspect failed: %w", err)
		}
		if !inspect.State.Running {
			logs, _ := c.Logs(ctx)
			return fmt.Errorf("container exited unexpectedly (status: %s), logs:\n%s", inspect.State.Status, logs)
		}

		// Try health check
		if err := healthCheck(); err == nil {
			return nil
		}

		// Exponential backoff
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(backoff):
			backoff = min(backoff*2, maxBackoff)
		}
	}
}

// GetHostPort returns the host port mapped to a container port.
func (c *Container) GetHostPort(containerPort string) string {
	return c.hostPorts[containerPort]
}

// GetHostAddress returns the full host address (127.0.0.1:port) for a container port.
func (c *Container) GetHostAddress(containerPort string) string {
	if port := c.GetHostPort(containerPort); port != "" {
		return fmt.Sprintf("127.0.0.1:%s", port)
	}
	return ""
}

// Logs returns the container logs.
func (c *Container) Logs(ctx context.Context) (string, error) {
	reader, err := c.client.ContainerLogs(ctx, c.ID, container.LogsOptions{
		ShowStdout: true,
		ShowStderr: true,
		Tail:       "200",
	})
	if err != nil {
		return "", fmt.Errorf("failed to get container logs: %w", err)
	}
	defer reader.Close()

	logs, err := io.ReadAll(reader)
	if err != nil {
		return "", fmt.Errorf("failed to read container logs: %w", err)
	}

	return string(logs), nil
}

// Stop stops the container.
func (c *Container) Stop(ctx context.Context) error {
	return c.client.ContainerStop(ctx, c.ID, container.StopOptions{})
}

// Remove removes the container.
func (c *Container) Remove(ctx context.Context) {
	if err := c.client.ContainerRemove(ctx, c.ID, container.RemoveOptions{Force: true}); err != nil {
		log.Printf("failed to remove container %s: %v", c.Name, err)
	}
}

// StopAndRemove stops and removes the container.
func (c *Container) StopAndRemove(ctx context.Context) {
	if err := c.Stop(ctx); err != nil {
		log.Printf("failed to stop container %s: %v", c.Name, err)
	}
	c.Remove(ctx)
}

// IsRunning returns true if the container is currently running.
func (c *Container) IsRunning(ctx context.Context) (bool, error) {
	inspect, err := c.client.ContainerInspect(ctx, c.ID)
	if err != nil {
		return false, fmt.Errorf("failed to inspect container: %w", err)
	}
	return inspect.State.Running, nil
}

// GetNetworkInfo returns debug information about the container's network configuration.
func (c *Container) GetNetworkInfo(ctx context.Context) (string, error) {
	inspect, err := c.client.ContainerInspect(ctx, c.ID)
	if err != nil {
		return "", fmt.Errorf("failed to inspect container: %w", err)
	}

	var b strings.Builder
	fmt.Fprintf(&b, "Container %s (%s) network info:\n", c.Name, c.ID)
	fmt.Fprintf(&b, "  State: %s (Running: %v)\n", inspect.State.Status, inspect.State.Running)

	if inspect.NetworkSettings != nil {
		fmt.Fprintf(&b, "  IPAddress: %s\n", inspect.NetworkSettings.IPAddress)
		for netName, netSettings := range inspect.NetworkSettings.Networks {
			fmt.Fprintf(&b, "  Network '%s':\n", netName)
			fmt.Fprintf(&b, "    NetworkID: %s\n", netSettings.NetworkID)
			fmt.Fprintf(&b, "    IPAddress: %s\n", netSettings.IPAddress)
			fmt.Fprintf(&b, "    Gateway: %s\n", netSettings.Gateway)
			if len(netSettings.Aliases) > 0 {
				fmt.Fprintf(&b, "    Aliases: %v\n", netSettings.Aliases)
			}
		}
		b.WriteString("  Ports:\n")
		for port, bindings := range inspect.NetworkSettings.Ports {
			if len(bindings) > 0 {
				fmt.Fprintf(&b, "    %s -> %s:%s\n", port, bindings[0].HostIP, bindings[0].HostPort)
			} else {
				fmt.Fprintf(&b, "    %s -> (no binding)\n", port)
			}
		}
	}

	return b.String(), nil
}

// GenerateContainerName generates a unique container name using random bytes.
// Returns a 64-character hex string (32 random bytes encoded as hex).
func GenerateContainerName() string {
	randomBytes := make([]byte, 32)
	if _, err := rand.Read(randomBytes); err != nil {
		// Fallback to timestamp-based if crypto/rand fails
		return fmt.Sprintf("%x", time.Now().UnixNano())
	}
	return hex.EncodeToString(randomBytes)
}
