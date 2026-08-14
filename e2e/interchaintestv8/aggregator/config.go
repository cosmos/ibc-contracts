// SPDX-License-Identifier: Apache-2.0

package aggregator

import (
	"os"

	"github.com/BurntSushi/toml"
)

// ServerConfig represents the server configuration section
type ServerConfig struct {
	ListenerAddr string `toml:"listener_addr"`
	LogLevel     string `toml:"log_level"`
}

// AttestorConfig represents the attestor configuration section
type AttestorConfig struct {
	AttestorQueryTimeoutMs int      `toml:"attestor_query_timeout_ms"`
	QuorumThreshold        int      `toml:"quorum_threshold"`
	AttestorEndpoints      []string `toml:"attestor_endpoints"`
}

// CacheConfig represents the cache configuration section
type CacheConfig struct {
	StateCacheMaxEntries  int `toml:"state_cache_max_entries"`
	PacketCacheMaxEntries int `toml:"packet_cache_max_entries"`
}

// AggregatorConfig represents the full aggregator configuration
type AggregatorConfig struct {
	Server   ServerConfig   `toml:"server"`
	Attestor AttestorConfig `toml:"attestor"`
	Cache    CacheConfig    `toml:"cache"`
}

// DefaultAggregatorConfig returns a config with sensible defaults
func DefaultAggregatorConfig() *AggregatorConfig {
	return &AggregatorConfig{
		Server: ServerConfig{
			ListenerAddr: "127.0.0.1:8080",
			LogLevel:     "INFO",
		},
		Attestor: AttestorConfig{
			AttestorQueryTimeoutMs: 5000,
			QuorumThreshold:        1,
			AttestorEndpoints:      []string{"http://127.0.0.1:9000"},
		},
		Cache: CacheConfig{
			StateCacheMaxEntries:  100000,
			PacketCacheMaxEntries: 100000,
		},
	}
}

// NewAggregatorConfigWithEndpoints creates a config with specified attestor endpoints
func NewAggregatorConfigWithEndpoints(attestorEndpoints []string, quorumThreshold int) *AggregatorConfig {
	config := DefaultAggregatorConfig()
	config.Attestor.AttestorEndpoints = attestorEndpoints
	config.Attestor.QuorumThreshold = quorumThreshold
	return config
}

// WriteTomlConfig writes the aggregator config to a TOML file
func (c *AggregatorConfig) WriteTomlConfig(filePath string) error {
	file, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer file.Close()

	encoder := toml.NewEncoder(file)
	return encoder.Encode(c)
}

// GetServerAddress returns the server address
func (c *AggregatorConfig) GetServerAddress() string {
	return c.Server.ListenerAddr
}

// GetAttestorEndpoints returns the attestor endpoints
func (c *AggregatorConfig) GetAttestorEndpoints() []string {
	return c.Attestor.AttestorEndpoints
}

// GetQuorumThreshold returns the quorum threshold
func (c *AggregatorConfig) GetQuorumThreshold() int {
	return c.Attestor.QuorumThreshold
}

// CleanupConfig removes the config file
func CleanupConfig(filePath string) error {
	return os.Remove(filePath)
}
