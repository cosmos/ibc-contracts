// SPDX-License-Identifier: Apache-2.0

package proofapi

import (
	"encoding/json"
	"errors"
	"fmt"
	"math/rand"
	"net"
	"os"
	"text/template"
	"time"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

const (
	ModuleCosmosToCosmos    = "cosmos_to_cosmos"
	ModuleCosmosToEth       = "cosmos_to_eth"
	ModuleEthToCosmos       = "eth_to_cosmos"
	ModuleEthToCosmosCompat = "eth_to_cosmos_compat"
	ModuleEthToEth          = "eth_to_eth"
	ModuleSolanaToCosmos    = "solana_to_cosmos"
	ModuleCosmosToSolana    = "cosmos_to_solana"
	ModuleEthToSolana       = "eth_to_solana"
	ModuleSolanaToEth       = "solana_to_eth"
)

// Config represents the proof API configuration structure aligned with the Rust ProofApiConfig
// and serves as template data for generation in e2e.
type Config struct {
	Modules       []ModuleConfig      `json:"modules"`
	Server        ServerConfig        `json:"server"`
	Observability ObservabilityConfig `json:"observability"`
}

// ServerConfig mirrors the Rust ServerConfig
type ServerConfig struct {
	Address string `json:"address"`
	Port    int    `json:"port"`
}

// ObservabilityConfig mirrors the Rust ObservabilityConfig
type ObservabilityConfig struct {
	Level        string  `json:"level"`
	UseOtel      bool    `json:"use_otel"`
	ServiceName  string  `json:"service_name"`
	OTelEndpoint *string `json:"otel_endpoint,omitempty"`
}

// ModuleConfig represents a module configuration
type ModuleConfig struct {
	Name     string `json:"name"`
	SrcChain string `json:"src_chain"`
	DstChain string `json:"dst_chain"`
	Config   any    `json:"config"`
}

// SP1ProverConfig represents the configuration for SP1 prover
type SP1ProverConfig struct {
	Type              string `json:"type"`
	NetworkPrivateKey string `json:"network_private_key,omitempty"`
	NetworkRpcUrl     string `json:"network_rpc_url,omitempty"`
	PrivateCluster    bool   `json:"private_cluster,omitempty"`
}

type SP1ProgramPaths struct {
	UpdateClient              string `json:"update_client"`
	Membership                string `json:"membership"`
	UpdateClientAndMembership string `json:"update_client_and_membership"`
	Misbehaviour              string `json:"misbehaviour"`
}

// CosmosToCosmosModuleConfig represents the configuration for cosmos_to_cosmos module
type CosmosToCosmosModuleConfig struct {
	SrcRpcUrl     string `json:"src_rpc_url"`
	TargetRpcUrl  string `json:"target_rpc_url"`
	SignerAddress string `json:"signer_address"`
}

// GetAvailablePort returns a random available TCP port on localhost within a typical ephemeral range.
// It tries up to maxAttempts times to find a free port by binding and immediately releasing it.
func GetAvailablePort() (int, error) {
	const (
		minPort     = 20000
		maxPort     = 40000
		maxAttempts = 50
	)
	// Seed RNG once per process
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < maxAttempts; i++ {
		p := rand.Intn(maxPort-minPort+1) + minPort
		ln, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", p))
		if err == nil {
			_ = ln.Close()
			return p, nil
		}
	}
	return 0, errors.New("failed to find an available port")
}

// NewConfig creates a new Config with default values
func NewConfig(modules []ModuleConfig) Config {
	addr := "127.0.0.1"
	port, err := GetAvailablePort()
	if err != nil {
		// Fallback to a sensible default if no port found; tests may still override via env
		port = 3000
	}

	// Make the chosen address visible to tests that use DefaultProofAPIGRPCAddress.
	_ = os.Setenv("PROOF_API_GRPC_ADDR", fmt.Sprintf("%s:%d", addr, port))

	// Observability configuration
	rustLog := os.Getenv(testvalues.EnvKeyRustLog)
	if rustLog == "" {
		rustLog = testvalues.EnvValueRustLog_Info
	}

	// Local observability flag strictly equals "true"
	enableLocalObservability := os.Getenv(testvalues.EnvKeyEnableLocalObservability) == testvalues.EnvValueEnableLocalObservability_True

	var otlpEndpoint *string
	tracingLevel := rustLog
	useOtel := false
	if enableLocalObservability {
		// Force the endpoint and enable OTLP, but respect RUST_LOG for level
		endpoint := "http://127.0.0.1:4317"
		otlpEndpoint = &endpoint
		useOtel = true
	}

	observability := ObservabilityConfig{
		Level:        tracingLevel,
		UseOtel:      useOtel,
		ServiceName:  "proof-api",
		OTelEndpoint: otlpEndpoint,
	}

	return Config{
		Modules: modules,
		Server: ServerConfig{
			Address: addr,
			Port:    port,
		},
		Observability: observability,
	}
}

// GenerateConfig creates a config from the template
func (c *Config) GenerateConfigFile(filePath string) error {
	tmpl, err := template.ParseFiles("e2e/interchaintestv8/proofapi/config.tmpl")
	if err != nil {
		return err
	}

	f, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer f.Close()

	return tmpl.Execute(f, c)
}

// ToJSON converts a ModuleConfig to JSON string - designed for use in templates
func (mc ModuleConfig) ToJSON() string {
	jsonBytes, err := json.MarshalIndent(mc, "", "  ")
	if err != nil {
		return fmt.Sprintf("\"Error generating JSON: %s\"", err)
	}
	return string(jsonBytes)
}

// ModulesToJSON converts a slice of ModuleConfig to a JSON string
func ModulesToJSON(modules []ModuleConfig) (string, error) {
	if len(modules) == 0 {
		return "[]", nil
	}

	jsonBytes, err := json.MarshalIndent(modules, "", "  ")
	if err != nil {
		return "", fmt.Errorf("failed to marshal modules to JSON: %w", err)
	}

	return string(jsonBytes), nil
}

// EthToCosmosModuleConfig represents the configuration for eth_to_cosmos module
type EthToCosmosModuleConfig struct {
	Ics26Address    string        `json:"ics26_address"`
	TmRpcUrl        string        `json:"tm_rpc_url"`
	EthRpcUrl       string        `json:"eth_rpc_url"`
	EthBeaconApiUrl string        `json:"eth_beacon_api_url"`
	SignerAddress   string        `json:"signer_address"`
	Mode            TxBuilderMode `json:"mode"`
}

// TxBuilderMode serializes to Rust's externally tagged enum format.
// Unit variants serialize as strings, tuple variants as {"variant": content}.
type TxBuilderMode interface {
	json.Marshaler
}

// RealMode uses Ethereum beacon chain proofs.
type RealMode struct{}

func (RealMode) MarshalJSON() ([]byte, error) { return json.Marshal("real") }

// MockMode is for testing without real proofs.
type MockMode struct{}

func (MockMode) MarshalJSON() ([]byte, error) { return json.Marshal("mock") }

// AttestedMode uses aggregator attestations.
type AttestedMode struct {
	Config AggregatorConfig
}

func (m AttestedMode) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]AggregatorConfig{"attested": m.Config})
}

// SP1Mode uses zero-knowledge proofs.
type SP1Mode struct {
	Prover   SP1ProverConfig
	Programs SP1ProgramPaths
}

func (m SP1Mode) MarshalJSON() ([]byte, error) {
	return json.Marshal(map[string]interface{}{
		"sp1": map[string]interface{}{
			"sp1_prover":   m.Prover,
			"sp1_programs": m.Programs,
		},
	})
}

// CosmosToEthModuleConfig represents the configuration for cosmos_to_eth module.
type CosmosToEthModuleConfig struct {
	TmRpcUrl     string        `json:"tm_rpc_url"`
	Ics26Address string        `json:"ics26_address"`
	EthRpcUrl    string        `json:"eth_rpc_url"`
	Mode         TxBuilderMode `json:"mode"`
}

// EthToEthModuleConfig represents the configuration for eth_to_eth module
type EthToEthModuleConfig struct {
	SrcRpcUrl       string        `json:"src_rpc_url"`
	SrcIcs26Address string        `json:"src_ics26_address"`
	DstRpcUrl       string        `json:"dst_rpc_url"`
	DstIcs26Address string        `json:"dst_ics26_address"`
	Mode            TxBuilderMode `json:"mode"`
}

// AttestorConfig represents the attestor configuration section
type AttestorConfig struct {
	AttestorQueryTimeoutMs int      `json:"attestor_query_timeout_ms"`
	QuorumThreshold        int      `json:"quorum_threshold"`
	AttestorEndpoints      []string `json:"attestor_endpoints"`
}

// CacheConfig represents the cache configuration section
type CacheConfig struct {
	StateCacheMaxEntries  int `json:"state_cache_max_entries"`
	PacketCacheMaxEntries int `json:"packet_cache_max_entries"`
}

// AggregatorConfig represents the full aggregator configuration
type AggregatorConfig struct {
	Attestor AttestorConfig `json:"attestor"`
	Cache    CacheConfig    `json:"cache"`
}

// DefaultAggregatorConfig returns a config with sensible defaults
func DefaultAggregatorConfig() AggregatorConfig {
	return AggregatorConfig{
		Attestor: AttestorConfig{
			AttestorQueryTimeoutMs: 5000,
			QuorumThreshold:        1,
			AttestorEndpoints:      []string{"http://127.0.0.1:2025"},
		},
		Cache: CacheConfig{
			StateCacheMaxEntries:  100000,
			PacketCacheMaxEntries: 100000,
		},
	}
}

// DefaultSP1ProgramPaths returns the default paths for SP1 program ELF files
func DefaultSP1ProgramPaths() SP1ProgramPaths {
	return SP1ProgramPaths{
		UpdateClient:              "./ibc-solidity/programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/sp1-ics07-tendermint-update-client",
		Membership:                "./ibc-solidity/programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/sp1-ics07-tendermint-membership",
		UpdateClientAndMembership: "./ibc-solidity/programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/sp1-ics07-tendermint-uc-and-membership",
		Misbehaviour:              "./ibc-solidity/programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/sp1-ics07-tendermint-misbehaviour",
	}
}

// SolanaToCosmosModuleConfig represents the configuration for solana_to_cosmos module
type SolanaToCosmosModuleConfig struct {
	SolanaChainId        string        `json:"solana_chain_id"`
	SrcRpcUrl            string        `json:"src_rpc_url"`
	TargetRpcUrl         string        `json:"target_rpc_url"`
	SignerAddress        string        `json:"signer_address"`
	SolanaIcs26ProgramId string        `json:"solana_ics26_program_id"`
	Mode                 TxBuilderMode `json:"mode"`
}

// CosmosToSolanaModuleConfig represents the configuration for cosmos_to_solana module
type CosmosToSolanaModuleConfig struct {
	SourceRpcUrl           string        `json:"source_rpc_url"`
	TargetRpcUrl           string        `json:"target_rpc_url"`
	SolanaIcs26ProgramId   string        `json:"solana_ics26_program_id"`
	SolanaFeePayer         string        `json:"solana_fee_payer"`
	SolanaAltAddress       *string       `json:"solana_alt_address,omitempty"`
	SolanaIftProgramIds    []string      `json:"solana_ift_program_ids,omitempty"`
	SkipPreVerifyThreshold *int          `json:"skip_pre_verify_threshold,omitempty"`
	Mode                   TxBuilderMode `json:"mode,omitempty"`
}

type EthToSolanaModuleConfig struct {
	Ics26Address         string        `json:"ics26_address"`
	EthRpcUrl            string        `json:"eth_rpc_url"`
	SolanaRpcUrl         string        `json:"solana_rpc_url"`
	SolanaIcs26ProgramId string        `json:"solana_ics26_program_id"`
	SolanaFeePayer       string        `json:"solana_fee_payer"`
	SolanaAltAddress     *string       `json:"solana_alt_address,omitempty"`
	SolanaIftProgramIds  []string      `json:"solana_ift_program_ids,omitempty"`
	Mode                 TxBuilderMode `json:"mode"`
}

type SolanaToEthModuleConfig struct {
	SolanaRpcUrl         string        `json:"solana_rpc_url"`
	SolanaIcs26ProgramId string        `json:"solana_ics26_program_id"`
	EthRpcUrl            string        `json:"eth_rpc_url"`
	Ics26Address         string        `json:"ics26_address"`
	Mode                 TxBuilderMode `json:"mode"`
}
