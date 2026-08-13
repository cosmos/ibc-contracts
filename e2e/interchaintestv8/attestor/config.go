// SPDX-License-Identifier: Apache-2.0

package attestor

import (
	"fmt"
	"net"
	"os"

	"github.com/BurntSushi/toml"
)

// ServerConfig represents the server configuration section
type ServerConfig struct {
	ListenAddr string `toml:"listen_addr"`
	HealthAddr string `toml:"health_addr"`
}

// AdapterConfig represents the blockchain adapter configuration
// This can be used for EVM, Cosmos, or Solana chains
type AdapterConfig struct {
	URL            string `toml:"url"`
	RouterAddress  string `toml:"router_address,omitempty"` // Used for EVM and Solana (program ID)
	FinalityOffset uint64 `toml:"finality_offset,omitempty"`
}

type LocalSignerConfig struct {
	KeystorePath string `toml:"keystore_path,omitempty"`
}

// AttestorConfig represents the full attestor configuration
type AttestorConfig struct {
	Server  ServerConfig      `toml:"server"`
	Signer  LocalSignerConfig `toml:"signer,omitempty"`
	Adapter AdapterConfig     `toml:"adapter"`
}

// DefaultAttestorConfig returns a config with sensible defaults
func DefaultAttestorConfig() *AttestorConfig {
	return &AttestorConfig{
		Server: ServerConfig{
			ListenAddr: "0.0.0.0:2025",
			HealthAddr: "0.0.0.0:8081",
		},
		Signer: LocalSignerConfig{
			KeystorePath: KeystorePath(0),
		},
		Adapter: AdapterConfig{
			URL:           "https://api.tatum.io/v3/blockchain/node/ethereum-mainnet",
			RouterAddress: "0xa348CfE719B63151F228e3C30EB424BA5a983012",
		},
	}
}

// WriteTomlConfig writes the attestor config to a TOML file
func (c *AttestorConfig) WriteTomlConfig(filePath string) error {
	file, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer file.Close()

	encoder := toml.NewEncoder(file)
	return encoder.Encode(c)
}

// GetServerAddress returns the server address and port
func (c *AttestorConfig) GetServerAddress() string {
	return c.Server.ListenAddr
}

// GetServerPort returns just the port number
func (c *AttestorConfig) GetServerPort() (int, error) {
	// Parse the listen_addr to extract the port
	_, portStr, err := net.SplitHostPort(c.Server.ListenAddr)
	if err != nil {
		return 0, fmt.Errorf("failed to parse listen_addr: %w", err)
	}

	var port int
	_, err = fmt.Sscanf(portStr, "%d", &port)
	if err != nil {
		return 0, fmt.Errorf("failed to parse port: %w", err)
	}

	return port, nil
}

// GetAdapterURL returns the blockchain adapter URL
func (c *AttestorConfig) GetAdapterURL() string {
	return c.Adapter.URL
}

// CleanupConfig removes the config file
func CleanupConfig(filePath string) error {
	return os.Remove(filePath)
}
