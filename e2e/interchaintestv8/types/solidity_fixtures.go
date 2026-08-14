// SPDX-License-Identifier: Apache-2.0

package types

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

type SolidityFixtureGenerator struct {
	Enabled           bool
	sp1GenesisFixture *Sp1GenesisFixture
}

// NewSolidityFixtureGenerator creates a new SolidityFixtureGenerator
func NewSolidityFixtureGenerator() *SolidityFixtureGenerator {
	return &SolidityFixtureGenerator{
		Enabled: os.Getenv(testvalues.EnvKeyGenerateSolidityFixtures) == testvalues.EnvValueGenerateFixtures_True,
	}
}

// Sp1GenesisFixture is the genesis fixture for the sp1 light client
type Sp1GenesisFixture struct {
	// The trusted client state of the sp1 light client
	TrustedClientState string `json:"trustedClientState"`
	// The trusted consensus state of the sp1 light client
	TrustedConsensusStateHash string `json:"trustedConsensusStateHash"`
	// The vkey for the update client program
	UpdateClientVkey string `json:"updateClientVkey"`
	// The vkey for the membership program
	MembershipVkey string `json:"membershipVkey"`
	// The vkey for the update client and membership program
	UcAndMembershipVkey string `json:"ucAndMembershipVkey"`
	// The vkey for the misbehaviour program
	MisbehaviourVkey string `json:"misbehaviourVkey"`
}

// GenericSolidityFixture is the fixture to be unmarshalled into a test case in Solidity tests
type GenericSolidityFixture struct {
	// Hex encoded bytes for sp1 genesis fixture
	Sp1GenesisFixture string `json:"sp1GenesisFixture"`
	// Hex encoded bytes to be fed into the router contract
	Msg string `json:"msg"`
	// Hex encoded bytes for the IICS26RouterMsgsPacket in the context of this fixture
	Packet string `json:"packet"`
	// The contract address of the ERC20 token
	Erc20Address string `json:"erc20Address"`
	// The timestamp in seconds around the time of submitting the Msg to the router contract
	Timestamp int64 `json:"timestamp"`
}

// GenerateAndSaveSolidityFixture generates a fixture and saves it to a file
func (g *SolidityFixtureGenerator) GenerateAndSaveSolidityFixture(fileName, erc20Address string, msgBz []byte, packet ics26router.IICS26RouterMsgsPacket) error {
	if !g.Enabled {
		return nil
	}

	fixture, err := g.generateFixture(erc20Address, msgBz, packet)
	if err != nil {
		return err
	}

	fixtureBz, err := json.Marshal(fixture)
	if err != nil {
		return err
	}

	filePath := testvalues.SolidityFixturesDir + "/" + fileName
	// nolint:gosec
	return os.WriteFile(filePath, fixtureBz, 0o644)
}

func (g *SolidityFixtureGenerator) generateFixture(erc20Address string, msgBz []byte, packet ics26router.IICS26RouterMsgsPacket) (GenericSolidityFixture, error) {
	genesisBz, err := g.GetGenesisFixture()
	if err != nil {
		return GenericSolidityFixture{}, err
	}

	packetBz, err := AbiEncodePacket(packet)
	if err != nil {
		return GenericSolidityFixture{}, err
	}

	// Generate the fixture
	fixture := GenericSolidityFixture{
		Sp1GenesisFixture: hex.EncodeToString(genesisBz),
		Msg:               hex.EncodeToString(msgBz),
		Erc20Address:      erc20Address,
		Timestamp:         time.Now().Unix(),
		Packet:            hex.EncodeToString(packetBz),
	}
	return fixture, nil
}

func (g *SolidityFixtureGenerator) GetGenesisFixture() ([]byte, error) {
	genesisBz, err := json.Marshal(g.sp1GenesisFixture)
	if err != nil {
		return nil, err
	}

	return genesisBz, nil
}

func (g *SolidityFixtureGenerator) SetGenesisFixture(
	clientState []byte, consensusStateHash, updateClientVkey,
	membershipVkey, ucAndMembershipVkey, misbehaviourVkey [32]byte,
) {
	if !g.Enabled {
		return
	}

	g.sp1GenesisFixture = &Sp1GenesisFixture{
		TrustedClientState:        hex.EncodeToString(clientState),
		TrustedConsensusStateHash: hex.EncodeToString(consensusStateHash[:]),
		UpdateClientVkey:          hex.EncodeToString(updateClientVkey[:]),
		MembershipVkey:            hex.EncodeToString(membershipVkey[:]),
		UcAndMembershipVkey:       hex.EncodeToString(ucAndMembershipVkey[:]),
		MisbehaviourVkey:          hex.EncodeToString(misbehaviourVkey[:]),
	}
}

func AbiEncodePacket(packet ics26router.IICS26RouterMsgsPacket) ([]byte, error) {
	structType, err := abi.NewType("tuple", "", []abi.ArgumentMarshaling{
		{Name: "sequence", Type: "uint64"},
		{Name: "sourceClient", Type: "string"},
		{Name: "destClient", Type: "string"},
		{Name: "timeoutTimestamp", Type: "uint64"},
		{Name: "payloads", Type: "tuple[]", Components: []abi.ArgumentMarshaling{
			{Name: "sourcePort", Type: "string"},
			{Name: "destPort", Type: "string"},
			{Name: "version", Type: "string"},
			{Name: "encoding", Type: "string"},
			{Name: "value", Type: "bytes"},
		}},
	})
	if err != nil {
		return nil, err
	}

	args := abi.Arguments{
		{Type: structType},
	}

	return args.Pack(packet)
}

// PacketCompact represents a compact packet with path and commitment
type PacketCompact struct {
	Path       [32]byte
	Commitment [32]byte
}

// PacketAttestation represents the attested packet data
type PacketAttestation struct {
	Height  uint64
	Packets []PacketCompact
}

// AbiDecodePacketAttestation decodes ABI-encoded PacketAttestation data
func AbiDecodePacketAttestation(attestedData []byte) (*PacketAttestation, error) {
	// Define the PacketAttestation struct type
	packetAttestationType, err := abi.NewType("tuple", "", []abi.ArgumentMarshaling{
		{Name: "height", Type: "uint64"},
		{Name: "packets", Type: "tuple[]", Components: []abi.ArgumentMarshaling{
			{Name: "path", Type: "bytes32"},
			{Name: "commitment", Type: "bytes32"},
		}},
	})
	if err != nil {
		return nil, err
	}

	args := abi.Arguments{
		{Type: packetAttestationType},
	}

	// Unpack the data
	unpacked, err := args.Unpack(attestedData)
	if err != nil {
		return nil, err
	}

	// Extract the decoded struct
	decoded := unpacked[0].(struct {
		Height  uint64 `json:"height"`
		Packets []struct {
			Path       [32]byte `json:"path"`
			Commitment [32]byte `json:"commitment"`
		} `json:"packets"`
	})

	// Convert to our PacketAttestation type
	packets := make([]PacketCompact, len(decoded.Packets))
	for i, p := range decoded.Packets {
		packets[i] = PacketCompact{
			Path:       p.Path,
			Commitment: p.Commitment,
		}
	}

	return &PacketAttestation{
		Height:  decoded.Height,
		Packets: packets,
	}, nil
}
