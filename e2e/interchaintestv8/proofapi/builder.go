// SPDX-License-Identifier: Apache-2.0

package proofapi

import "os"

// ConfigBuilder provides a fluent API for building proof API configurations.
type ConfigBuilder struct {
	modules []ModuleConfig
}

// NewConfigBuilder creates a new configuration builder.
func NewConfigBuilder() *ConfigBuilder {
	return &ConfigBuilder{
		modules: make([]ModuleConfig, 0),
	}
}

// AddModule adds a pre-built module to the configuration.
func (b *ConfigBuilder) AddModule(module ModuleConfig) *ConfigBuilder {
	b.modules = append(b.modules, module)
	return b
}

// Build creates the final Config from all added modules.
func (b *ConfigBuilder) Build() Config {
	return NewConfig(b.modules)
}

// =============================================================================
// Eth → Cosmos (Beacon Chain Light Client)
// =============================================================================

// EthToCosmosParams contains parameters for Eth→Cosmos module using beacon chain.
type EthToCosmosParams struct {
	EthChainID    string
	CosmosChainID string
	TmRPC         string
	ICS26Address  string
	EthRPC        string
	BeaconAPI     string
	SignerAddress string
	MockClient    bool
}

// EthToCosmos adds an Eth→Cosmos compat module using beacon chain light client.
func (b *ConfigBuilder) EthToCosmos(p EthToCosmosParams) *ConfigBuilder {
	var mode TxBuilderMode
	if p.MockClient {
		mode = MockMode{}
	} else {
		mode = RealMode{}
	}

	module := ModuleConfig{
		Name:     ModuleEthToCosmosCompat,
		SrcChain: p.EthChainID,
		DstChain: p.CosmosChainID,
		Config: EthToCosmosModuleConfig{
			TmRpcUrl:        p.TmRPC,
			Ics26Address:    p.ICS26Address,
			EthRpcUrl:       p.EthRPC,
			EthBeaconApiUrl: p.BeaconAPI,
			SignerAddress:   p.SignerAddress,
			Mode:            mode,
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Eth → Cosmos (Attestor)
// =============================================================================

// EthToCosmosAttestedParams contains parameters for Eth→Cosmos module using attestor.
type EthToCosmosAttestedParams struct {
	EthChainID        string
	CosmosChainID     string
	TmRPC             string
	ICS26Address      string
	EthRPC            string
	SignerAddress     string
	AttestorEndpoints []string
	AttestorTimeout   int // Optional, defaults to 5000
	QuorumThreshold   int // Optional, defaults to 1
}

// EthToCosmosAttested adds an Eth→Cosmos module using attestor.
func (b *ConfigBuilder) EthToCosmosAttested(p EthToCosmosAttestedParams) *ConfigBuilder {
	aggConfig := DefaultAggregatorConfig()
	if len(p.AttestorEndpoints) > 0 {
		aggConfig.Attestor.AttestorEndpoints = p.AttestorEndpoints
	}
	if p.AttestorTimeout > 0 {
		aggConfig.Attestor.AttestorQueryTimeoutMs = p.AttestorTimeout
	}
	if p.QuorumThreshold > 0 {
		aggConfig.Attestor.QuorumThreshold = p.QuorumThreshold
	}

	module := ModuleConfig{
		Name:     ModuleEthToCosmos,
		SrcChain: p.EthChainID,
		DstChain: p.CosmosChainID,
		Config: EthToCosmosModuleConfig{
			Ics26Address:    p.ICS26Address,
			TmRpcUrl:        p.TmRPC,
			EthRpcUrl:       p.EthRPC,
			EthBeaconApiUrl: "",
			SignerAddress:   p.SignerAddress,
			Mode:            AttestedMode{Config: aggConfig},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Cosmos → Eth (SP1)
// =============================================================================

// CosmosToEthSP1Params contains parameters for Cosmos→Eth module using SP1 proofs.
type CosmosToEthSP1Params struct {
	CosmosChainID string
	EthChainID    string
	TmRPC         string
	ICS26Address  string
	EthRPC        string
	Prover        SP1ProverConfig
}

// CosmosToEthSP1 adds a Cosmos→Eth module using SP1 proofs.
func (b *ConfigBuilder) CosmosToEthSP1(p CosmosToEthSP1Params) *ConfigBuilder {
	sp1Programs := DefaultSP1ProgramPaths()
	module := ModuleConfig{
		Name:     ModuleCosmosToEth,
		SrcChain: p.CosmosChainID,
		DstChain: p.EthChainID,
		Config: CosmosToEthModuleConfig{
			TmRpcUrl:     p.TmRPC,
			Ics26Address: p.ICS26Address,
			EthRpcUrl:    p.EthRPC,
			Mode:         SP1Mode{Prover: p.Prover, Programs: sp1Programs},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Cosmos → Eth (Attestor)
// =============================================================================

// CosmosToEthAttestedParams contains parameters for Cosmos→Eth module using attestor.
type CosmosToEthAttestedParams struct {
	CosmosChainID     string
	EthChainID        string
	TmRPC             string
	ICS26Address      string
	EthRPC            string
	AttestorEndpoints []string
	AttestorTimeout   int // Optional, defaults to 5000
	QuorumThreshold   int // Optional, defaults to 1
}

// CosmosToEthAttested adds a Cosmos→Eth module using attestor.
func (b *ConfigBuilder) CosmosToEthAttested(p CosmosToEthAttestedParams) *ConfigBuilder {
	aggConfig := DefaultAggregatorConfig()
	if len(p.AttestorEndpoints) > 0 {
		aggConfig.Attestor.AttestorEndpoints = p.AttestorEndpoints
	}
	if p.AttestorTimeout > 0 {
		aggConfig.Attestor.AttestorQueryTimeoutMs = p.AttestorTimeout
	}
	if p.QuorumThreshold > 0 {
		aggConfig.Attestor.QuorumThreshold = p.QuorumThreshold
	}

	module := ModuleConfig{
		Name:     ModuleCosmosToEth,
		SrcChain: p.CosmosChainID,
		DstChain: p.EthChainID,
		Config: CosmosToEthModuleConfig{
			TmRpcUrl:     p.TmRPC,
			Ics26Address: p.ICS26Address,
			EthRpcUrl:    p.EthRPC,
			Mode:         AttestedMode{Config: aggConfig},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Eth → Eth (Attestor)
// =============================================================================

// EthToEthAttestedParams contains parameters for Eth→Eth module using attestor.
type EthToEthAttestedParams struct {
	SrcChainID        string
	DstChainID        string
	SrcRPC            string
	DstRPC            string
	SrcICS26          string
	DstICS26          string
	AttestorEndpoints []string
	AttestorTimeout   int // Optional, defaults to 5000
	QuorumThreshold   int // Optional, defaults to 1
}

// EthToEthAttested adds an Eth→Eth module using attestor.
func (b *ConfigBuilder) EthToEthAttested(p EthToEthAttestedParams) *ConfigBuilder {
	aggConfig := DefaultAggregatorConfig()
	if len(p.AttestorEndpoints) > 0 {
		aggConfig.Attestor.AttestorEndpoints = p.AttestorEndpoints
	}
	if p.AttestorTimeout > 0 {
		aggConfig.Attestor.AttestorQueryTimeoutMs = p.AttestorTimeout
	}
	if p.QuorumThreshold > 0 {
		aggConfig.Attestor.QuorumThreshold = p.QuorumThreshold
	}

	module := ModuleConfig{
		Name:     ModuleEthToEth,
		SrcChain: p.SrcChainID,
		DstChain: p.DstChainID,
		Config: EthToEthModuleConfig{
			SrcRpcUrl:       p.SrcRPC,
			SrcIcs26Address: p.SrcICS26,
			DstRpcUrl:       p.DstRPC,
			DstIcs26Address: p.DstICS26,
			Mode:            AttestedMode{Config: aggConfig},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Besu → Besu
// =============================================================================

// BesuToBesuParams contains parameters for besu_to_besu module.
type BesuToBesuParams struct {
	SrcChainID    string
	DstChainID    string
	SrcRPC        string
	DstRPC        string
	SrcICS26      string
	DstICS26      string
	ConsensusType string
}

// BesuToBesu adds a Besu→Besu module.
func (b *ConfigBuilder) BesuToBesu(p BesuToBesuParams) *ConfigBuilder {
	module := ModuleConfig{
		Name:     ModuleBesuToBesu,
		SrcChain: p.SrcChainID,
		DstChain: p.DstChainID,
		Config: BesuToBesuModuleConfig{
			SrcChainID:      p.SrcChainID,
			SrcRPCURL:       p.SrcRPC,
			SrcICS26Address: p.SrcICS26,
			DstRPCURL:       p.DstRPC,
			DstICS26Address: p.DstICS26,
			ConsensusType:   p.ConsensusType,
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Cosmos → Cosmos
// =============================================================================

// CosmosToCosmosParams contains parameters for Cosmos→Cosmos module.
type CosmosToCosmosParams struct {
	SrcChainID    string
	DstChainID    string
	SrcRPC        string
	DstRPC        string
	SignerAddress string
}

// CosmosToCosmos adds a Cosmos→Cosmos module.
func (b *ConfigBuilder) CosmosToCosmos(p CosmosToCosmosParams) *ConfigBuilder {
	module := ModuleConfig{
		Name:     ModuleCosmosToCosmos,
		SrcChain: p.SrcChainID,
		DstChain: p.DstChainID,
		Config: CosmosToCosmosModuleConfig{
			SrcRpcUrl:     p.SrcRPC,
			TargetRpcUrl:  p.DstRPC,
			SignerAddress: p.SignerAddress,
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Solana → Cosmos
// =============================================================================

// SolanaToCosmosMockParams contains parameters for Solana→Cosmos module.
type SolanaToCosmosMockParams struct {
	SolanaChainID  string
	CosmosChainID  string
	SolanaRPC      string
	TmRPC          string
	ICS26ProgramID string
	SignerAddress  string
	MockClient     bool
}

// SolanaToCosmosMock adds a Solana→Cosmos module.
func (b *ConfigBuilder) SolanaToCosmosMock(p SolanaToCosmosMockParams) *ConfigBuilder {
	module := ModuleConfig{
		Name:     ModuleSolanaToCosmos,
		SrcChain: p.SolanaChainID,
		DstChain: p.CosmosChainID,
		Config: SolanaToCosmosModuleConfig{
			SolanaChainId:        p.SolanaChainID,
			SrcRpcUrl:            p.SolanaRPC,
			TargetRpcUrl:         p.TmRPC,
			SignerAddress:        p.SignerAddress,
			SolanaIcs26ProgramId: p.ICS26ProgramID,
			Mode:                 MockMode{},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Solana → Cosmos (Attestor)
// =============================================================================

// SolanaToCosmosAttestedParams contains parameters for Solana→Cosmos module.
type SolanaToCosmosAttestedParams struct {
	SolanaChainID     string
	CosmosChainID     string
	SolanaRPC         string
	TmRPC             string
	ICS26ProgramID    string
	SignerAddress     string
	AttestorEndpoints []string
	AttestorTimeout   int // Optional, defaults to 5000
	QuorumThreshold   int // Optional, defaults to 1
}

// SolanaToCosmosAttested adds a Solana→Cosmos module.
func (b *ConfigBuilder) SolanaToCosmosAttested(p SolanaToCosmosAttestedParams) *ConfigBuilder {
	aggConfig := DefaultAggregatorConfig()
	if len(p.AttestorEndpoints) > 0 {
		aggConfig.Attestor.AttestorEndpoints = p.AttestorEndpoints
	}
	if p.AttestorTimeout > 0 {
		aggConfig.Attestor.AttestorQueryTimeoutMs = p.AttestorTimeout
	}
	if p.QuorumThreshold > 0 {
		aggConfig.Attestor.QuorumThreshold = p.QuorumThreshold
	}

	module := ModuleConfig{
		Name:     ModuleSolanaToCosmos,
		SrcChain: p.SolanaChainID,
		DstChain: p.CosmosChainID,
		Config: SolanaToCosmosModuleConfig{
			SolanaChainId:        p.SolanaChainID,
			SrcRpcUrl:            p.SolanaRPC,
			TargetRpcUrl:         p.TmRPC,
			SignerAddress:        p.SignerAddress,
			SolanaIcs26ProgramId: p.ICS26ProgramID,
			Mode:                 AttestedMode{Config: aggConfig},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Cosmos → Solana
// =============================================================================

// CosmosToSolanaParams contains parameters for Cosmos→Solana module.
type CosmosToSolanaParams struct {
	CosmosChainID          string
	SolanaChainID          string
	SolanaRPC              string
	TmRPC                  string
	ICS07ProgramID         string
	ICS26ProgramID         string
	FeePayer               string
	ALTAddress             string   // Optional
	IFTProgramIDs          []string // Optional
	SkipPreVerifyThreshold *int     // Optional
}

// CosmosToSolana adds a Cosmos→Solana module.
func (b *ConfigBuilder) CosmosToSolana(p CosmosToSolanaParams) *ConfigBuilder {
	var altAddress *string
	if p.ALTAddress != "" {
		altAddress = &p.ALTAddress
	}
	module := ModuleConfig{
		Name:     ModuleCosmosToSolana,
		SrcChain: p.CosmosChainID,
		DstChain: p.SolanaChainID,
		Config: CosmosToSolanaModuleConfig{
			SourceRpcUrl:           p.TmRPC,
			TargetRpcUrl:           p.SolanaRPC,
			SolanaIcs26ProgramId:   p.ICS26ProgramID,
			SolanaFeePayer:         p.FeePayer,
			SolanaAltAddress:       altAddress,
			SolanaIftProgramIds:    p.IFTProgramIDs,
			SkipPreVerifyThreshold: p.SkipPreVerifyThreshold,
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Cosmos → Solana (Attestor)
// =============================================================================

// CosmosToSolanaAttestedParams contains parameters for Cosmos→Solana module using attestation light client.
type CosmosToSolanaAttestedParams struct {
	CosmosChainID          string
	SolanaChainID          string
	SolanaRPC              string
	TmRPC                  string
	ICS26ProgramID         string
	FeePayer               string
	ALTAddress             string   // Optional
	IFTProgramIDs          []string // Optional
	SkipPreVerifyThreshold *int     // Optional
	AttestorEndpoints      []string // Required for attestation mode
	AttestorTimeout        int      // Optional, defaults to 5000
	QuorumThreshold        int      // Optional, defaults to 1
}

// CosmosToSolanaAttested adds a Cosmos→Solana module using attestor.
func (b *ConfigBuilder) CosmosToSolanaAttested(p CosmosToSolanaAttestedParams) *ConfigBuilder {
	var altAddress *string
	if p.ALTAddress != "" {
		altAddress = &p.ALTAddress
	}
	aggConfig := DefaultAggregatorConfig()
	if len(p.AttestorEndpoints) > 0 {
		aggConfig.Attestor.AttestorEndpoints = p.AttestorEndpoints
	}
	if p.AttestorTimeout > 0 {
		aggConfig.Attestor.AttestorQueryTimeoutMs = p.AttestorTimeout
	}
	if p.QuorumThreshold > 0 {
		aggConfig.Attestor.QuorumThreshold = p.QuorumThreshold
	}

	module := ModuleConfig{
		Name:     ModuleCosmosToSolana,
		SrcChain: p.CosmosChainID,
		DstChain: p.SolanaChainID,
		Config: CosmosToSolanaModuleConfig{
			SourceRpcUrl:           p.TmRPC,
			TargetRpcUrl:           p.SolanaRPC,
			SolanaIcs26ProgramId:   p.ICS26ProgramID,
			SolanaFeePayer:         p.FeePayer,
			SolanaAltAddress:       altAddress,
			SolanaIftProgramIds:    p.IFTProgramIDs,
			SkipPreVerifyThreshold: p.SkipPreVerifyThreshold,
			Mode:                   AttestedMode{Config: aggConfig},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

type EthToSolanaAttestedParams struct {
	EthChainID        string
	SolanaChainID     string
	EthRPC            string
	ICS26Address      string
	SolanaRPC         string
	ICS26ProgramID    string
	FeePayer          string
	ALTAddress        string   // Optional
	IFTProgramIDs     []string // Optional
	AttestorEndpoints []string // Required for attestation mode
	AttestorTimeout   int      // Optional, defaults to 5000
	QuorumThreshold   int      // Optional, defaults to 1
}

func (b *ConfigBuilder) EthToSolanaAttested(p EthToSolanaAttestedParams) *ConfigBuilder {
	aggConfig := DefaultAggregatorConfig()
	if len(p.AttestorEndpoints) > 0 {
		aggConfig.Attestor.AttestorEndpoints = p.AttestorEndpoints
	}
	if p.AttestorTimeout > 0 {
		aggConfig.Attestor.AttestorQueryTimeoutMs = p.AttestorTimeout
	}
	if p.QuorumThreshold > 0 {
		aggConfig.Attestor.QuorumThreshold = p.QuorumThreshold
	}

	var altAddress *string
	if p.ALTAddress != "" {
		altAddress = &p.ALTAddress
	}
	module := ModuleConfig{
		Name:     ModuleEthToSolana,
		SrcChain: p.EthChainID,
		DstChain: p.SolanaChainID,
		Config: EthToSolanaModuleConfig{
			Ics26Address:         p.ICS26Address,
			EthRpcUrl:            p.EthRPC,
			SolanaRpcUrl:         p.SolanaRPC,
			SolanaIcs26ProgramId: p.ICS26ProgramID,
			SolanaFeePayer:       p.FeePayer,
			SolanaAltAddress:     altAddress,
			SolanaIftProgramIds:  p.IFTProgramIDs,
			Mode:                 AttestedMode{Config: aggConfig},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

type SolanaToEthAttestedParams struct {
	SolanaChainID     string
	EthChainID        string
	SolanaRPC         string
	ICS26ProgramID    string
	EthRPC            string
	ICS26Address      string
	AttestorEndpoints []string
	AttestorTimeout   int // Optional, defaults to 5000
	QuorumThreshold   int // Optional, defaults to 1
}

func (b *ConfigBuilder) SolanaToEthAttested(p SolanaToEthAttestedParams) *ConfigBuilder {
	aggConfig := DefaultAggregatorConfig()
	if len(p.AttestorEndpoints) > 0 {
		aggConfig.Attestor.AttestorEndpoints = p.AttestorEndpoints
	}
	if p.AttestorTimeout > 0 {
		aggConfig.Attestor.AttestorQueryTimeoutMs = p.AttestorTimeout
	}
	if p.QuorumThreshold > 0 {
		aggConfig.Attestor.QuorumThreshold = p.QuorumThreshold
	}

	module := ModuleConfig{
		Name:     ModuleSolanaToEth,
		SrcChain: p.SolanaChainID,
		DstChain: p.EthChainID,
		Config: SolanaToEthModuleConfig{
			SolanaRpcUrl:         p.SolanaRPC,
			SolanaIcs26ProgramId: p.ICS26ProgramID,
			EthRpcUrl:            p.EthRPC,
			Ics26Address:         p.ICS26Address,
			Mode:                 AttestedMode{Config: aggConfig},
		},
	}
	b.modules = append(b.modules, module)
	return b
}

// =============================================================================
// Helper constructors for common prover configurations
// =============================================================================

// MockProver returns an SP1ProverConfig for mock proving.
func MockProver() SP1ProverConfig {
	return SP1ProverConfig{Type: "mock"}
}

// NetworkProver returns an SP1ProverConfig for network proving.
func NetworkProver(privateKey string, privateCluster bool) SP1ProverConfig {
	return SP1ProverConfig{
		Type:              "network",
		NetworkPrivateKey: privateKey,
		PrivateCluster:    privateCluster,
	}
}

// NetworkProverFromEnv returns an SP1ProverConfig using environment variables.
func NetworkProverFromEnv() SP1ProverConfig {
	return SP1ProverConfig{
		Type:              "network",
		NetworkPrivateKey: os.Getenv("NETWORK_PRIVATE_KEY"),
		PrivateCluster:    os.Getenv("SP1_PRIVATE_CLUSTER") == "true",
	}
}
